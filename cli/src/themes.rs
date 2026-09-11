//! Theme management, presets, and color parsing for macicon.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct ThemePreset {
    pub bg_top: &'static str,
    pub bg_bottom: &'static str,
    pub border: &'static str,
    pub symbol: &'static str,
}

pub const THEME_PRESETS: &[(&str, ThemePreset)] = &[
    (
        "light",
        ThemePreset {
            bg_top: "#FFFFFF",
            bg_bottom: "#EBECEF",
            border: "#D8D9DC",
            symbol: "#202022",
        },
    ),
    (
        "dark",
        ThemePreset {
            bg_top: "#161618",
            bg_bottom: "#0D0D0E",
            border: "#28282C",
            symbol: "#FFFFFF",
        },
    ),
    (
        "white",
        ThemePreset {
            bg_top: "#FFFFFF",
            bg_bottom: "#FFFFFF",
            border: "#E5E5EA",
            symbol: "#1C1C1E",
        },
    ),
    (
        "black",
        ThemePreset {
            bg_top: "#161618",
            bg_bottom: "#0D0D0E",
            border: "#28282C",
            symbol: "#FFFFFF",
        },
    ),
    (
        "slate",
        ThemePreset {
            bg_top: "#F1F5F9",
            bg_bottom: "#E2E8F0",
            border: "#CBD5E1",
            symbol: "#0F172A",
        },
    ),
    (
        "nord",
        ThemePreset {
            bg_top: "#2E3440",
            bg_bottom: "#242933",
            border: "#3B4252",
            symbol: "#ECEFF4",
        },
    ),
    (
        "catppuccin",
        ThemePreset {
            bg_top: "#1E1E2E",
            bg_bottom: "#181825",
            border: "#313244",
            symbol: "#CDD6F4",
        },
    ),
    (
        "dracula",
        ThemePreset {
            bg_top: "#282A36",
            bg_bottom: "#1E1F29",
            border: "#44475A",
            symbol: "#F8F8F2",
        },
    ),
    (
        "rose-pine",
        ThemePreset {
            bg_top: "#191724",
            bg_bottom: "#12101B",
            border: "#26233A",
            symbol: "#E0DEF4",
        },
    ),
    (
        "solarized-dark",
        ThemePreset {
            bg_top: "#073642",
            bg_bottom: "#002B36",
            border: "#586E75",
            symbol: "#FDF6E3",
        },
    ),
    (
        "solarized-light",
        ThemePreset {
            bg_top: "#FDF6E3",
            bg_bottom: "#EEE8D5",
            border: "#93A1A1",
            symbol: "#657B83",
        },
    ),
    (
        "apple",
        ThemePreset {
            bg_top: "#FFFFFF",
            bg_bottom: "#EBECEF",
            border: "#D8D9DC",
            symbol: "#00C8FF,#0072FE",
        },
    ),
];

pub const COLOR_SHORTCUTS: &[(&str, (&str, &str))] = &[
    ("white", ("#FFFFFF", "#EBECEF")),
    ("pure-white", ("#FFFFFF", "#FFFFFF")),
    ("light", ("#FFFFFF", "#EBECEF")),
    ("dark", ("#2C2D31", "#1C1D20")),
    ("black", ("#161618", "#0D0D0E")),
    ("slate", ("#F1F5F9", "#E2E8F0")),
    ("gray", ("#F2F2F7", "#E5E5EA")),
    ("nord", ("#2E3440", "#242933")),
    ("catppuccin", ("#1E1E2E", "#181825")),
    ("dracula", ("#282A36", "#1E1F29")),
    ("rose-pine", ("#191724", "#12101B")),
];

pub const SYMBOL_SHORTCUTS: &[(&str, &str)] = &[
    ("black", "#202022"),
    ("dark", "#202022"),
    ("charcoal", "#202022"),
    ("white", "#FFFFFF"),
    ("light", "#FFFFFF"),
    ("blue", "#007AFF"),
    ("blue-light", "#0097FF"),
    ("finder", "#00C8FF,#0072FE"),
    ("apple", "#00C8FF,#0072FE"),
    ("gray", "#8E8E93"),
    ("red", "#FF3B30"),
    ("green", "#34C759"),
    ("purple", "#AF52DE"),
    ("cyan", "#00FFFF"),
];

const HEX_SHORT_LEN: usize = 3;
const HEX_FULL_LEN: usize = 6;
const DARK_LUMINANCE_THRESHOLD: f64 = 100.0;
const COLOR_PAIR_LEN: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    pub bg_top: String,
    pub bg_bottom: String,
    pub border: String,
    pub symbol_color: String,
    #[serde(default = "default_shadow")]
    pub shadow: bool,
    #[serde(default = "default_scale")]
    pub scale: f64,
}

fn default_shadow() -> bool {
    true
}

fn default_scale() -> f64 {
    1.25
}

#[derive(Debug, Clone, PartialEq)]
pub struct IconStyle {
    pub bg_top: String,
    pub bg_bottom: String,
    pub border: String,
    pub symbol_color: String,
}

/// Return true if the hex color has a dark perceived luminance.
pub fn is_color_dark(hex_color: &str) -> bool {
    let hex = hex_color.trim_start_matches('#');
    let normalized = if hex.len() == HEX_SHORT_LEN {
        let mut s = String::with_capacity(6);
        for c in hex.chars() {
            s.push(c);
            s.push(c);
        }
        s
    } else if hex.len() == HEX_FULL_LEN {
        hex.to_string()
    } else {
        return false;
    };

    let r = u8::from_str_radix(&normalized[0..2], 16).ok();
    let g = u8::from_str_radix(&normalized[2..4], 16).ok();
    let b = u8::from_str_radix(&normalized[4..6], 16).ok();

    match (r, g, b) {
        (Some(r), Some(g), Some(b)) => {
            let luminance = 0.299 * (r as f64) + 0.587 * (g as f64) + 0.114 * (b as f64);
            luminance < DARK_LUMINANCE_THRESHOLD
        }
        _ => false,
    }
}

/// Resolve background gradient, border color, and symbol color.
pub fn compute_styling(
    theme: &str,
    bg: Option<&str>,
    color: Option<&str>,
    border_color: Option<&str>,
) -> IconStyle {
    let preset = THEME_PRESETS
        .iter()
        .find(|(name, _)| *name == theme)
        .map(|(_, p)| *p)
        .unwrap_or(THEME_PRESETS[0].1);

    let mut bg_top = preset.bg_top.to_string();
    let mut bg_bottom = preset.bg_bottom.to_string();
    let mut border = preset.border.to_string();
    let mut symbol_color = preset.symbol.to_string();

    if let Some(bg_val) = bg {
        let raw_bg = bg_val.trim();
        let lowered = raw_bg.to_lowercase();
        if let Some((_, (top, bottom))) = COLOR_SHORTCUTS.iter().find(|(k, _)| *k == lowered) {
            bg_top = (*top).to_string();
            bg_bottom = (*bottom).to_string();
        } else {
            let parts: Vec<&str> = raw_bg.split(',').map(|s| s.trim()).collect();
            if parts.len() == 1 {
                bg_top = parts[0].to_string();
                bg_bottom = parts[0].to_string();
            } else if parts.len() >= COLOR_PAIR_LEN {
                bg_top = parts[0].to_string();
                bg_bottom = parts[1].to_string();
            }
        }
    }

    if let Some(bc) = border_color {
        border = bc.to_string();
    } else if is_color_dark(&bg_top) || is_color_dark(&bg_bottom) {
        border = if bg_top.eq_ignore_ascii_case("#161618")
            || bg_top.eq_ignore_ascii_case("#000000")
            || bg_top.eq_ignore_ascii_case("#0d0d0e")
            || theme == "black"
        {
            "#28282C".to_string()
        } else {
            "#3A3B40".to_string()
        };
    } else if bg.is_some() && bg_top == bg_bottom {
        border = bg_top.clone();
    }

    if let Some(c) = color {
        let raw_color = c.trim();
        let lowered = raw_color.to_lowercase();
        if let Some((_, shortcut)) = SYMBOL_SHORTCUTS.iter().find(|(k, _)| *k == lowered) {
            symbol_color = (*shortcut).to_string();
        } else {
            symbol_color = raw_color.to_string();
        }
    }

    IconStyle {
        bg_top,
        bg_bottom,
        border,
        symbol_color,
    }
}

/// Resolve base directory for icons and themes manifest.
pub fn get_icons_base_dir(custom_path: Option<&str>) -> PathBuf {
    if let Some(cp) = custom_path {
        let path_str = if cp.starts_with('~') {
            if let Some(home) = std::env::var_os("HOME") {
                cp.replacen('~', home.to_string_lossy().as_ref(), 1)
            } else {
                cp.to_string()
            }
        } else {
            cp.to_string()
        };
        return PathBuf::from(path_str);
    }

    if Path::new("icons").is_dir() {
        return PathBuf::from("icons");
    }
    if Path::new("nix/icons").is_dir() {
        return PathBuf::from("nix/icons");
    }
    PathBuf::from("icons")
}

/// Load registered themes manifest or return standard defaults.
pub fn load_themes_manifest(
    manifest_path: &Path,
    icons_base_dir: &Path,
) -> HashMap<String, ThemeConfig> {
    let mut data: HashMap<String, ThemeConfig> = HashMap::new();
    if manifest_path.is_file()
        && let Ok(content) = fs::read_to_string(manifest_path)
        && let Ok(parsed) = serde_json::from_str::<HashMap<String, ThemeConfig>>(&content)
    {
        data = parsed;
    }

    if !data.contains_key("light") && icons_base_dir.join("light").is_dir() {
        data.insert(
            "light".to_string(),
            ThemeConfig {
                bg_top: "#FFFFFF".to_string(),
                bg_bottom: "#EBECEF".to_string(),
                border: "#D8D9DC".to_string(),
                symbol_color: "#202022".to_string(),
                shadow: true,
                scale: 1.25,
            },
        );
    }

    if !data.contains_key("dark") && icons_base_dir.join("dark").is_dir() {
        data.insert(
            "dark".to_string(),
            ThemeConfig {
                bg_top: "#161618".to_string(),
                bg_bottom: "#0D0D0E".to_string(),
                border: "#28282C".to_string(),
                symbol_color: "#FFFFFF".to_string(),
                shadow: false,
                scale: 1.25,
            },
        );
    }

    data
}

/// Save themes manifest to disk as formatted JSON.
pub fn save_themes_manifest(
    manifest_path: &Path,
    data: &HashMap<String, ThemeConfig>,
) -> io::Result<()> {
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json_str = serde_json::to_string_pretty(data)?;
    fs::write(manifest_path, format!("{}\n", json_str))?;
    Ok(())
}
