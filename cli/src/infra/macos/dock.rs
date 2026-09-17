//! macOS Dock lifecycle and cache flush controls.

use std::process::Command;

use crate::error::InfraError;

/// Restart the macOS Dock process to reload cached application icons.
pub fn restart_dock() -> Result<(), InfraError> {
    let status =
        Command::new("killall")
            .arg("Dock")
            .status()
            .map_err(|e| InfraError::ToolExecution {
                tool: "killall Dock".to_string(),
                message: e.to_string(),
            })?;

    if !status.success() {
        return Err(InfraError::ToolExecution {
            tool: "killall Dock".to_string(),
            message: "Failed to restart Dock".to_string(),
        });
    }

    Ok(())
}
