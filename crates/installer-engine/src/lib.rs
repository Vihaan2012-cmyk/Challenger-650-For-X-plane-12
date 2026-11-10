//! What the Challenger 650 installer does, kept apart from the window so it
//! can be tested without one.

pub mod job;
pub mod log;
pub mod paths;
pub mod settings;
pub mod status;

/// The folder under `%APPDATA%` that holds the installer's settings and logs.
pub fn data_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA").map(|roaming| std::path::Path::new(&roaming).join("CL650 Installer"))
}

#[cfg(test)]
pub(crate) fn test_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cl650-installer-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("test folder");
    dir
}
