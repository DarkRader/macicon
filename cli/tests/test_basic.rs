use macicon::fetcher::extract_path_from_svg;
use macicon::renderer::build_letter_svg;
use macicon::themes::{compute_styling, is_color_dark, IconStyle, THEME_PRESETS};

#[test]
fn test_theme_presets() {
    let preset_names: Vec<&str> = THEME_PRESETS.iter().map(|(k, _)| *k).collect();
    assert!(preset_names.contains(&"light"));
    assert!(preset_names.contains(&"dark"));
    assert!(preset_names.contains(&"nord"));
    assert!(preset_names.contains(&"apple"));
}

#[test]
fn test_color_dark() {
    assert!(is_color_dark("#000000"));
    assert!(!is_color_dark("#FFFFFF"));
    assert!(is_color_dark("#161618"));
    assert!(!is_color_dark("#EBECEF"));
    assert!(is_color_dark("#000")); // 3-digit hex
    assert!(!is_color_dark("#fff")); // 3-digit hex
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
fn test_extract_path() {
    let svg = r#"<svg viewBox="0 0 48 48"><path d="M10 10 L20 20"/></svg>"#;
    let (path_d, vb, fr) = extract_path_from_svg(svg).unwrap();
    assert_eq!(path_d, "M10 10 L20 20");
    assert_eq!(vb, "0 0 48 48");
    assert_eq!(fr, "evenodd");
}

#[test]
fn test_build_letter_svg() {
    let style = IconStyle {
        bg_top: "#FFF".to_string(),
        bg_bottom: "#EEE".to_string(),
        border: "#DDD".to_string(),
        symbol_color: "#000".to_string(),
    };
    let svg = build_letter_svg("A", &style, 1.25, true);
    assert!(svg.contains("<text"));
    assert!(svg.contains(">A</text>"));
    assert!(svg.contains("rx=\"185\""));
}
