//! Integration tests for theme manifest loading, saving, auto-discovery, and file system layout.

use std::collections::HashMap;
use std::fs;

use macicon::application::sync_themes::{load_themes_manifest, save_themes_manifest};
use macicon::domain::theme::ThemeConfig;
use tempfile::tempdir;

#[test]
fn test_themes_manifest_roundtrip_persistence() {
    let tmp = tempdir().unwrap();
    let manifest_file = tmp.path().join("themes.json");

    let mut catalog = HashMap::new();
    catalog.insert(
        "nord".to_string(),
        ThemeConfig {
            bg_top: "#2E3440".to_string(),
            bg_bottom: "#242933".to_string(),
            border: "#3B4252".to_string(),
            symbol_color: "#ECEFF4".to_string(),
            shadow: true,
            scale: 1.25,
        },
    );
    catalog.insert(
        "solarized-dark".to_string(),
        ThemeConfig {
            bg_top: "#073642".to_string(),
            bg_bottom: "#002B36".to_string(),
            border: "#586E75".to_string(),
            symbol_color: "#FDF6E3".to_string(),
            shadow: false,
            scale: 1.2,
        },
    );

    save_themes_manifest(&manifest_file, &catalog).unwrap();
    assert!(manifest_file.is_file());

    let loaded = load_themes_manifest(&manifest_file, tmp.path());
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.get("nord").unwrap().symbol_color, "#ECEFF4");
    assert!(!loaded.get("solarized-dark").unwrap().shadow);
    assert_eq!(loaded.get("solarized-dark").unwrap().scale, 1.2);
}

#[test]
fn test_themes_manifest_auto_detect_from_existing_folders() {
    let tmp = tempdir().unwrap();
    let dark_folder = tmp.path().join("dark");
    fs::create_dir_all(&dark_folder).unwrap();

    let manifest_file = tmp.path().join("missing.json");
    let loaded = load_themes_manifest(&manifest_file, tmp.path());

    assert!(loaded.contains_key("dark"));
    let dark_cfg = loaded.get("dark").unwrap();
    assert_eq!(dark_cfg.bg_top, "#161618");
    assert_eq!(dark_cfg.bg_bottom, "#0D0D0E");
    assert_eq!(dark_cfg.border, "#28282C");
    assert_eq!(dark_cfg.symbol_color, "#FFFFFF");
    assert!(!dark_cfg.shadow);
}
