//! Apple HIG squircle geometry and canvas invariants for macicon.

/// Standard macOS application icon canvas dimension (1024x1024).
pub const CANVAS_SIZE: u32 = 1024;

/// Standard Apple squircle tile dimension (832x832).
pub const TILE_SIZE: u32 = 832;

/// Standard Apple squircle tile X offset.
pub const TILE_X: u32 = 96;

/// Standard Apple squircle tile Y offset in top-left coordinate system (SVG / Web).
pub const TILE_Y: u32 = 88;

/// Standard Apple squircle tile Y offset in bottom-left coordinate system (Cocoa / AppKit).
/// Derived as: CANVAS_SIZE (1024) - TILE_Y (88) - TILE_SIZE (832) = 104.
pub const COCOA_TILE_Y: u32 = 104;

/// Apple continuous-curvature squircle corner radius.
pub const CORNER_RADIUS: u32 = 185;

/// Default baseline vector symbol size on the 1024x1024 canvas.
pub const DEFAULT_SYMBOL_SIZE: f64 = 480.0;

/// Symbol placement and scaling transform.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolTransform {
    pub scale: f64,
    pub center_x: f64,
    pub center_y: f64,
}

/// Compute vector symbol scaling factor and center point based on its viewBox dimensions.
pub fn calculate_symbol_transform(
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
    scale: f64,
) -> SymbolTransform {
    let center_x = min_x + (width / 2.0);
    let center_y = min_y + (height / 2.0);
    let max_dim = width.max(height).max(1.0);
    let calc_scale = (DEFAULT_SYMBOL_SIZE / max_dim) * scale;

    SymbolTransform {
        scale: calc_scale,
        center_x,
        center_y,
    }
}

/// Typography lettermark font size and vertical alignment baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LettermarkMetrics {
    pub font_size: u32,
    pub y_pos: u32,
}

/// Compute font size and vertical position for Apple-style lettermark monograms.
pub fn calculate_lettermark_metrics(letter: &str, scale: f64) -> LettermarkMetrics {
    let count = letter.chars().count();
    let (base_size, y_pos) = if count == 1 {
        (400.0, 635)
    } else {
        (280.0, 600)
    };

    LettermarkMetrics {
        font_size: (base_size * scale) as u32,
        y_pos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_hig_dimensions() {
        assert_eq!(CANVAS_SIZE, 1024);
        assert_eq!(TILE_SIZE, 832);
        assert_eq!(TILE_X, 96);
        assert_eq!(TILE_Y, 88);
        assert_eq!(CORNER_RADIUS, 185);
        // Cocoa coordinate invariant check
        assert_eq!(CANVAS_SIZE - TILE_Y - TILE_SIZE, COCOA_TILE_Y);
        assert_eq!(COCOA_TILE_Y, 104);
    }

    #[test]
    fn test_symbol_transform() {
        let t = calculate_symbol_transform(0.0, 0.0, 24.0, 24.0, 1.25);
        assert_eq!(t.center_x, 12.0);
        assert_eq!(t.center_y, 12.0);
        assert_eq!(t.scale, (480.0 / 24.0) * 1.25); // 25.0
    }

    #[test]
    fn test_lettermark_metrics() {
        let single = calculate_lettermark_metrics("A", 1.25);
        assert_eq!(single.font_size, 500);
        assert_eq!(single.y_pos, 635);

        let double = calculate_lettermark_metrics("AI", 1.0);
        assert_eq!(double.font_size, 280);
        assert_eq!(double.y_pos, 600);
    }
}
