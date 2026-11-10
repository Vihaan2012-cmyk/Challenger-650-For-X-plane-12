//! Finding the simulator and choosing where the aircraft goes.

use std::path::{Path, PathBuf};

pub const AIRCRAFT_FOLDER: &str = "Challenger 650";

/// X-Plane records every install as one line of `x-plane_install_12.txt`.
pub fn parse_install_list(text: &str) -> Vec<PathBuf> {
    text.lines().map(str::trim).filter(|l| !l.is_empty()).map(|l| normalise(l)).collect()
}

/// Backslashes throughout and no trailing separator, so a path reads the same
/// whether X-Plane, the folder picker or the user typed it.
pub fn normalise(path: &str) -> PathBuf {
    let mut s = path.trim().replace('/', "\\");
    while s.len() > 3 && s.ends_with('\\') {
        s.pop();
    }
    PathBuf::from(s)
}

pub fn is_xplane_root(path: &Path) -> bool {
    path.join("Aircraft").is_dir() && path.join("Resources").is_dir()
}

pub fn default_install_dir(sim: &Path) -> PathBuf {
    sim.join("Aircraft").join(AIRCRAFT_FOLDER)
}

/// The first recorded install that is still an X-Plane folder.
pub fn detect_xplane() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    let text = std::fs::read_to_string(Path::new(&local).join("x-plane_install_12.txt")).ok()?;
    parse_install_list(&text).into_iter().find(|p| is_xplane_root(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_list_skips_blank_lines_and_normalises() {
        let list = parse_install_list("D:\\Steam Games/steamapps/common/X-Plane 12/\r\n\r\n  C:\\X-Plane 12  \n");
        assert_eq!(list, vec![PathBuf::from("D:\\Steam Games\\steamapps\\common\\X-Plane 12"), PathBuf::from("C:\\X-Plane 12")]);
    }

    #[test]
    fn normalise_keeps_a_drive_root() {
        assert_eq!(normalise("D:/"), PathBuf::from("D:\\"));
        assert_eq!(normalise("D:\\Sims\\\\"), PathBuf::from("D:\\Sims"));
    }

    #[test]
    fn aircraft_goes_under_the_sims_aircraft_folder() {
        assert_eq!(default_install_dir(Path::new("D:\\X-Plane 12")), PathBuf::from("D:\\X-Plane 12\\Aircraft\\Challenger 650"));
    }

    #[test]
    fn a_folder_without_aircraft_is_not_xplane() {
        assert!(!is_xplane_root(Path::new("C:\\definitely\\not\\here")));
    }
}
