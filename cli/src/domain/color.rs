//! Color models, luminance calculation, and palette shortcuts for macicon.

use crate::error::DomainError;

const HEX_SHORT_LEN: usize = 3;
const HEX_FULL_LEN: usize = 6;
const DARK_LUMINANCE_THRESHOLD: f64 = 100.0;

/// Built-in background color shortcuts mapping a name to (top, bottom) gradient colors.
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

/// Built-in symbol color shortcuts mapping a name to a hex or gradient.
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

/// Normalize a 3-character or 6-character hex string into a standard 6-character hex.
pub fn normalize_hex(hex_str: &str) -> Result<String, DomainError> {
    let trimmed = hex_str.trim().trim_start_matches('#');
    if trimmed.len() == HEX_SHORT_LEN {
        let mut full = String::with_capacity(7);
        full.push('#');
        for c in trimmed.chars() {
            full.push(c);
            full.push(c);
        }
        Ok(full.to_uppercase())
    } else if trimmed.len() == HEX_FULL_LEN {
        let mut full = String::with_capacity(7);
        full.push('#');
        full.push_str(trimmed);
        Ok(full.to_uppercase())
    } else {
        Err(DomainError::InvalidHexColor(hex_str.to_string()))
    }
}

/// Calculate perceived luminance from a hex color.
pub fn calculate_luminance(hex_color: &str) -> Option<f64> {
    let hex = hex_color.trim().trim_start_matches('#');
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
        return None;
    };

    let r = u8::from_str_radix(&normalized[0..2], 16).ok()?;
    let g = u8::from_str_radix(&normalized[2..4], 16).ok()?;
    let b = u8::from_str_radix(&normalized[4..6], 16).ok()?;

    Some(0.299 * (r as f64) + 0.587 * (g as f64) + 0.114 * (b as f64))
}

/// Return true if the hex color has a dark perceived luminance (< 100.0).
pub fn is_color_dark(hex_color: &str) -> bool {
    calculate_luminance(hex_color).is_some_and(|lum| lum < DARK_LUMINANCE_THRESHOLD)
}

/// Resolve a background color or gradient string against shortcuts or comma-separated pairs.
pub fn resolve_background_colors(bg_input: &str) -> (String, String) {
    let raw_bg = bg_input.trim();
    let lowered = raw_bg.to_lowercase();
    if let Some((_, (top, bottom))) = COLOR_SHORTCUTS.iter().find(|(k, _)| *k == lowered) {
        return ((*top).to_string(), (*bottom).to_string());
    }

    let parts: Vec<&str> = raw_bg.split(',').map(|s| s.trim()).collect();
    if parts.len() == 1 {
        (parts[0].to_string(), parts[0].to_string())
    } else {
        (parts[0].to_string(), parts[1].to_string())
    }
}

/// Resolve a symbol color string against shortcuts.
pub fn resolve_symbol_color(color_input: &str) -> String {
    let raw_color = color_input.trim();
    let lowered = raw_color.to_lowercase();
    if let Some((_, shortcut)) = SYMBOL_SHORTCUTS.iter().find(|(k, _)| *k == lowered) {
        (*shortcut).to_string()
    } else {
        raw_color.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_hex() {
        assert_eq!(normalize_hex("#fff").unwrap(), "#FFFFFF");
        assert_eq!(normalize_hex("000").unwrap(), "#000000");
        assert_eq!(normalize_hex("#161618").unwrap(), "#161618");
        assert!(normalize_hex("invalid").is_err());
    }

    #[test]
    fn test_is_color_dark() {
        assert!(is_color_dark("#000000"));
        assert!(!is_color_dark("#FFFFFF"));
        assert!(is_color_dark("#161618"));
        assert!(!is_color_dark("#EBECEF"));
        assert!(is_color_dark("#000"));
        assert!(!is_color_dark("#fff"));
    }

    #[test]
    fn test_resolve_background_colors() {
        let (top, btm) = resolve_background_colors("white");
        assert_eq!(top, "#FFFFFF");
        assert_eq!(btm, "#EBECEF");

        let (top2, btm2) = resolve_background_colors("#111111,#222222");
        assert_eq!(top2, "#111111");
        assert_eq!(btm2, "#222222");

        let (top3, btm3) = resolve_background_colors("#FF0000");
        assert_eq!(top3, "#FF0000");
        assert_eq!(btm3, "#FF0000");
    }

    #[test]
    fn test_resolve_symbol_color() {
        assert_eq!(resolve_symbol_color("black"), "#202022");
        assert_eq!(resolve_symbol_color("apple"), "#00C8FF,#0072FE");
        assert_eq!(resolve_symbol_color("#123456"), "#123456");
    }
}
