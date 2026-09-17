//! Theme domain models, presets, and styling calculation for macicon.

use serde::{Deserialize, Serialize};

use super::color::{is_color_dark, resolve_background_colors, resolve_symbol_color};

/// Preset style values for a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePreset {
    pub bg_top: &'static str,
    pub bg_bottom: &'static str,
    pub border: &'static str,
    pub symbol: &'static str,
}

/// Registered built-in theme presets.
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

/// Serialized configuration for a registered theme in themes.json.
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

/// Resolved icon styling colors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconStyle {
    pub bg_top: String,
    pub bg_bottom: String,
    pub border: String,
    pub symbol_color: String,
}

/// Find a built-in theme preset by name.
pub fn find_preset(theme: &str) -> Option<ThemePreset> {
    THEME_PRESETS
        .iter()
        .find(|(name, _)| *name == theme)
        .map(|(_, p)| *p)
}

/// Resolve background gradient, border color, and symbol color based on presets and CLI overrides.
pub fn compute_styling(
    theme: &str,
    bg: Option<&str>,
    color: Option<&str>,
    border_color: Option<&str>,
) -> IconStyle {
    let preset = find_preset(theme).unwrap_or(THEME_PRESETS[0].1);

    let mut bg_top = preset.bg_top.to_string();
    let mut bg_bottom = preset.bg_bottom.to_string();
    let mut border = preset.border.to_string();
    let mut symbol_color = preset.symbol.to_string();

    if let Some(bg_val) = bg {
        let (resolved_top, resolved_bottom) = resolve_background_colors(bg_val);
        bg_top = resolved_top;
        bg_bottom = resolved_bottom;
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
        symbol_color = resolve_symbol_color(c);
    }

    IconStyle {
        bg_top,
        bg_bottom,
        border,
        symbol_color,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_presets() {
        let preset_names: Vec<&str> = THEME_PRESETS.iter().map(|(k, _)| *k).collect();
        assert!(preset_names.contains(&"light"));
        assert!(preset_names.contains(&"dark"));
        assert!(preset_names.contains(&"nord"));
        assert!(preset_names.contains(&"apple"));
    }

    #[test]
    fn test_compute_styling() {
        let style = compute_styling("light", Some("white"), Some("black"), None);
        assert_eq!(style.bg_top, "#FFFFFF");
        assert_eq!(style.bg_bottom, "#EBECEF");
        assert_eq!(style.border, "#D8D9DC");
        assert_eq!(style.symbol_color, "#202022");
    }

    #[test]
    fn test_compute_styling_custom_gradient() {
        let style = compute_styling(
            "light",
            Some("#111111,#222222"),
            Some("#FF0000"),
            Some("#333333"),
        );
        assert_eq!(style.bg_top, "#111111");
        assert_eq!(style.bg_bottom, "#222222");
        assert_eq!(style.border, "#333333");
        assert_eq!(style.symbol_color, "#FF0000");
    }

    #[test]
    fn test_compute_styling_dark_defaults() {
        let style = compute_styling("dark", None, None, None);
        assert_eq!(style.bg_top, "#161618");
        assert_eq!(style.bg_bottom, "#0D0D0E");
        assert_eq!(style.border, "#28282C");
        assert_eq!(style.symbol_color, "#FFFFFF");
    }
}
