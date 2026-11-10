//! The installer's log: time-stamped lines, shown in the window and kept in a file.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const LOG_FILE: &str = "installer.log";

pub fn log_dir() -> Option<PathBuf> {
    crate::data_dir().map(|dir| dir.join("logs"))
}

/// Local time of day as `HH:MM:SS`.
#[cfg(windows)]
pub fn timestamp() -> String {
    let now = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    format!("{:02}:{:02}:{:02}", now.wHour, now.wMinute, now.wSecond)
}

#[cfg(not(windows))]
pub fn timestamp() -> String {
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) % 86_400;
    format!("{:02}:{:02}:{:02}", seconds / 3600, seconds / 60 % 60, seconds % 60)
}

/// Puts the time in front of a line.
pub fn stamped(text: &str) -> String {
    format!("{}  {text}", timestamp())
}

/// Appends to `installer.log`. A log that cannot be opened is skipped: the
/// installer must still work on a machine where that folder is read-only.
pub struct LogFile(Option<File>);

impl LogFile {
    pub fn open(dir: &Path) -> LogFile {
        let file = std::fs::create_dir_all(dir).ok().and_then(|_| std::fs::OpenOptions::new().create(true).append(true).open(dir.join(LOG_FILE)).ok());
        LogFile(file)
    }

    pub fn none() -> LogFile {
        LogFile(None)
    }

    pub fn line(&mut self, line: &str) {
        if let Some(file) = &mut self.0 {
            let _ = writeln!(file, "{line}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_timestamp_is_a_time_of_day() {
        let stamp = timestamp();
        let parts: Vec<u32> = stamp.split(':').map(|p| p.parse().expect("number")).collect();
        assert_eq!(stamp.len(), 8);
        assert!(parts.len() == 3 && parts[0] < 24 && parts[1] < 60 && parts[2] < 60, "{stamp}");
    }

    #[test]
    fn lines_are_appended_and_kept_across_openings() {
        let dir = crate::test_dir("log-append").join("logs");
        LogFile::open(&dir).line("first");
        LogFile::open(&dir).line("second");
        assert_eq!(std::fs::read_to_string(dir.join(LOG_FILE)).unwrap(), "first\nsecond\n");
    }

    #[test]
    fn a_log_that_cannot_be_opened_is_skipped_quietly() {
        LogFile::none().line("nowhere to write this");
    }
}
