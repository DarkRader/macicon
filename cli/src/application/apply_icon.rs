//! Application use case: Apply icon to macOS app bundle and flush Dock caches.

use std::path::Path;

use crate::error::MacIconError;
use crate::infra::macos::dock::restart_dock;
use crate::infra::macos::tools::touch_path;
use crate::infra::macos::workspace::{apply_icon_to_app_bundle, find_app_bundle};

/// Apply a .icns file directly to a target .app bundle without breaking code signatures.
pub fn apply_icon_to_app(
    app_path: &str,
    icns_path: &Path,
    should_restart_dock: bool,
) -> Result<(), MacIconError> {
    let target_app = find_app_bundle(app_path)?;

    println!("Applying icon to '{}'...", target_app.display());
    apply_icon_to_app_bundle(&target_app, icns_path)?;

    let _ = touch_path(&target_app);
    println!("Successfully applied icon to {}.", target_app.display());

    if should_restart_dock {
        restart_dock()?;
        println!("Restarted Dock.");
    }

    Ok(())
}
