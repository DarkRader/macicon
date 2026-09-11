//! macOS application bundle icon application via Cocoa NSWorkspace.

use std::path::{Path, PathBuf};
use std::process::Command;

fn expand_tilde(path_str: &str) -> PathBuf {
    if path_str.starts_with('~') {
        if let Some(home) = std::env::var_os("HOME") {
            let expanded = path_str.replacen('~', home.to_string_lossy().as_ref(), 1);
            return PathBuf::from(expanded);
        }
    }
    PathBuf::from(path_str)
}

/// Find matching .app bundle path from name or path.
pub fn find_app_bundle(app_path_or_name: &str) -> Option<PathBuf> {
    let p = expand_tilde(app_path_or_name);
    if p.exists() {
        return Some(p);
    }

    let name = app_path_or_name
        .strip_suffix(".app")
        .unwrap_or(app_path_or_name);
    let mut candidates = vec![
        format!("/Applications/{}.app", name),
        format!("/Applications/{}.app", app_path_or_name),
        format!("/System/Applications/{}.app", name),
        format!("/System/Applications/Utilities/{}.app", name),
    ];

    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(format!(
            "{}/Applications/{}.app",
            home.to_string_lossy(),
            name
        ));
    }

    for cand in candidates {
        let path = PathBuf::from(cand);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Apply .icns directly to a target .app bundle without breaking code signatures.
pub fn apply_icon_to_app(
    app_path: &str,
    icns_path: &Path,
    restart_dock: bool,
) -> Result<(), String> {
    let target_app = find_app_bundle(app_path)
        .ok_or_else(|| format!("Target app '{}' does not exist.", app_path))?;

    if !icns_path.exists() {
        return Err(format!(
            "Icon file '{}' does not exist.",
            icns_path.display()
        ));
    }

    println!("Applying icon to '{}'...", target_app.display());

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
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    if stdout_str.contains("true") {
        let _ = Command::new("touch")
            .arg(target_app.to_str().unwrap())
            .status();
        println!("Successfully applied icon to {}.", target_app.display());

        if restart_dock {
            let _ = Command::new("killall").arg("Dock").status();
            println!("Restarted Dock.");
        }
        Ok(())
    } else {
        Err(format!(
            "Could not set icon directly. If {} is owned by root, run with sudo.",
            target_app.display()
        ))
    }
}
