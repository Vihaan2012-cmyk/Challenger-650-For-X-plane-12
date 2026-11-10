//! Which version is installed and which is the newest available.

use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::settings::Release;

/// The installer writes this into the aircraft folder to record what it put there.
pub const STATE_FILE: &str = "cl650-install.json";

#[derive(Deserialize)]
struct Installed {
    version: String,
}

pub fn installed_version(install_dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(install_dir.join(STATE_FILE)).ok()?;
    let installed: Installed = serde_json::from_str(&text).ok()?;
    Some(installed.version).filter(|v| !v.trim().is_empty())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Latest {
    /// No repository has been set, so there is nowhere to look.
    NoSource,
    Checking,
    Version(String),
    /// The repository answered, and has nothing of the kind asked for.
    NonePublished,
    /// The check itself failed; the text says why.
    Unknown(String),
}

impl Latest {
    pub fn label(&self) -> String {
        match self {
            Latest::NoSource => "no release source yet".into(),
            Latest::Checking => "checking...".into(),
            Latest::Version(version) => version.clone(),
            Latest::NonePublished => "none published".into(),
            Latest::Unknown(_) => "could not check".into(),
        }
    }
}

#[derive(Deserialize)]
struct ReleaseEntry {
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
}

/// The newest release in GitHub's list, which comes newest first. Stable
/// skips pre-releases; test builds take whatever is newest.
pub fn newest_release(json: &str, include_prereleases: bool) -> Option<String> {
    let entries: Vec<ReleaseEntry> = serde_json::from_str(json).ok()?;
    entries.into_iter().find(|e| !e.draft && (include_prereleases || !e.prerelease)).map(|e| e.tag_name)
}

#[derive(Deserialize)]
struct CommitEntry {
    sha: String,
}

/// The newest commit, shortened the way Git shows it.
pub fn newest_commit(json: &str) -> Option<String> {
    let entries: Vec<CommitEntry> = serde_json::from_str(json).ok()?;
    entries.first().map(|e| e.sha.chars().take(7).collect())
}

fn fetch(url: &str) -> Result<String, Latest> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).build();
    match agent.get(url).set("User-Agent", "cl650-installer").set("Accept", "application/vnd.github+json").call() {
        Ok(response) => response.into_string().map_err(|e| Latest::Unknown(e.to_string())),
        Err(ureq::Error::Status(404, _)) => Err(Latest::NonePublished),
        Err(ureq::Error::Status(code, _)) => Err(Latest::Unknown(format!("GitHub answered {code}"))),
        Err(_) => Err(Latest::Unknown("no connection to GitHub".into())),
    }
}

/// Asks GitHub. Blocks for up to ten seconds, so call it off the window's thread.
pub fn latest(repo: &str, release: Release) -> Latest {
    let repo = repo.trim();
    if repo.is_empty() {
        return Latest::NoSource;
    }
    let (url, pick): (String, fn(&str) -> Option<String>) = match release {
        Release::Stable => (format!("https://api.github.com/repos/{repo}/releases?per_page=30"), |json| newest_release(json, false)),
        Release::Test => (format!("https://api.github.com/repos/{repo}/releases?per_page=30"), |json| newest_release(json, true)),
        Release::Source => (format!("https://api.github.com/repos/{repo}/commits?per_page=1"), newest_commit),
    };
    match fetch(&url) {
        Ok(json) => pick(&json).map(Latest::Version).unwrap_or(Latest::NonePublished),
        Err(outcome) => outcome,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASES: &str = r#"[
        { "tag_name": "v0.3.0-draft", "prerelease": false, "draft": true },
        { "tag_name": "v0.2.0-beta1", "prerelease": true, "draft": false },
        { "tag_name": "v0.1.0", "prerelease": false, "draft": false }
    ]"#;

    #[test]
    fn stable_skips_drafts_and_prereleases() {
        assert_eq!(newest_release(RELEASES, false).as_deref(), Some("v0.1.0"));
    }

    #[test]
    fn test_builds_take_the_newest_that_is_not_a_draft() {
        assert_eq!(newest_release(RELEASES, true).as_deref(), Some("v0.2.0-beta1"));
    }

    #[test]
    fn an_empty_or_unexpected_answer_means_nothing_is_published() {
        assert_eq!(newest_release("[]", true), None);
        assert_eq!(newest_release(r#"{ "message": "Not Found" }"#, true), None);
        assert_eq!(newest_commit("[]"), None);
    }

    #[test]
    fn a_commit_is_shown_by_its_first_seven_characters() {
        assert_eq!(newest_commit(r#"[{ "sha": "8e6a0021c4f09b1d" }]"#).as_deref(), Some("8e6a002"));
    }

    #[test]
    fn no_repository_means_no_source_and_no_request() {
        assert_eq!(latest("  ", Release::Stable), Latest::NoSource);
    }

    #[test]
    fn the_installed_version_comes_from_the_state_file() {
        let dir = crate::test_dir("status-installed");
        assert_eq!(installed_version(&dir), None);
        std::fs::write(dir.join(STATE_FILE), r#"{ "version": "0.4.1", "files": [] }"#).unwrap();
        assert_eq!(installed_version(&dir).as_deref(), Some("0.4.1"));
        std::fs::write(dir.join(STATE_FILE), "not json").unwrap();
        assert_eq!(installed_version(&dir), None);
    }
}
