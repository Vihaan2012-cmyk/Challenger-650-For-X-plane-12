//! The installer's shortcut in the Start menu.

use std::path::{Path, PathBuf};

use windows::core::{Interface, PCWSTR};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Shell::{FOLDERID_Programs, IShellLinkW, SHGetKnownFolderPath, ShellLink, KF_FLAG_DEFAULT};

pub const FILE_NAME: &str = "Challenger 650 Installer.lnk";

fn wide(path: &Path) -> Vec<u16> {
    path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect()
}

/// The current user's Start menu Programs folder.
pub fn programs_dir() -> Option<PathBuf> {
    unsafe {
        let folder = SHGetKnownFolderPath(&FOLDERID_Programs, KF_FLAG_DEFAULT, None).ok()?;
        let path = folder.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(folder.0 as *const std::ffi::c_void));
        path
    }
}

pub fn exists(dir: &Path) -> bool {
    dir.join(FILE_NAME).is_file()
}

/// Writes a shortcut in `dir` that opens `target`. COM must be initialised on
/// the calling thread.
pub fn create(dir: &Path, target: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let link_path = dir.join(FILE_NAME);
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(|e| e.message())?;
        link.SetPath(PCWSTR(wide(target).as_ptr())).map_err(|e| e.message())?;
        if let Some(folder) = target.parent() {
            link.SetWorkingDirectory(PCWSTR(wide(folder).as_ptr())).map_err(|e| e.message())?;
        }
        let description: Vec<u16> = "Install, update or repair the Challenger 650".encode_utf16().chain(std::iter::once(0)).collect();
        link.SetDescription(PCWSTR(description.as_ptr())).map_err(|e| e.message())?;
        let file: IPersistFile = link.cast().map_err(|e| e.message())?;
        file.Save(PCWSTR(wide(&link_path).as_ptr()), true).map_err(|e| e.message())?;
    }
    Ok(link_path)
}

/// A shortcut that is already gone counts as removed.
pub fn remove(dir: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(dir.join(FILE_NAME)) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

/// Where a shortcut file points.
#[cfg(test)]
pub fn target_of(link_path: &Path) -> Option<PathBuf> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let file: IPersistFile = link.cast().ok()?;
        file.Load(PCWSTR(wide(link_path).as_ptr()), windows::Win32::System::Com::STGM_READ).ok()?;
        let mut buffer = [0u16; 1024];
        link.GetPath(&mut buffer, std::ptr::null_mut(), 0).ok()?;
        let length = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        Some(PathBuf::from(String::from_utf16_lossy(&buffer[..length]))).filter(|p| !p.as_os_str().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cl650-installer-shortcut-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_shortcut_points_at_its_target_and_can_be_removed() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }
        let dir = test_dir("round-trip");
        let target = std::env::current_exe().unwrap();
        assert!(!exists(&dir));

        let link = create(&dir, &target).expect("shortcut created");
        assert!(exists(&dir));
        assert_eq!(link, dir.join(FILE_NAME));
        assert_eq!(target_of(&link).map(|p| p.to_string_lossy().to_lowercase()), Some(target.to_string_lossy().to_lowercase()));

        remove(&dir).unwrap();
        assert!(!exists(&dir));
        remove(&dir).expect("removing a missing shortcut is not an error");
    }

    #[test]
    fn the_start_menu_folder_is_found() {
        let dir = programs_dir().expect("Programs folder");
        assert!(dir.is_dir(), "{}", dir.display());
    }
}
