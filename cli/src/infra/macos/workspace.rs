//! Cocoa NSWorkspace integration via AppleScript for setting application icons.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::domain::app::MacApp;
use crate::error::InfraError;

/// Find an existing .app bundle path on the local filesystem from an app name or path.
pub fn find_app_bundle(app_path_or_name: &str) -> Result<PathBuf, InfraError> {
    let app = MacApp::new(app_path_or_name);
    let home_dir_buf = std::env::var_os("HOME").map(PathBuf::from);
    let candidates = app.candidate_paths(home_dir_buf.as_deref());

    for cand in candidates {
        if cand.exists() {
            return Ok(cand);
        }
    }

    Err(InfraError::AppBundleNotFound(app_path_or_name.to_string()))
}

/// Apply a .icns file to an application bundle using NSWorkspace via Cocoa AppleScript.
pub fn apply_icon_to_app_bundle(target_app: &Path, icns_path: &Path) -> Result<(), InfraError> {
    if !target_app.exists() {
        return Err(InfraError::AppBundleNotFound(
            target_app.to_string_lossy().to_string(),
        ));
    }

    if !icns_path.exists() {
        return Err(InfraError::IconFileNotFound(
            icns_path.to_string_lossy().to_string(),
        ));
    }

    let applescript = format!(
        r#"use framework "Cocoa"
set iconPath to "{}"
set destPath to "{}"
set imageData to (current application's NSImage's alloc()'s initWithContentsOfFile:iconPath)
return (current application's NSWorkspace's sharedWorkspace()'s setIcon:imageData forFile:destPath options:2)
"#,
        icns_path.display(),
        target_app.display()
    );

    let output = Command::new("osascript")
        .args(["-e", &applescript])
        .output()
        .map_err(|e| InfraError::AppleScriptFailed(e.to_string()))?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    if stdout_str.contains("true") {
        Ok(())
    } else {
        Err(InfraError::AppleScriptFailed(format!(
            "Could not set icon directly on '{}'. If owned by root, try running with sudo.",
            target_app.display()
        )))
    }
}
