//! macOS application domain representations and candidate bundle path derivations.

use std::path::{Path, PathBuf};

/// Pure domain representation of a macOS application target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacApp {
    pub raw_name_or_path: String,
}

impl MacApp {
    pub fn new(name_or_path: impl Into<String>) -> Self {
        Self {
            raw_name_or_path: name_or_path.into(),
        }
    }

    /// Derive candidate file system paths where this application bundle might reside.
    pub fn candidate_paths(&self, home_dir: Option<&Path>) -> Vec<PathBuf> {
        let input = &self.raw_name_or_path;
        let mut candidates = Vec::new();

        // 1. Direct path or tilde-expanded path
        if input.starts_with('~') {
            if let Some(home) = home_dir {
                let expanded = input.replacen('~', home.to_string_lossy().as_ref(), 1);
                candidates.push(PathBuf::from(expanded));
            } else {
                candidates.push(PathBuf::from(input));
            }
        } else {
            candidates.push(PathBuf::from(input));
        }

        // 2. Canonical app name stripping .app suffix
        let name = input.strip_suffix(".app").unwrap_or(input);

        // Standard system and user application directories
        candidates.push(PathBuf::from(format!("/Applications/{}.app", name)));
        candidates.push(PathBuf::from(format!("/Applications/{}.app", input)));
        candidates.push(PathBuf::from(format!("/System/Applications/{}.app", name)));
        candidates.push(PathBuf::from(format!(
            "/System/Applications/Utilities/{}.app",
            name
        )));

        if let Some(home) = home_dir {
            candidates.push(PathBuf::from(format!(
                "{}/Applications/{}.app",
                home.to_string_lossy(),
                name
            )));
        }

        candidates
    }

    /// Derive safe stem name for temporary icon files.
    pub fn safe_stem(&self) -> String {
        Path::new(&self.raw_name_or_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_lowercase()
            .replace(' ', "-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_paths_derivation() {
        let app = MacApp::new("Slack");
        let home = Path::new("/Users/testuser");
        let candidates = app.candidate_paths(Some(home));

        assert!(candidates.contains(&PathBuf::from("/Applications/Slack.app")));
        assert!(candidates.contains(&PathBuf::from("/System/Applications/Slack.app")));
        assert!(candidates.contains(&PathBuf::from("/Users/testuser/Applications/Slack.app")));
    }

    #[test]
    fn test_tilde_expansion_candidate() {
        let app = MacApp::new("~/Applications/Custom.app");
        let home = Path::new("/Users/testuser");
        let candidates = app.candidate_paths(Some(home));

        assert_eq!(
            candidates[0],
            PathBuf::from("/Users/testuser/Applications/Custom.app")
        );
    }

    #[test]
    fn test_safe_stem() {
        assert_eq!(
            MacApp::new("Google Chrome.app").safe_stem(),
            "google-chrome"
        );
        assert_eq!(MacApp::new("Slack").safe_stem(), "slack");
    }
}
