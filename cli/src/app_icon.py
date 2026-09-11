"""macOS application bundle icon application via Cocoa NSWorkspace."""

import subprocess
import sys
from pathlib import Path


def apply_icon_to_app(app_path: str, icns_path: str, restart_dock: bool = True) -> None:
    """Applies .icns directly to a target .app bundle without breaking code signatures."""
    target_app = Path(app_path).expanduser().resolve()
    target_icns = Path(icns_path).expanduser().resolve()

    if not target_app.exists():
        candidate = (
            Path(f"/Applications/{app_path}.app")
            if not app_path.endswith(".app")
            else Path(f"/Applications/{app_path}")
        )
        if candidate.exists():
            target_app = candidate
        else:
            sys.exit(f"⚠️  Error: Target app '{app_path}' does not exist.")

    if not target_icns.exists():
        sys.exit(f"⚠️  Error: Icon file '{icns_path}' does not exist.")

    print(f"Applying icon to '{target_app}'...")
    applescript = f"""
use framework "Cocoa"
set iconPath to "{target_icns}"
set destPath to "{target_app}"
set imageData to (current application's NSImage's alloc()'s initWithContentsOfFile:iconPath)
return (current application's NSWorkspace's sharedWorkspace()'s setIcon:imageData forFile:destPath options:2)
"""
    result = subprocess.run(
        ["osascript", "-e", applescript], capture_output=True, text=True, check=True
    )
    if "true" in result.stdout:
        subprocess.run(["touch", str(target_app)], check=False)
        print(f"Successfully applied icon to {target_app}.")
        if restart_dock:
            subprocess.run(["killall", "Dock"], check=False)
            print("Restarted Dock.")
    else:
        print(
            f"\n⚠️  Warning: Could not set icon directly. If {target_app} is owned by root, run with sudo.",
            file=sys.stderr,
        )
        sys.exit(1)
