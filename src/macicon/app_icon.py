"""macOS application bundle icon application via Cocoa NSWorkspace."""

import os
import subprocess
import sys


def apply_icon_to_app(app_path: str, icns_path: str, restart_dock: bool = True):
    """Applies .icns directly to a target .app bundle without breaking code signatures."""
    app_path = os.path.abspath(os.path.expanduser(app_path))
    icns_path = os.path.abspath(os.path.expanduser(icns_path))

    if not os.path.exists(app_path):
        candidate = f"/Applications/{app_path}.app" if not app_path.endswith(".app") else f"/Applications/{app_path}"
        if os.path.exists(candidate):
            app_path = candidate
        else:
            sys.exit(f"Error: Target app '{app_path}' does not exist.")

    if not os.path.exists(icns_path):
        sys.exit(f"Error: Icon file '{icns_path}' does not exist.")

    print(f"Applying icon to '{app_path}'...")
    applescript = f"""
use framework "Cocoa"
set iconPath to "{icns_path}"
set destPath to "{app_path}"
set imageData to (current application's NSImage's alloc()'s initWithContentsOfFile:iconPath)
return (current application's NSWorkspace's sharedWorkspace()'s setIcon:imageData forFile:destPath options:2)
"""
    result = subprocess.run(["osascript", "-e", applescript], capture_output=True, text=True, check=True)
    if "true" in result.stdout:
        subprocess.run(["touch", app_path], check=False)
        print(f"Successfully applied icon to {app_path}.")
        if restart_dock:
            subprocess.run(["killall", "Dock"], check=False)
            print("Restarted Dock.")
    else:
        print(
            f"\nWarning: Could not set icon directly. If {app_path} is owned by root, run with sudo.",
            file=sys.stderr
        )
