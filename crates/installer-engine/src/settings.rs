//! The choices made in the window, kept between runs.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The GitHub repository releases are published from.
pub const DEFAULT_REPO: &str = "Vihaan2012-cmyk/Challenger-650-For-X-plane-12";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Texture {
    #[serde(rename = "1k")]
    K1,
    #[serde(rename = "2k")]
    K2,
    #[default]
    #[serde(rename = "4k")]
    K4,
    #[serde(rename = "8k")]
    K8,
}

impl Texture {
    pub const ALL: [Texture; 4] = [Texture::K1, Texture::K2, Texture::K4, Texture::K8];

    pub fn label(self) -> &'static str {
        ["1K", "2K", "4K", "8K"][self.index()]
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or_default()
    }
}

/// Where the aircraft comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Release {
    #[default]
    Stable,
    Test,
    Source,
}

impl Release {
    pub const ALL: [Release; 3] = [Release::Stable, Release::Test, Release::Source];

    pub fn label(self) -> &'static str {
        ["Stable", "Test builds", "Source"][self.index()]
    }

    /// One line explaining the choice, shown under the progress bar.
    pub fn note(self) -> &'static str {
        ["Ready", "Test builds are newer and less tested.", "Source compiles the aircraft on this PC. It needs Git and Rust."][self.index()]
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&r| r == self).unwrap_or(0)
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub sim_dir: String,
    pub install_dir: String,
    /// The user picked the install folder, so a new sim folder must not move it.
    pub install_dir_chosen: bool,
    pub texture: Texture,
    pub release: Release,
    pub optimise: bool,
    /// `owner/name` of the GitHub repository releases come from.
    pub repo: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sim_dir: String::new(),
            install_dir: String::new(),
            install_dir_chosen: false,
            texture: Texture::default(),
            release: Release::default(),
            optimise: true,
            repo: DEFAULT_REPO.to_owned(),
        }
    }
}

impl Settings {
    pub fn default_path() -> Option<PathBuf> {
        crate::data_dir().map(|dir| dir.join("settings.json"))
    }

    /// A missing or unreadable file gives the defaults: remembered choices
    /// are a convenience and must never stop the installer opening.
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
    }

    /// Written beside the target first, so a crash never leaves half a file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let partial = path.with_extension("json.part");
        std::fs::write(&partial, serde_json::to_string_pretty(self).map_err(std::io::Error::other)?)?;
        std::fs::rename(&partial, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices_survive_a_save_and_load() {
        let path = crate::test_dir("settings-round-trip").join("nested").join("settings.json");
        let chosen = Settings {
            sim_dir: "D:\\X-Plane 12".into(),
            install_dir: "D:\\Planes\\CL650".into(),
            install_dir_chosen: true,
            texture: Texture::K8,
            release: Release::Source,
            optimise: false,
            repo: "someone/cl650".into(),
        };
        chosen.save(&path).unwrap();
        assert_eq!(Settings::load(&path), chosen);
        assert!(!path.with_extension("json.part").exists());
    }

    #[test]
    fn a_missing_file_gives_the_defaults() {
        let loaded = Settings::load(&crate::test_dir("settings-missing").join("settings.json"));
        assert_eq!(loaded, Settings::default());
        assert!(loaded.optimise);
        assert_eq!(loaded.texture, Texture::K4);
        assert_eq!(loaded.release, Release::Stable);
    }

    #[test]
    fn a_damaged_file_gives_the_defaults() {
        let path = crate::test_dir("settings-damaged").join("settings.json");
        std::fs::write(&path, "{ this is not json").unwrap();
        assert_eq!(Settings::load(&path), Settings::default());
    }

    #[test]
    fn a_file_from_an_older_version_keeps_what_it_has() {
        let path = crate::test_dir("settings-older").join("settings.json");
        std::fs::write(&path, r#"{ "texture": "2k", "release": "test", "gone_in_this_version": 1 }"#).unwrap();
        let loaded = Settings::load(&path);
        assert_eq!((loaded.texture, loaded.release), (Texture::K2, Release::Test));
        assert!(loaded.optimise);
    }

    #[test]
    fn indexes_match_the_order_of_the_radio_buttons() {
        for (index, texture) in Texture::ALL.iter().enumerate() {
            assert_eq!((texture.index(), Texture::from_index(index)), (index, *texture));
        }
        for (index, release) in Release::ALL.iter().enumerate() {
            assert_eq!((release.index(), Release::from_index(index)), (index, *release));
        }
        assert_eq!(Texture::from_index(99), Texture::K4);
    }
}
