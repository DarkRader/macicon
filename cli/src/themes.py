"""Theme management, presets, and color parsing for macicon."""

import json
import os
import sys

THEME_PRESETS = {
    "light": {
        "bg_top": "#FFFFFF",
        "bg_bottom": "#EBECEF",
        "border": "#D8D9DC",
        "symbol": "#202022",
    },
    "dark": {
        "bg_top": "#161618",
        "bg_bottom": "#0D0D0E",
        "border": "#28282C",
        "symbol": "#FFFFFF",
    },
    "white": {
        "bg_top": "#FFFFFF",
        "bg_bottom": "#FFFFFF",
        "border": "#E5E5EA",
        "symbol": "#1C1C1E",
    },
    "black": {
        "bg_top": "#161618",
        "bg_bottom": "#0D0D0E",
        "border": "#28282C",
        "symbol": "#FFFFFF",
    },
    "slate": {
        "bg_top": "#F1F5F9",
        "bg_bottom": "#E2E8F0",
        "border": "#CBD5E1",
        "symbol": "#0F172A",
    },
    "nord": {
        "bg_top": "#2E3440",
        "bg_bottom": "#242933",
        "border": "#3B4252",
        "symbol": "#ECEFF4",
    },
    "catppuccin": {
        "bg_top": "#1E1E2E",
        "bg_bottom": "#181825",
        "border": "#313244",
        "symbol": "#CDD6F4",
    },
    "dracula": {
        "bg_top": "#282A36",
        "bg_bottom": "#1E1F29",
        "border": "#44475A",
        "symbol": "#F8F8F2",
    },
    "rose-pine": {
        "bg_top": "#191724",
        "bg_bottom": "#12101B",
        "border": "#26233A",
        "symbol": "#E0DEF4",
    },
    "solarized-dark": {
        "bg_top": "#073642",
        "bg_bottom": "#002B36",
        "border": "#586E75",
        "symbol": "#FDF6E3",
    },
    "solarized-light": {
        "bg_top": "#FDF6E3",
        "bg_bottom": "#EEE8D5",
        "border": "#93A1A1",
        "symbol": "#657B83",
    },
    "apple": {
        "bg_top": "#FFFFFF",
        "bg_bottom": "#EBECEF",
        "border": "#D8D9DC",
        "symbol": "#00C8FF,#0072FE",
    },
}

COLOR_SHORTCUTS = {
    "white": ("#FFFFFF", "#EBECEF"),
    "pure-white": ("#FFFFFF", "#FFFFFF"),
    "light": ("#FFFFFF", "#EBECEF"),
    "dark": ("#2C2D31", "#1C1D20"),
    "black": ("#161618", "#0D0D0E"),
    "slate": ("#F1F5F9", "#E2E8F0"),
    "gray": ("#F2F2F7", "#E5E5EA"),
    "nord": ("#2E3440", "#242933"),
    "catppuccin": ("#1E1E2E", "#181825"),
    "dracula": ("#282A36", "#1E1F29"),
    "rose-pine": ("#191724", "#12101B"),
}

SYMBOL_SHORTCUTS = {
    "black": "#202022",
    "dark": "#202022",
    "charcoal": "#202022",
    "white": "#FFFFFF",
    "light": "#FFFFFF",
    "blue": "#007AFF",
    "blue-light": "#0097FF",
    "finder": "#00C8FF,#0072FE",
    "apple": "#00C8FF,#0072FE",
    "gray": "#8E8E93",
    "red": "#FF3B30",
    "green": "#34C759",
    "purple": "#AF52DE",
    "cyan": "#00FFFF",
}


def is_color_dark(hex_color: str) -> bool:
    """Returns True if the hex color has a dark perceived luminance."""
    hex_color = hex_color.lstrip("#")
    if len(hex_color) == 3:
        hex_color = "".join(c * 2 for c in hex_color)
    if len(hex_color) != 6:
        return False
    try:
        r, g, b = int(hex_color[0:2], 16), int(hex_color[2:4], 16), int(hex_color[4:6], 16)
        luminance = 0.299 * r + 0.587 * g + 0.114 * b
        return luminance < 100
    except ValueError:
        return False


def compute_styling(
    theme: str,
    bg: str | None = None,
    color: str | None = None,
    border_color: str | None = None,
) -> tuple[str, str, str, str | tuple[str, str]]:
    """Resolves background gradient, border color, and symbol color."""
    preset = THEME_PRESETS.get(theme, THEME_PRESETS["light"])
    bg_top = preset["bg_top"]
    bg_bottom = preset["bg_bottom"]
    border = preset["border"]
    symbol_color = preset["symbol"]

    if bg:
        raw_bg = bg.strip()
        lowered = raw_bg.lower()
        if lowered in COLOR_SHORTCUTS:
            bg_top, bg_bottom = COLOR_SHORTCUTS[lowered]
        else:
            parts = [p.strip() for p in raw_bg.split(",")]
            if len(parts) == 1:
                bg_top = parts[0]
                bg_bottom = parts[0]
            elif len(parts) >= 2:
                bg_top = parts[0]
                bg_bottom = parts[1]

    if border_color:
        border = border_color
    else:
        if is_color_dark(bg_top) or is_color_dark(bg_bottom):
            border = "#28282C" if bg_top.lower() in ("#161618", "#000000", "#0d0d0e") or theme == "black" else "#3A3B40"
        elif bg and bg_top == bg_bottom:
            border = bg_top

    if color:
        raw_color = color.strip()
        lowered = raw_color.lower()
        symbol_color = SYMBOL_SHORTCUTS.get(lowered, raw_color)

    return bg_top, bg_bottom, border, symbol_color


def get_icons_base_dir(custom_path: str | None = None) -> str:
    """Resolves base directory for icons and themes manifest."""
    if custom_path:
        return os.path.abspath(os.path.expanduser(custom_path))

    # Check current directory
    if os.path.isdir("icons"):
        return os.path.abspath("icons")

    # Check nix/icons if run inside a nix/dotfiles workspace
    if os.path.isdir("nix/icons"):
        return os.path.abspath("nix/icons")

    # Default to ./icons
    return os.path.abspath("icons")


def load_themes_manifest(manifest_path: str, icons_base_dir: str) -> dict[str, dict[str, object]]:
    """Loads registered themes manifest or creates standard default definitions."""
    data: dict[str, dict[str, object]] = {}
    if os.path.isfile(manifest_path):
        try:
            with open(manifest_path, encoding="utf-8") as f:
                data = json.load(f)
        except (OSError, json.JSONDecodeError):
            data = {}

    if "light" not in data and os.path.isdir(os.path.join(icons_base_dir, "light")):
        data["light"] = {
            "bg_top": "#FFFFFF",
            "bg_bottom": "#EBECEF",
            "border": "#D8D9DC",
            "symbol_color": "#202022",
            "shadow": True,
            "scale": 1.25,
        }
    if "dark" not in data and os.path.isdir(os.path.join(icons_base_dir, "dark")):
        data["dark"] = {
            "bg_top": "#161618",
            "bg_bottom": "#0D0D0E",
            "border": "#28282C",
            "symbol_color": "#FFFFFF",
            "shadow": False,
            "scale": 1.25,
        }
    return data


def save_themes_manifest(manifest_path: str, data: dict[str, dict[str, object]]) -> None:
    """Saves themes manifest to disk as formatted JSON."""
    try:
        os.makedirs(os.path.dirname(os.path.abspath(manifest_path)), exist_ok=True)
        with open(manifest_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)
            f.write("\n")
    except OSError as e:
        print(f"Warning: Could not save themes manifest to {manifest_path}: {e}", file=sys.stderr)
