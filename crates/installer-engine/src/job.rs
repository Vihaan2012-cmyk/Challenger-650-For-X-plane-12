//! A job is a list of steps run one after another on a worker thread. It
//! reports what it is doing as events and stops between steps when cancelled.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::paths;
use crate::settings::{Release, Texture};
use crate::status::{self, Latest};

/// Shared between the window, which sets it, and the job, which reads it.
#[derive(Clone, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Finished(String),
    Cancelled,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Step { index: usize, total: usize, name: String },
    Log(String),
    Done(Outcome),
}

/// What a step asks for when it succeeds.
pub enum Flow {
    Continue,
    /// End the job here, successfully, with this message.
    Stop(String),
}

type Action = Box<dyn FnOnce(&mut dyn FnMut(String)) -> Result<Flow, String> + Send>;

pub struct Step {
    pub name: String,
    action: Action,
}

impl Step {
    /// `action` gets a function for writing log lines.
    pub fn new(name: impl Into<String>, action: impl FnOnce(&mut dyn FnMut(String)) -> Result<Flow, String> + Send + 'static) -> Self {
        Step { name: name.into(), action: Box::new(action) }
    }
}

/// Waits for `pause`, giving up early if the job is cancelled.
fn pause_unless_cancelled(pause: Duration, cancel: &Cancel) -> bool {
    let until = Instant::now() + pause;
    while Instant::now() < until {
        if cancel.is_cancelled() {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20).min(until - Instant::now()));
    }
    !cancel.is_cancelled()
}

/// Runs the steps in order. `pause` is a wait after each step, zero in normal
/// use; it exists so the window's behaviour can be watched and tested.
pub fn run(steps: Vec<Step>, cancel: &Cancel, pause: Duration, sink: &mut dyn FnMut(Event)) -> Outcome {
    let total = steps.len();
    let mut outcome = Outcome::Finished("Done.".into());
    for (index, step) in steps.into_iter().enumerate() {
        if cancel.is_cancelled() {
            outcome = Outcome::Cancelled;
            break;
        }
        sink(Event::Step { index, total, name: step.name });
        match (step.action)(&mut |line| sink(Event::Log(line))) {
            Ok(Flow::Continue) => {}
            Ok(Flow::Stop(message)) => {
                outcome = Outcome::Finished(message);
                break;
            }
            Err(reason) => {
                outcome = Outcome::Failed(reason);
                break;
            }
        }
        if !pause_unless_cancelled(pause, cancel) {
            outcome = Outcome::Cancelled;
            break;
        }
    }
    sink(Event::Done(outcome.clone()));
    outcome
}

/// Everything chosen in the window when Install or Build was pressed.
#[derive(Clone, Debug)]
pub struct Request {
    pub sim_dir: PathBuf,
    pub install_dir: PathBuf,
    pub texture: Texture,
    pub release: Release,
    pub optimise: bool,
    pub repo: String,
}

/// The checks made before anything is written. Installing itself is not
/// built yet, so the last step says so and stops.
pub fn preflight(request: Request) -> Vec<Step> {
    let mut steps = Vec::new();

    let first = request.clone();
    steps.push(Step::new("Checking the simulator folder", move |log| {
        if !paths::is_xplane_root(&first.sim_dir) {
            return Err(format!("{} is not an X-Plane 12 install.", first.sim_dir.display()));
        }
        log(format!("Simulator: {}", first.sim_dir.display()));
        log(format!("Install to: {}", first.install_dir.display()));
        log(format!(
            "Textures {}, release {}, optimise {}",
            first.texture.label(),
            first.release.label(),
            if first.optimise { "on" } else { "off" }
        ));
        Ok(Flow::Continue)
    }));

    let install_dir = request.install_dir.clone();
    steps.push(Step::new("Checking free disk space", move |log| {
        match free_space(&install_dir) {
            Some(bytes) => log(format!("Free space on that drive: {}", gigabytes(bytes))),
            None => log("Free space on that drive could not be read.".into()),
        }
        Ok(Flow::Continue)
    }));

    steps.push(Step::new("Checking that X-Plane is closed", |log| {
        if process_running("X-Plane.exe") {
            return Err("X-Plane is running. Close it and try again.".into());
        }
        log("X-Plane is not running.".into());
        Ok(Flow::Continue)
    }));

    if request.release == Release::Source {
        for (tool, name, link) in [("git", "Git", "https://git-scm.com"), ("cargo", "Rust", "https://rustup.rs")] {
            steps.push(Step::new(format!("Checking for {name}"), move |log| match tool_version(tool) {
                Some(version) => {
                    log(format!("Found {version}"));
                    Ok(Flow::Continue)
                }
                None => Err(format!("{name} is not installed. Source builds need it: {link}")),
            }));
        }
    } else {
        let (repo, release) = (request.repo.clone(), request.release);
        steps.push(Step::new("Checking for the latest release", move |log| match status::latest(&repo, release) {
            Latest::Version(version) => {
                log(format!("Latest release: {version}"));
                Ok(Flow::Continue)
            }
            Latest::NonePublished => Ok(Flow::Stop("No release has been published yet, so there is nothing to install.".into())),
            Latest::NoSource => Ok(Flow::Stop("No release source is set, so there is nothing to install.".into())),
            Latest::Unknown(reason) => Err(format!("The latest release could not be checked: {reason}.")),
            Latest::Checking => Ok(Flow::Continue),
        }));
    }

    let source = request.release == Release::Source;
    steps.push(Step::new("Finishing", move |_| {
        Ok(Flow::Stop(format!(
            "Checks passed. The install engine is not built yet, so nothing was {}.",
            if source { "built or installed" } else { "installed" }
        )))
    }));
    steps
}

pub fn gigabytes(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
}

/// The first line a tool prints for `--version`, or nothing if it will not run.
pub fn tool_version(tool: &str) -> Option<String> {
    let mut command = std::process::Command::new(tool);
    command.arg("--version");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command.output().ok().filter(|o| o.status.success())?;
    String::from_utf8_lossy(&output.stdout).lines().next().map(|line| line.trim().to_owned()).filter(|line| !line.is_empty())
}

/// Free bytes on the drive that holds `path`, which need not exist yet.
#[cfg(windows)]
pub fn free_space(path: &Path) -> Option<u64> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let mut existing = path.to_path_buf();
    while !existing.is_dir() {
        if !existing.pop() {
            return None;
        }
    }
    let wide: Vec<u16> = existing.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
    let mut free = 0u64;
    unsafe { GetDiskFreeSpaceExW(PCWSTR(wide.as_ptr()), Some(&mut free), None, None).ok()? };
    Some(free)
}

#[cfg(not(windows))]
pub fn free_space(_path: &Path) -> Option<u64> {
    None
}

#[cfg(windows)]
pub fn process_running(name: &str) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};

    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return false };
        let mut entry = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut found = false;
        let mut more = Process32FirstW(snapshot, &mut entry).is_ok();
        while more {
            let length = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..length]).eq_ignore_ascii_case(name) {
                found = true;
                break;
            }
            more = Process32NextW(snapshot, &mut entry).is_ok();
        }
        let _ = CloseHandle(snapshot);
        found
    }
}

#[cfg(not(windows))]
pub fn process_running(_name: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn collect(steps: Vec<Step>, cancel: &Cancel, pause: Duration) -> (Outcome, Vec<Event>) {
        let mut events = Vec::new();
        let outcome = run(steps, cancel, pause, &mut |event| events.push(event));
        (outcome, events)
    }

    fn noting(name: &'static str, ran: &Arc<Mutex<Vec<&'static str>>>, result: fn() -> Result<Flow, String>) -> Step {
        let ran = ran.clone();
        Step::new(name, move |log| {
            ran.lock().unwrap().push(name);
            log(format!("{name} ran"));
            result()
        })
    }

    #[test]
    fn steps_run_in_order_and_report_themselves() {
        let ran = Arc::new(Mutex::new(Vec::new()));
        let steps = vec![noting("one", &ran, || Ok(Flow::Continue)), noting("two", &ran, || Ok(Flow::Continue))];
        let (outcome, events) = collect(steps, &Cancel::new(), Duration::ZERO);
        assert_eq!(outcome, Outcome::Finished("Done.".into()));
        assert_eq!(*ran.lock().unwrap(), vec!["one", "two"]);
        assert_eq!(
            events,
            vec![
                Event::Step { index: 0, total: 2, name: "one".into() },
                Event::Log("one ran".into()),
                Event::Step { index: 1, total: 2, name: "two".into() },
                Event::Log("two ran".into()),
                Event::Done(Outcome::Finished("Done.".into())),
            ]
        );
    }

    #[test]
    fn a_step_can_end_the_job_early_with_a_message() {
        let ran = Arc::new(Mutex::new(Vec::new()));
        let steps = vec![noting("one", &ran, || Ok(Flow::Stop("nothing to do".into()))), noting("two", &ran, || Ok(Flow::Continue))];
        let (outcome, events) = collect(steps, &Cancel::new(), Duration::ZERO);
        assert_eq!(outcome, Outcome::Finished("nothing to do".into()));
        assert_eq!(*ran.lock().unwrap(), vec!["one"]);
        assert_eq!(events.last(), Some(&Event::Done(Outcome::Finished("nothing to do".into()))));
    }

    #[test]
    fn a_failing_step_stops_the_job() {
        let ran = Arc::new(Mutex::new(Vec::new()));
        let steps = vec![noting("one", &ran, || Err("disk full".into())), noting("two", &ran, || Ok(Flow::Continue))];
        let (outcome, _) = collect(steps, &Cancel::new(), Duration::ZERO);
        assert_eq!(outcome, Outcome::Failed("disk full".into()));
        assert_eq!(*ran.lock().unwrap(), vec!["one"]);
    }

    #[test]
    fn a_job_cancelled_before_it_starts_runs_nothing() {
        let ran = Arc::new(Mutex::new(Vec::new()));
        let cancel = Cancel::new();
        cancel.cancel();
        let (outcome, events) = collect(vec![noting("one", &ran, || Ok(Flow::Continue))], &cancel, Duration::ZERO);
        assert_eq!(outcome, Outcome::Cancelled);
        assert!(ran.lock().unwrap().is_empty());
        assert_eq!(events, vec![Event::Done(Outcome::Cancelled)]);
    }

    #[test]
    fn cancelling_during_a_step_stops_before_the_next_one() {
        let ran = Arc::new(Mutex::new(Vec::new()));
        let cancel = Cancel::new();
        let (during, noted) = (cancel.clone(), ran.clone());
        let first = Step::new("one", move |_| {
            noted.lock().unwrap().push("one");
            during.cancel();
            Ok(Flow::Continue)
        });
        let (outcome, _) = collect(vec![first, noting("two", &ran, || Ok(Flow::Continue))], &cancel, Duration::ZERO);
        assert_eq!(outcome, Outcome::Cancelled);
        assert_eq!(*ran.lock().unwrap(), vec!["one"]);
    }

    #[test]
    fn cancelling_cuts_a_long_pause_short() {
        let cancel = Cancel::new();
        let later = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(80));
            later.cancel();
        });
        let started = Instant::now();
        let (outcome, _) = collect(vec![Step::new("one", |_| Ok(Flow::Continue))], &cancel, Duration::from_secs(30));
        assert_eq!(outcome, Outcome::Cancelled);
        assert!(started.elapsed() < Duration::from_secs(5), "waited {:?}", started.elapsed());
    }

    #[test]
    fn preflight_refuses_a_folder_that_is_not_xplane() {
        let request = Request {
            sim_dir: crate::test_dir("job-not-xplane"),
            install_dir: PathBuf::from("C:\\nowhere"),
            texture: Texture::K4,
            release: Release::Stable,
            optimise: true,
            repo: String::new(),
        };
        let (outcome, events) = collect(preflight(request), &Cancel::new(), Duration::ZERO);
        assert!(matches!(&outcome, Outcome::Failed(reason) if reason.contains("not an X-Plane 12 install")), "{outcome:?}");
        assert_eq!(events.iter().filter(|e| matches!(e, Event::Step { .. })).count(), 1);
    }

    #[test]
    fn source_builds_check_for_git_and_rust_instead_of_a_release() {
        let request = |release| Request {
            sim_dir: PathBuf::new(),
            install_dir: PathBuf::new(),
            texture: Texture::K4,
            release,
            optimise: true,
            repo: String::new(),
        };
        let names = |release| preflight(request(release)).into_iter().map(|s| s.name).collect::<Vec<_>>();
        let source = names(Release::Source);
        assert!(source.contains(&"Checking for Git".to_owned()) && source.contains(&"Checking for Rust".to_owned()));
        assert!(!source.contains(&"Checking for the latest release".to_owned()));
        assert!(names(Release::Stable).contains(&"Checking for the latest release".to_owned()));
    }

    #[test]
    fn sizes_read_in_gigabytes() {
        assert_eq!(gigabytes(1_073_741_824), "1.0 GB");
        assert_eq!(gigabytes(536_870_912), "0.5 GB");
    }

    #[test]
    fn a_tool_that_does_not_exist_has_no_version() {
        assert_eq!(tool_version("cl650-no-such-tool-exists"), None);
    }
}
