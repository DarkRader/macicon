//! Integration tests for domain models, aggregates, and business invariant calculations.

use macicon::domain::app::MacApp;
use macicon::domain::color::{is_color_dark, normalize_hex, resolve_background_colors};
use macicon::domain::geometry::{
    CANVAS_SIZE, COCOA_TILE_Y, CORNER_RADIUS, TILE_SIZE, TILE_X, TILE_Y,
    calculate_lettermark_metrics, calculate_symbol_transform,
};
use macicon::domain::icon::{derive_candidate_slugs, find_known_icon};
use macicon::domain::theme::{THEME_PRESETS, compute_styling, find_preset};

#[test]
fn test_apple_hig_geometry_invariants() {
    assert_eq!(CANVAS_SIZE, 1024);
    assert_eq!(TILE_SIZE, 832);
    assert_eq!(TILE_X, 96);
    assert_eq!(TILE_Y, 88);
    assert_eq!(CORNER_RADIUS, 185);

    // Apple continuous-curvature squircle Cocoa origin offset
    assert_eq!(CANVAS_SIZE - TILE_Y - TILE_SIZE, COCOA_TILE_Y);
    assert_eq!(COCOA_TILE_Y, 104);
}

#[test]
fn test_theme_and_styling_integration() {
    for (name, preset) in THEME_PRESETS {
        let style = compute_styling(name, None, None, None);
        assert_eq!(style.bg_top, preset.bg_top);
        assert_eq!(style.bg_bottom, preset.bg_bottom);
        assert_eq!(style.symbol_color, preset.symbol);

        let found = find_preset(name);
        assert!(found.is_some());
        assert_eq!(found.unwrap().bg_top, preset.bg_top);
        assert_eq!(found.unwrap().border, preset.border);
    }
}

#[test]
fn test_custom_gradient_and_contrast_integration() {
    let (top, btm) = resolve_background_colors("black");
    assert!(is_color_dark(&top));
    assert!(is_color_dark(&btm));

    let style = compute_styling("dark", Some("black"), None, None);
    assert_eq!(style.bg_top, "#161618");
    assert_eq!(style.bg_bottom, "#0D0D0E");
    assert_eq!(style.border, "#28282C");

    let norm_white = normalize_hex("fff").unwrap();
    assert_eq!(norm_white, "#FFFFFF");
    assert!(!is_color_dark(&norm_white));
}

#[test]
fn test_icon_alias_and_candidate_resolution() {
    let aliases = [
        ("zed", "zedindustries"),
        ("gemini", "googlegemini"),
        ("vscode", "visualstudiocode"),
        ("cron", "notion-calendar"),
    ];

    for (alias, expected_cand) in aliases {
        let (_, candidates) = derive_candidate_slugs(alias);
        assert!(
            candidates.iter().any(|c| c == expected_cand),
            "Expected candidate '{}' for alias '{}'",
            expected_cand,
            alias
        );
    }

    let known_spark = find_known_icon("spark").expect("spark icon must exist");
    assert_eq!(known_spark.name(), "spark");
}

#[test]
fn test_app_model_and_bundle_discovery_derivation() {
    let app = MacApp::new("Visual Studio Code");
    assert_eq!(app.safe_stem(), "visual-studio-code");

    let candidates = app.candidate_paths(None);
    assert!(
        candidates
            .iter()
            .any(|p| p.ends_with("Visual Studio Code.app"))
    );
}

#[test]
fn test_symbol_and_lettermark_transforms() {
    let transform = calculate_symbol_transform(0.0, 0.0, 48.0, 48.0, 1.0);
    assert_eq!(transform.center_x, 24.0);
    assert_eq!(transform.center_y, 24.0);
    assert_eq!(transform.scale, 10.0);

    let metrics_single = calculate_lettermark_metrics("M", 1.0);
    assert_eq!(metrics_single.font_size, 400);
    assert_eq!(metrics_single.y_pos, 635);

    let metrics_double = calculate_lettermark_metrics("AI", 1.0);
    assert_eq!(metrics_double.font_size, 280);
    assert_eq!(metrics_double.y_pos, 600);
}
