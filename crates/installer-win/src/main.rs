//! The Challenger 650 installer window: plain Win32 controls, no UI framework.
//!
//! The window gathers the choices, remembers them, and runs the engine's job
//! on a worker thread. Events from workers arrive as `Message`s and are
//! handled on the window's thread, which is the only one that touches controls.

#![windows_subsystem = "windows"]

mod shortcut;

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use engine::job::{self, Cancel, Event, Outcome, Request};
use engine::log::{self, LogFile};
use engine::paths;
use engine::settings::{Release, Settings, Texture};
use engine::status::{self, Latest};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW, DeleteDC, DeleteObject, GdiFlush, GetDC, GetDIBits, GetSysColorBrush,
    RedrawWindow, ReleaseDC, SelectObject, UpdateWindow, BITMAPINFO, BITMAPINFOHEADER, COLOR_BTNFACE, DIB_RGB_COLORS, HFONT, LOGFONTW,
    RDW_ALLCHILDREN, RDW_INVALIDATE, RDW_UPDATENOW, SRCCOPY,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{InitCommonControlsEx, ICC_PROGRESS_CLASS, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX};
use windows::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForSystem, GetDpiForWindow};
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled};
use windows::Win32::UI::Shell::{
    FileOpenDialog, IFileOpenDialog, IShellItem, SHCreateItemFromParsingName, ShellExecuteW, FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetAncestor, GetClientRect, GetDlgItem, GetMessageW, GetSystemMetrics,
    GetWindowRect,
    GetWindowTextLengthW, GetWindowTextW, IsDialogMessageW, KillTimer, LoadCursorW, MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassExW,
    SendMessageW, SetTimer, SetWindowTextW, ShowWindow, TranslateMessage, GA_ROOT, HMENU, IDC_ARROW, MB_ICONWARNING, MB_OK, MSG, SM_CXSCREEN,
    SM_CYSCREEN, SW_SHOW, SW_SHOWNORMAL, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_SETFONT, WM_TIMER,
    WNDCLASSEXW, WS_CAPTION, WS_CHILD, WS_EX_CLIENTEDGE, WS_GROUP, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};

const TITLE: &str = "Challenger 650 Installer";

// Control styles and messages, by their Win32 values.
const BS_PUSHBUTTON: u32 = 0x0;
const BS_DEFPUSHBUTTON: u32 = 0x1;
const BS_AUTOCHECKBOX: u32 = 0x3;
const BS_AUTORADIOBUTTON: u32 = 0x9;
const ES_MULTILINE: u32 = 0x4;
const ES_AUTOVSCROLL: u32 = 0x40;
const ES_AUTOHSCROLL: u32 = 0x80;
const ES_READONLY: u32 = 0x800;
const SS_PATHELLIPSIS: u32 = 0x8000;
const BM_GETCHECK: u32 = 0xF0;
const BM_SETCHECK: u32 = 0xF1;
const BM_CLICK: u32 = 0xF5;
const EM_SETSEL: u32 = 0xB1;
const EM_SCROLLCARET: u32 = 0xB7;
const EM_REPLACESEL: u32 = 0xC2;
const EM_GETLINECOUNT: u32 = 0xBA;
const EM_GETFIRSTVISIBLELINE: u32 = 0xCE;
const PBM_SETPOS: u32 = 0x402;
const PBM_SETRANGE32: u32 = 0x406;
const EN_KILLFOCUS: u32 = 0x200;
const EN_CHANGE: u32 = 0x300;
const IDCANCEL: i32 = 2;

/// Posted to the window when a worker has put a `Message` in the queue.
const WM_EVENTS: u32 = WM_APP + 1;

const ID_SIM_EDIT: i32 = 101;
const ID_SIM_BROWSE: i32 = 102;
const ID_INSTALL_EDIT: i32 = 103;
const ID_INSTALL_BROWSE: i32 = 104;
const ID_TEXTURE_FIRST: i32 = 110;
const ID_RELEASE_FIRST: i32 = 115;
const ID_OPTIMISE: i32 = 120;
const ID_SHORTCUT: i32 = 121;
const ID_STATUS: i32 = 130;
const ID_PROGRESS: i32 = 131;
const ID_DETAIL: i32 = 132;
const ID_REPAIR: i32 = 140;
const ID_UNINSTALL: i32 = 141;
const ID_LOGS: i32 = 142;
const ID_INSTALL: i32 = 143;
const ID_CLOSE: i32 = 144;
const ID_LOG: i32 = 150;

/// Everything that is locked while a job runs.
const CHOICE_IDS: [i32; 16] = [
    ID_SIM_EDIT, ID_SIM_BROWSE, ID_INSTALL_EDIT, ID_INSTALL_BROWSE, 110, 111, 112, 113, 115, 116, 117, ID_OPTIMISE, ID_SHORTCUT, ID_REPAIR, ID_UNINSTALL,
    ID_INSTALL,
];

/// Column offset and width of each release radio button.
const RELEASE_COLUMNS: [(i32, i32); 3] = [(0, 80), (86, 104), (196, 90)];

const TIMER_SCREENSHOT: usize = 99;
const TIMER_CANCEL: usize = 98;

#[derive(Clone, Copy, Default)]
struct Ui {
    instance: HINSTANCE,
    font: HFONT,
    dpi: u32,
}

/// What workers send to the window.
enum Message {
    Latest { check: u64, latest: Latest },
    Job(Event),
}

struct State {
    settings_path: Option<PathBuf>,
    log_dir: Option<PathBuf>,
    /// The Start menu folder the shortcut goes in.
    shortcut_dir: Option<PathBuf>,
    log: LogFile,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    /// Present while a job is running.
    cancel: Option<Cancel>,
    close_when_done: bool,
    /// The user chose the install folder, so a new sim folder must not move it.
    install_dir_chosen: bool,
    repo: String,
    latest: Latest,
    /// Counts version checks, so an answer to an older question is ignored.
    latest_check: u64,
    progress_total: usize,
    step_pause: Duration,
}

thread_local! {
    static UI: Cell<Ui> = Cell::new(Ui::default());
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
    /// Set while the window itself writes into an edit box.
    static WRITING: Cell<bool> = const { Cell::new(false) };
    static SCREENSHOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static DUMP: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

fn ui() -> Ui {
    UI.with(|u| u.get())
}

/// Reads or changes the state. Never call into Win32 from inside `change`:
/// a control can send a message straight back and find the state borrowed.
fn state<R>(change: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| change(s.borrow_mut().as_mut().expect("state is set before the window is created")))
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Layout is written for 96 dpi and scaled to the screen.
fn px(value: i32) -> i32 {
    (value as i64 * ui().dpi as i64 + 48) as i32 / 96
}

unsafe fn control(parent: HWND, class: PCWSTR, text: &str, style: u32, ex_style: u32, rect: [i32; 4], id: i32) -> HWND {
    let text = wide(text);
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(ex_style),
        class,
        PCWSTR(text.as_ptr()),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | style),
        px(rect[0]),
        px(rect[1]),
        px(rect[2]),
        px(rect[3]),
        Some(parent),
        Some(HMENU(id as isize as *mut c_void)),
        Some(ui().instance),
        None,
    )
    .unwrap_or_default();
    SendMessageW(hwnd, WM_SETFONT, Some(WPARAM(ui().font.0 as usize)), Some(LPARAM(1)));
    hwnd
}

unsafe fn item(parent: HWND, id: i32) -> HWND {
    GetDlgItem(Some(parent), id).unwrap_or_default()
}

unsafe fn text_of(hwnd: HWND) -> String {
    let mut buffer = vec![0u16; GetWindowTextLengthW(hwnd) as usize + 1];
    let length = GetWindowTextW(hwnd, &mut buffer) as usize;
    String::from_utf16_lossy(&buffer[..length])
}

unsafe fn set_text(hwnd: HWND, text: &str) {
    let text = wide(text);
    WRITING.with(|w| w.set(true));
    let _ = SetWindowTextW(hwnd, PCWSTR(text.as_ptr()));
    WRITING.with(|w| w.set(false));
}

unsafe fn checked(hwnd: HWND) -> bool {
    SendMessageW(hwnd, BM_GETCHECK, None, None).0 == 1
}

unsafe fn set_checked(hwnd: HWND, on: bool) {
    SendMessageW(hwnd, BM_SETCHECK, Some(WPARAM(on as usize)), None);
}

/// Which radio button of a row is on.
unsafe fn selected(hwnd: HWND, first_id: i32, count: usize) -> usize {
    (0..count).find(|&i| checked(item(hwnd, first_id + i as i32))).unwrap_or(0)
}

unsafe fn select(hwnd: HWND, first_id: i32, count: usize, index: usize) {
    for i in 0..count {
        set_checked(item(hwnd, first_id + i as i32), i == index);
    }
}

unsafe fn warn(owner: HWND, text: &str) {
    let (text, title) = (wide(text), wide(TITLE));
    MessageBoxW(Some(owner), PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONWARNING);
}

/// Adds a time-stamped line to the log box and the log file.
unsafe fn log_line(hwnd: HWND, text: &str) {
    let line = log::stamped(text);
    state(|s| s.log.line(&line));
    let edit = item(hwnd, ID_LOG);
    let end = GetWindowTextLengthW(edit);
    let addition = wide(&if end == 0 { line } else { format!("\r\n{line}") });
    SendMessageW(edit, EM_SETSEL, Some(WPARAM(end as usize)), Some(LPARAM(end as isize)));
    SendMessageW(edit, EM_REPLACESEL, Some(WPARAM(0)), Some(LPARAM(addition.as_ptr() as isize)));
    SendMessageW(edit, EM_SCROLLCARET, None, None);
}

unsafe fn pick_folder(owner: HWND, title: &str, start: &Path) -> Option<PathBuf> {
    let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
    let options = dialog.GetOptions().ok()?;
    dialog.SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM).ok()?;
    let title = wide(title);
    let _ = dialog.SetTitle(PCWSTR(title.as_ptr()));
    let mut folder = start.to_path_buf();
    while !folder.is_dir() && folder.pop() {}
    if folder.is_dir() {
        let folder = wide(&folder.to_string_lossy());
        if let Ok(shell_item) = SHCreateItemFromParsingName::<_, _, IShellItem>(PCWSTR(folder.as_ptr()), None) {
            let _ = dialog.SetFolder(&shell_item);
        }
    }
    dialog.Show(Some(owner)).ok()?;
    let name = dialog.GetResult().ok()?.GetDisplayName(SIGDN_FILESYSPATH).ok()?;
    let picked = name.to_string().ok();
    CoTaskMemFree(Some(name.0 as *const c_void));
    picked.map(|p| paths::normalise(&p))
}

/// A function workers call to hand a message to the window's thread.
fn notifier(hwnd: HWND) -> impl Fn(Message) + Send + 'static {
    let (sender, target) = (state(|s| s.sender.clone()), hwnd.0 as isize);
    move |message| {
        if sender.send(message).is_ok() {
            unsafe {
                let _ = PostMessageW(Some(HWND(target as *mut c_void)), WM_EVENTS, WPARAM(0), LPARAM(0));
            }
        }
    }
}

unsafe fn release_of(hwnd: HWND) -> Release {
    Release::from_index(selected(hwnd, ID_RELEASE_FIRST, Release::ALL.len()))
}

/// The choices as they stand in the window.
unsafe fn gather(hwnd: HWND) -> Settings {
    Settings {
        sim_dir: text_of(item(hwnd, ID_SIM_EDIT)).trim().to_owned(),
        install_dir: text_of(item(hwnd, ID_INSTALL_EDIT)).trim().to_owned(),
        install_dir_chosen: state(|s| s.install_dir_chosen),
        texture: Texture::from_index(selected(hwnd, ID_TEXTURE_FIRST, Texture::ALL.len())),
        release: release_of(hwnd),
        optimise: checked(item(hwnd, ID_OPTIMISE)),
        repo: state(|s| s.repo.clone()),
    }
}

unsafe fn save_settings(hwnd: HWND) {
    let settings = gather(hwnd);
    if let Some(path) = state(|s| s.settings_path.clone()) {
        let _ = settings.save(&path);
    }
}

unsafe fn refresh_status(hwnd: HWND) {
    let installed = status::installed_version(&paths::normalise(&text_of(item(hwnd, ID_INSTALL_EDIT))));
    let latest = state(|s| s.latest.label());
    set_text(item(hwnd, ID_STATUS), &format!("Installed: {}        Latest: {latest}", installed.as_deref().unwrap_or("none")));
}

/// Asks, on a worker thread, what the newest version of the chosen release is.
unsafe fn check_latest(hwnd: HWND) {
    let release = release_of(hwnd);
    let (check, repo) = state(|s| {
        s.latest_check += 1;
        s.latest = if s.repo.trim().is_empty() { Latest::NoSource } else { Latest::Checking };
        (s.latest_check, s.repo.clone())
    });
    refresh_status(hwnd);
    if !repo.trim().is_empty() {
        let notify = notifier(hwnd);
        std::thread::spawn(move || notify(Message::Latest { check, latest: status::latest(&repo, release) }));
    }
}

unsafe fn latest_arrived(hwnd: HWND, check: u64, latest: Latest) {
    if state(|s| s.latest_check) != check {
        return;
    }
    let kind = release_of(hwnd).label().to_lowercase();
    match &latest {
        Latest::Version(version) => log_line(hwnd, &format!("Latest ({kind}): {version}")),
        Latest::NonePublished => log_line(hwnd, &format!("Nothing has been published yet ({kind}).")),
        Latest::Unknown(reason) => log_line(hwnd, &format!("The latest version could not be checked: {reason}.")),
        Latest::NoSource | Latest::Checking => {}
    }
    state(|s| s.latest = latest);
    refresh_status(hwnd);
}

unsafe fn build_main(hwnd: HWND) {
    const LEFT: i32 = 14;
    const WIDTH: i32 = 492;
    const BROWSE: i32 = 84;
    const OPTIONS: i32 = LEFT + 96;
    let edit_width = WIDTH - BROWSE - 8;
    let tab = WS_TABSTOP.0;

    control(hwnd, w!("STATIC"), "Sim directory", 0, 0, [LEFT, 14, WIDTH, 16], -1);
    control(hwnd, w!("EDIT"), "", ES_AUTOHSCROLL | tab, WS_EX_CLIENTEDGE.0, [LEFT, 32, edit_width, 24], ID_SIM_EDIT);
    control(hwnd, w!("BUTTON"), "Browse...", BS_PUSHBUTTON | tab, 0, [LEFT + edit_width + 8, 32, BROWSE, 24], ID_SIM_BROWSE);

    control(hwnd, w!("STATIC"), "Install directory", 0, 0, [LEFT, 66, WIDTH, 16], -1);
    control(hwnd, w!("EDIT"), "", ES_AUTOHSCROLL | tab, WS_EX_CLIENTEDGE.0, [LEFT, 84, edit_width, 24], ID_INSTALL_EDIT);
    control(hwnd, w!("BUTTON"), "Browse...", BS_PUSHBUTTON | tab, 0, [LEFT + edit_width + 8, 84, BROWSE, 24], ID_INSTALL_BROWSE);

    control(hwnd, w!("STATIC"), "Texture size", 0, 0, [LEFT, 126, 90, 16], -1);
    for (index, texture) in Texture::ALL.iter().enumerate() {
        let group = if index == 0 { WS_GROUP.0 | tab } else { 0 };
        control(hwnd, w!("BUTTON"), texture.label(), BS_AUTORADIOBUTTON | group, 0, [OPTIONS + index as i32 * 62, 124, 54, 20], ID_TEXTURE_FIRST + index as i32);
    }

    control(hwnd, w!("STATIC"), "Release", 0, 0, [LEFT, 154, 90, 16], -1);
    for (index, release) in Release::ALL.iter().enumerate() {
        let group = if index == 0 { WS_GROUP.0 | tab } else { 0 };
        let (offset, width) = RELEASE_COLUMNS[index];
        control(hwnd, w!("BUTTON"), release.label(), BS_AUTORADIOBUTTON | group, 0, [OPTIONS + offset, 152, width, 20], ID_RELEASE_FIRST + index as i32);
    }

    control(hwnd, w!("BUTTON"), "Optimise performance (fewer triangles)", BS_AUTOCHECKBOX | WS_GROUP.0 | tab, 0, [LEFT, 182, 270, 20], ID_OPTIMISE);
    control(hwnd, w!("BUTTON"), "Start menu shortcut", BS_AUTOCHECKBOX | tab, 0, [LEFT + 300, 182, 190, 20], ID_SHORTCUT);

    control(hwnd, w!("STATIC"), "", 0, 0, [LEFT, 218, WIDTH, 16], ID_STATUS);
    control(hwnd, w!("msctls_progress32"), "", 0, 0, [LEFT, 238, WIDTH, 16], ID_PROGRESS);
    control(hwnd, w!("STATIC"), "", SS_PATHELLIPSIS, 0, [LEFT, 258, WIDTH, 16], ID_DETAIL);

    control(hwnd, w!("BUTTON"), "Repair", BS_PUSHBUTTON | WS_GROUP.0 | tab, 0, [LEFT, 286, 80, 28], ID_REPAIR);
    control(hwnd, w!("BUTTON"), "Uninstall", BS_PUSHBUTTON | tab, 0, [LEFT + 88, 286, 80, 28], ID_UNINSTALL);
    control(hwnd, w!("BUTTON"), "Open log folder", BS_PUSHBUTTON | tab, 0, [LEFT + 176, 286, 110, 28], ID_LOGS);
    control(hwnd, w!("BUTTON"), "Install", BS_DEFPUSHBUTTON | tab, 0, [LEFT + WIDTH - 188, 286, 90, 28], ID_INSTALL);
    control(hwnd, w!("BUTTON"), "Close", BS_PUSHBUTTON | tab, 0, [LEFT + WIDTH - 90, 286, 90, 28], ID_CLOSE);

    control(hwnd, w!("EDIT"), "", ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY | WS_VSCROLL.0, WS_EX_CLIENTEDGE.0, [LEFT, 326, WIDTH, 92], ID_LOG);

    restore_choices(hwnd);
}

/// Fills the window from the settings file, finding X-Plane if no usable
/// folder was remembered.
unsafe fn restore_choices(hwnd: HWND) {
    let path = state(|s| s.settings_path.clone());
    let remembered = path.as_deref().is_some_and(Path::is_file);
    let mut settings = path.as_deref().map(Settings::load).unwrap_or_default();

    let mut found = None;
    if !paths::is_xplane_root(&paths::normalise(&settings.sim_dir)) {
        found = paths::detect_xplane();
        if let Some(sim) = &found {
            settings.sim_dir = sim.to_string_lossy().into_owned();
        }
    }
    if !settings.sim_dir.is_empty() && (!settings.install_dir_chosen || settings.install_dir.is_empty()) {
        settings.install_dir = paths::default_install_dir(&paths::normalise(&settings.sim_dir)).to_string_lossy().into_owned();
        settings.install_dir_chosen = false;
    }
    let shortcut_dir = state(|s| {
        s.install_dir_chosen = settings.install_dir_chosen;
        if s.repo.is_empty() {
            s.repo = settings.repo.clone();
        }
        s.shortcut_dir.clone()
    });

    set_text(item(hwnd, ID_SIM_EDIT), &settings.sim_dir);
    set_text(item(hwnd, ID_INSTALL_EDIT), &settings.install_dir);
    select(hwnd, ID_TEXTURE_FIRST, Texture::ALL.len(), settings.texture.index());
    select(hwnd, ID_RELEASE_FIRST, Release::ALL.len(), settings.release.index());
    set_checked(item(hwnd, ID_OPTIMISE), settings.optimise);
    set_checked(item(hwnd, ID_SHORTCUT), shortcut_dir.as_deref().is_some_and(shortcut::exists));
    set_text(item(hwnd, ID_DETAIL), settings.release.note());
    set_text(item(hwnd, ID_INSTALL), if settings.release == Release::Source { "Build" } else { "Install" });

    log_line(hwnd, "Installer started.");
    if remembered {
        log_line(hwnd, "Choices restored from last time.");
    }
    if let Some(sim) = found {
        log_line(hwnd, &format!("X-Plane 12 found at {}", sim.display()));
    } else if !paths::is_xplane_root(&paths::normalise(&settings.sim_dir)) {
        log_line(hwnd, "X-Plane 12 was not found. Choose its folder with Browse.");
        set_text(item(hwnd, ID_DETAIL), "X-Plane 12 was not found. Choose its folder with Browse.");
    }
    check_latest(hwnd);
}

unsafe fn sim_dir_changed(hwnd: HWND) {
    if !state(|s| s.install_dir_chosen) {
        let sim = paths::normalise(&text_of(item(hwnd, ID_SIM_EDIT)));
        if !sim.as_os_str().is_empty() {
            set_text(item(hwnd, ID_INSTALL_EDIT), &paths::default_install_dir(&sim).to_string_lossy());
        }
    }
    refresh_status(hwnd);
}

unsafe fn release_changed(hwnd: HWND) {
    let release = release_of(hwnd);
    set_text(item(hwnd, ID_DETAIL), release.note());
    set_text(item(hwnd, ID_INSTALL), if release == Release::Source { "Build" } else { "Install" });
    save_settings(hwnd);
    check_latest(hwnd);
}

/// The tick box is the shortcut: ticking adds it, clearing removes it.
unsafe fn shortcut_clicked(hwnd: HWND) {
    let wanted = checked(item(hwnd, ID_SHORTCUT));
    let Some(dir) = state(|s| s.shortcut_dir.clone()) else {
        log_line(hwnd, "The Start menu folder could not be found.");
        set_checked(item(hwnd, ID_SHORTCUT), false);
        return;
    };
    let result = if wanted {
        std::env::current_exe().map_err(|e| e.to_string()).and_then(|exe| shortcut::create(&dir, &exe)).map(|_| "Start menu shortcut added.")
    } else {
        shortcut::remove(&dir).map_err(|e| e.to_string()).map(|_| "Start menu shortcut removed.")
    };
    match result {
        Ok(text) => log_line(hwnd, text),
        Err(reason) => {
            log_line(hwnd, &format!("The Start menu shortcut could not be changed: {reason}"));
            set_checked(item(hwnd, ID_SHORTCUT), shortcut::exists(&dir));
        }
    }
}

unsafe fn set_progress(hwnd: HWND, position: usize, total: usize) {
    let bar = item(hwnd, ID_PROGRESS);
    SendMessageW(bar, PBM_SETRANGE32, Some(WPARAM(0)), Some(LPARAM(total.max(1) as isize)));
    SendMessageW(bar, PBM_SETPOS, Some(WPARAM(position)), None);
}

/// Locks the choices while a job runs and turns Close into Cancel.
unsafe fn set_working(hwnd: HWND, working: bool) {
    for id in CHOICE_IDS {
        let _ = EnableWindow(item(hwnd, id), !working);
    }
    let close = item(hwnd, ID_CLOSE);
    set_text(close, if working { "Cancel" } else { "Close" });
    let _ = EnableWindow(close, true);
}

unsafe fn start_job(hwnd: HWND) {
    let settings = gather(hwnd);
    let request = Request {
        sim_dir: paths::normalise(&settings.sim_dir),
        install_dir: paths::normalise(&settings.install_dir),
        texture: settings.texture,
        release: settings.release,
        optimise: settings.optimise,
        repo: settings.repo.clone(),
    };
    if !paths::is_xplane_root(&request.sim_dir) {
        warn(hwnd, "That folder is not an X-Plane 12 install.\n\nChoose the folder that contains X-Plane.exe.");
        return;
    }
    if request.install_dir.as_os_str().is_empty() {
        warn(hwnd, "Choose where the aircraft should be installed.");
        return;
    }
    save_settings(hwnd);

    let cancel = Cancel::new();
    let pause = state(|s| {
        s.cancel = Some(cancel.clone());
        s.close_when_done = false;
        s.step_pause
    });
    set_working(hwnd, true);
    set_progress(hwnd, 0, 1);
    log_line(hwnd, if request.release == Release::Source { "Build started." } else { "Install started." });

    let notify = notifier(hwnd);
    std::thread::spawn(move || {
        job::run(job::preflight(request), &cancel, pause, &mut |event| notify(Message::Job(event)));
    });
}

unsafe fn cancel_job(hwnd: HWND) {
    if let Some(cancel) = state(|s| s.cancel.clone()) {
        if !cancel.is_cancelled() {
            cancel.cancel();
            let close = item(hwnd, ID_CLOSE);
            set_text(close, "Cancelling...");
            let _ = EnableWindow(close, false);
            log_line(hwnd, "Cancelling...");
        }
    }
}

unsafe fn job_event(hwnd: HWND, event: Event) {
    match event {
        Event::Step { index, total, name } => {
            state(|s| s.progress_total = total);
            set_progress(hwnd, index, total);
            set_text(item(hwnd, ID_DETAIL), &name);
            log_line(hwnd, &name);
        }
        Event::Log(line) => log_line(hwnd, &line),
        Event::Done(outcome) => {
            let total = state(|s| s.progress_total);
            let text = match outcome {
                Outcome::Finished(message) => {
                    set_progress(hwnd, total, total);
                    message
                }
                Outcome::Cancelled => {
                    set_progress(hwnd, 0, total);
                    "Cancelled.".to_owned()
                }
                Outcome::Failed(reason) => {
                    set_progress(hwnd, 0, total);
                    format!("Stopped: {reason}")
                }
            };
            log_line(hwnd, &text);
            set_text(item(hwnd, ID_DETAIL), &text);
            let close = state(|s| {
                s.cancel = None;
                std::mem::take(&mut s.close_when_done)
            });
            set_working(hwnd, false);
            refresh_status(hwnd);
            if close {
                let _ = DestroyWindow(hwnd);
            }
        }
    }
}

/// Handles everything workers have queued since the last call.
unsafe fn drain_events(hwnd: HWND) {
    loop {
        let Some(message) = state(|s| s.receiver.try_recv().ok()) else { break };
        match message {
            Message::Latest { check, latest } => latest_arrived(hwnd, check, latest),
            Message::Job(event) => job_event(hwnd, event),
        }
    }
}

unsafe fn open_log_folder(hwnd: HWND) {
    let Some(folder) = state(|s| s.log_dir.clone()) else { return };
    if std::fs::create_dir_all(&folder).is_err() {
        log_line(hwnd, "The log folder could not be created.");
        return;
    }
    let folder = wide(&folder.to_string_lossy());
    ShellExecuteW(Some(hwnd), w!("open"), PCWSTR(folder.as_ptr()), None, None, SW_SHOWNORMAL);
}

unsafe fn not_built(hwnd: HWND, action: &str) {
    let text = format!("{action} needs the install engine, which is not built yet.");
    log_line(hwnd, &text);
    set_text(item(hwnd, ID_DETAIL), &text);
}

unsafe extern "system" fn main_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            build_main(hwnd);
            LRESULT(0)
        }
        WM_COMMAND => {
            let (id, notification) = ((wparam.0 & 0xFFFF) as i32, ((wparam.0 >> 16) & 0xFFFF) as u32);
            let typed = notification == EN_CHANGE && !WRITING.with(|w| w.get());
            match id {
                ID_SIM_EDIT if typed => sim_dir_changed(hwnd),
                ID_INSTALL_EDIT if typed => {
                    state(|s| s.install_dir_chosen = true);
                    refresh_status(hwnd);
                }
                ID_SIM_EDIT | ID_INSTALL_EDIT if notification == EN_KILLFOCUS => save_settings(hwnd),
                ID_SIM_EDIT | ID_INSTALL_EDIT | ID_LOG => {}
                ID_SIM_BROWSE => {
                    let start = paths::normalise(&text_of(item(hwnd, ID_SIM_EDIT)));
                    if let Some(folder) = pick_folder(hwnd, "Choose your X-Plane 12 folder", &start) {
                        set_text(item(hwnd, ID_SIM_EDIT), &folder.to_string_lossy());
                        sim_dir_changed(hwnd);
                        save_settings(hwnd);
                    }
                }
                ID_INSTALL_BROWSE => {
                    let start = paths::normalise(&text_of(item(hwnd, ID_INSTALL_EDIT)));
                    if let Some(folder) = pick_folder(hwnd, "Choose where to install the Challenger 650", &start) {
                        set_text(item(hwnd, ID_INSTALL_EDIT), &folder.to_string_lossy());
                        state(|s| s.install_dir_chosen = true);
                        refresh_status(hwnd);
                        save_settings(hwnd);
                    }
                }
                ID_SHORTCUT => shortcut_clicked(hwnd),
                _ if (ID_RELEASE_FIRST..ID_RELEASE_FIRST + Release::ALL.len() as i32).contains(&id) => release_changed(hwnd),
                _ if (ID_TEXTURE_FIRST..ID_TEXTURE_FIRST + Texture::ALL.len() as i32).contains(&id) || id == ID_OPTIMISE => save_settings(hwnd),
                ID_REPAIR => not_built(hwnd, "Repair"),
                ID_UNINSTALL => not_built(hwnd, "Uninstall"),
                ID_LOGS => open_log_folder(hwnd),
                ID_INSTALL => start_job(hwnd),
                ID_CLOSE | IDCANCEL => {
                    if state(|s| s.cancel.is_some()) {
                        cancel_job(hwnd);
                    } else {
                        let _ = DestroyWindow(hwnd);
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_EVENTS => {
            drain_events(hwnd);
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER_CANCEL => {
            let _ = KillTimer(Some(hwnd), TIMER_CANCEL);
            let _ = PostMessageW(Some(item(hwnd, ID_CLOSE)), BM_CLICK, WPARAM(0), LPARAM(0));
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER_SCREENSHOT => {
            let _ = KillTimer(Some(hwnd), TIMER_SCREENSHOT);
            if let Some(path) = DUMP.with(|d| d.borrow_mut().take()) {
                let _ = std::fs::write(path, describe(hwnd));
            }
            if let Some(path) = SCREENSHOT.with(|s| s.borrow_mut().take()) {
                let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN);
                let _ = save_screenshot(hwnd, &path);
            }
            // A job still running must not outlive the window it reports to.
            if let Some(cancel) = state(|s| s.cancel.clone()) {
                cancel.cancel();
            }
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_CLOSE => {
            if state(|s| s.cancel.is_some()) {
                state(|s| s.close_when_done = true);
                cancel_job(hwnd);
            } else {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            save_settings(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Outer position and size for a window whose client area is `width` x `height`
/// at 96 dpi, centred on the screen.
unsafe fn placement(style: WINDOW_STYLE, width: i32, height: i32) -> (i32, i32, i32, i32) {
    let mut rect = RECT { left: 0, top: 0, right: px(width), bottom: px(height) };
    let _ = AdjustWindowRectExForDpi(&mut rect, style, false, WINDOW_EX_STYLE(0), ui().dpi);
    let (outer_width, outer_height) = (rect.right - rect.left, rect.bottom - rect.top);
    ((GetSystemMetrics(SM_CXSCREEN) - outer_width) / 2, (GetSystemMetrics(SM_CYSCREEN) - outer_height) / 2, outer_width, outer_height)
}

/// The window's real state as text: its size, every control's text and
/// whether it is enabled or ticked, and what the log box holds. A picture can
/// be wrong about these; this cannot.
unsafe fn describe(hwnd: HWND) -> String {
    let (mut outer, mut client) = (RECT::default(), RECT::default());
    let _ = GetWindowRect(hwnd, &mut outer);
    let _ = GetClientRect(hwnd, &mut client);
    let mut text = format!(
        "window {}x{} client {}x{} dpi {} layout-dpi {}
",
        outer.right - outer.left,
        outer.bottom - outer.top,
        client.right,
        client.bottom,
        GetDpiForWindow(hwnd),
        ui().dpi
    );
    let ids = [
        ID_SIM_EDIT, ID_SIM_BROWSE, ID_INSTALL_EDIT, ID_INSTALL_BROWSE, 110, 111, 112, 113, 115, 116, 117, ID_OPTIMISE, ID_SHORTCUT, ID_STATUS, ID_DETAIL,
        ID_REPAIR, ID_UNINSTALL, ID_LOGS, ID_INSTALL, ID_CLOSE,
    ];
    for id in ids {
        let control = item(hwnd, id);
        let mut rect = RECT::default();
        let _ = GetWindowRect(control, &mut rect);
        text += &format!(
            "{id} enabled={} checked={} right-edge={} text={:?}
",
            IsWindowEnabled(control).as_bool(),
            checked(control),
            rect.right - outer.left,
            text_of(control)
        );
    }
    let log_box = item(hwnd, ID_LOG);
    text += &format!(
        "log lines={} first-visible={}
{}
",
        SendMessageW(log_box, EM_GETLINECOUNT, None, None).0,
        SendMessageW(log_box, EM_GETFIRSTVISIBLELINE, None, None).0,
        text_of(log_box)
    );
    text
}

/// Saves what the window has drawn in its client area as a 32-bit BMP. The
/// pixels are copied from the window's own surface, so the picture is what
/// the controls painted, not what the desktop compositor last showed.
unsafe fn save_screenshot(hwnd: HWND, path: &Path) -> std::io::Result<()> {
    let mut rect = RECT::default();
    let _ = GetClientRect(hwnd, &mut rect);
    let (width, height) = (rect.right, rect.bottom);
    let window_dc = GetDC(Some(hwnd));
    let memory_dc = CreateCompatibleDC(Some(window_dc));
    let bitmap = CreateCompatibleBitmap(window_dc, width, height);
    let previous = SelectObject(memory_dc, bitmap.into());
    let _ = GdiFlush();
    let _ = BitBlt(memory_dc, 0, 0, width, height, Some(window_dc), 0, 0, SRCCOPY);
    SelectObject(memory_dc, previous);

    let header = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        ..Default::default()
    };
    let mut info = BITMAPINFO { bmiHeader: header, ..Default::default() };
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    GetDIBits(memory_dc, bitmap, 0, height as u32, Some(pixels.as_mut_ptr() as *mut c_void), &mut info, DIB_RGB_COLORS);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(memory_dc);
    ReleaseDC(Some(hwnd), window_dc);

    let mut file = Vec::with_capacity(54 + pixels.len());
    file.extend_from_slice(b"BM");
    file.extend_from_slice(&(54 + pixels.len() as u32).to_le_bytes());
    file.extend_from_slice(&[0; 4]);
    file.extend_from_slice(&54u32.to_le_bytes());
    file.extend_from_slice(&40u32.to_le_bytes());
    file.extend_from_slice(&width.to_le_bytes());
    file.extend_from_slice(&(-height).to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&32u16.to_le_bytes());
    file.extend_from_slice(&[0; 24]);
    file.extend_from_slice(&pixels);
    std::fs::write(path, file)
}

/// Options for driving the window from a script, used to test and to take
/// screenshots:
///
/// - `--settings FILE`  keep choices in FILE, and the log beside it
/// - `--shortcut-dir DIR`  put the Start menu shortcut in DIR instead
/// - `--repo OWNER/NAME`  look for releases there
/// - `--select-texture N`, `--select-release N`  click that option
/// - `--toggle-shortcut`  click the Start menu shortcut box
/// - `--run`  click Install; `--cancel-after-ms N`  then click Cancel
/// - `--step-pause-ms N`  wait after each step of a job
/// - `--screenshot FILE`  save a picture and exit; `--screenshot-after-ms N`
/// - `--dump FILE`  with a screenshot, also write the window's state as text
struct Options {
    arguments: Vec<String>,
}

impl Options {
    fn value(&self, name: &str) -> Option<&String> {
        self.arguments.iter().position(|a| a == name).and_then(|i| self.arguments.get(i + 1))
    }

    fn number(&self, name: &str) -> Option<u64> {
        self.value(name).and_then(|v| v.parse().ok())
    }

    fn has(&self, name: &str) -> bool {
        self.arguments.iter().any(|a| a == name)
    }
}

fn main() {
    let options = Options { arguments: std::env::args().collect() };
    let settings_path = options.value("--settings").map(PathBuf::from).or_else(Settings::default_path);
    let log_dir = match options.value("--settings") {
        Some(path) => Path::new(path).parent().map(|dir| dir.join("logs")),
        None => log::log_dir(),
    };
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let (sender, receiver) = channel();
    STATE.with(|s| {
        *s.borrow_mut() = Some(State {
            settings_path,
            shortcut_dir: options.value("--shortcut-dir").map(PathBuf::from).or_else(shortcut::programs_dir),
            log: log_dir.as_deref().map(LogFile::open).unwrap_or_else(LogFile::none),
            log_dir,
            sender,
            receiver,
            cancel: None,
            close_when_done: false,
            install_dir_chosen: false,
            repo: options.value("--repo").cloned().unwrap_or_default(),
            latest: Latest::NoSource,
            latest_check: 0,
            progress_total: 1,
            step_pause: Duration::from_millis(options.number("--step-pause-ms").unwrap_or(0)),
        });
    });
    SCREENSHOT.with(|s| *s.borrow_mut() = options.value("--screenshot").map(PathBuf::from));
    DUMP.with(|d| *d.borrow_mut() = options.value("--dump").map(PathBuf::from));

    unsafe {
        let _ = InitCommonControlsEx(&INITCOMMONCONTROLSEX { dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32, dwICC: ICC_STANDARD_CLASSES | ICC_PROGRESS_CLASS });

        let instance: HINSTANCE = GetModuleHandleW(None).map(Into::into).unwrap_or_default();
        let dpi = GetDpiForSystem();
        let mut face = LOGFONTW { lfHeight: -((9 * dpi as i32 + 36) / 72), lfWeight: 400, ..Default::default() };
        for (slot, unit) in face.lfFaceName.iter_mut().zip("Segoe UI".encode_utf16()) {
            *slot = unit;
        }
        UI.with(|u| u.set(Ui { instance, dpi, font: CreateFontIndirectW(&face) }));

        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(main_proc),
            hInstance: instance,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
            lpszClassName: w!("CL650Installer"),
            ..Default::default()
        };
        RegisterClassExW(&class);

        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
        let (x, y, width, height) = placement(style, 520, 432);
        let title = wide(TITLE);
        let hwnd = CreateWindowExW(WINDOW_EX_STYLE(0), w!("CL650Installer"), PCWSTR(title.as_ptr()), style, x, y, width, height, None, None, Some(instance), None)
            .unwrap_or_default();
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let click = |id: i32| {
            let _ = PostMessageW(Some(item(hwnd, id)), BM_CLICK, WPARAM(0), LPARAM(0));
        };
        if let Some(texture) = options.number("--select-texture").filter(|&n| n < Texture::ALL.len() as u64) {
            click(ID_TEXTURE_FIRST + texture as i32);
        }
        if let Some(release) = options.number("--select-release").filter(|&n| n < Release::ALL.len() as u64) {
            click(ID_RELEASE_FIRST + release as i32);
        }
        if options.has("--toggle-shortcut") {
            click(ID_SHORTCUT);
        }
        if options.has("--run") {
            click(ID_INSTALL);
        }
        if let Some(delay) = options.number("--cancel-after-ms") {
            SetTimer(Some(hwnd), TIMER_CANCEL, delay as u32, None);
        }
        if SCREENSHOT.with(|s| s.borrow().is_some()) {
            SetTimer(Some(hwnd), TIMER_SCREENSHOT, options.number("--screenshot-after-ms").unwrap_or(700) as u32, None);
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
            let root = GetAncestor(msg.hwnd, GA_ROOT);
            if !root.is_invalid() && IsDialogMessageW(root, &msg).as_bool() {
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
