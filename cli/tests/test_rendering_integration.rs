//! Integration tests for SVG squircle composition, monogram rendering, and vector path extraction.

use macicon::domain::theme::IconStyle;
use macicon::infra::render::svg::{build_letter_svg, build_svg, extract_path_from_svg};

#[test]
fn test_squircle_svg_composition_properties() {
    let style = IconStyle {
        bg_top: "#FFFFFF".to_string(),
        bg_bottom: "#EBECEF".to_string(),
        border: "#D8D9DC".to_string(),
        symbol_color: "#202022".to_string(),
    };

    let svg = build_svg(
        "M10 10 L50 50",
        "0 0 100 100",
        &style,
        1.25,
        "evenodd",
        true,
    );

    // Verify canvas dimensions and squircle rectangle
    assert!(svg.contains("width=\"1024\""));
    assert!(svg.contains("height=\"1024\""));
    assert!(svg.contains("<rect x=\"96\" y=\"88\" width=\"832\" height=\"832\" rx=\"185\""));
    assert!(svg.contains("stroke") && svg.contains("#D8D9DC"));

    // Verify background gradient
    assert!(svg.contains("<stop offset=\"0%\"") && svg.contains("#FFFFFF"));
    assert!(svg.contains("<stop offset=\"100%\"") && svg.contains("#EBECEF"));

    // Verify drop shadow filter is attached
    assert!(svg.contains("filter=\"url(#symbol-shadow)\""));
    assert!(svg.contains("<feDropShadow"));

    // Verify path content
    assert!(svg.contains("d=\"M10 10 L50 50\""));
}

#[test]
fn test_dual_color_symbol_gradient() {
    let style = IconStyle {
        bg_top: "#000000".to_string(),
        bg_bottom: "#111111".to_string(),
        border: "#222222".to_string(),
        symbol_color: "#FF0000,#00FF00".to_string(),
    };

    let svg = build_svg("M0 0 L10 10", "0 0 20 20", &style, 1.0, "nonzero", false);

    assert!(svg.contains("<linearGradient id=\"symbol-grad\""));
    assert!(svg.contains("stop-color") && svg.contains("#FF0000"));
    assert!(svg.contains("stop-color") && svg.contains("#00FF00"));
    assert!(svg.contains("fill=\"url(#symbol-grad)\""));
    assert!(!svg.contains("filter=\"url(#symbol-shadow)\""));
}

#[test]
fn test_lettermark_monogram_svg() {
    let style = IconStyle {
        bg_top: "#1E1E2E".to_string(),
        bg_bottom: "#181825".to_string(),
        border: "#313244".to_string(),
        symbol_color: "#CDD6F4".to_string(),
    };

    let svg_single = build_letter_svg("K", &style, 1.0, true);
    assert!(svg_single.contains(">K</text>"));
    assert!(svg_single.contains("font-size=\"400\""));
    assert!(svg_single.contains("y=\"635\""));

    let svg_double = build_letter_svg("GO", &style, 1.0, false);
    assert!(svg_double.contains(">GO</text>"));
    assert!(svg_double.contains("font-size=\"280\""));
    assert!(svg_double.contains("y=\"600\""));
}

#[test]
fn test_extract_complex_svg_paths() {
    let complex_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill-rule="nonzero">
        <g id="layer1">
            <path d="M100 100 L200 200 Z" />
            <path d="M300 300 L400 400 Z" />
        </g>
    </svg>"#;

    let (path_d, viewbox, fill_rule) = extract_path_from_svg(complex_svg).unwrap();
    assert_eq!(viewbox, "0 0 512 512");
    assert_eq!(fill_rule, "nonzero");
    assert!(path_d.contains("M100 100 L200 200 Z"));
    assert!(path_d.contains("M300 300 L400 400 Z"));
}
