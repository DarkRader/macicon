//! Application use case: Multi-theme synchronization, creation, and batch generation.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::application::fetch_vector::fetch_icon_or_create;
use crate::application::generate_icon::generate_single_icon;
use crate::domain::icon::IconInfo;
use crate::domain::theme::{IconStyle, THEME_PRESETS, ThemeConfig, compute_styling};
use crate::error::MacIconError;

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
) -> Result<(), MacIconError> {
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json_str = serde_json::to_string_pretty(data)
        .map_err(|e| MacIconError::Infra(crate::error::InfraError::Json(e)))?;
    fs::write(manifest_path, format!("{}\n", json_str))?;
    Ok(())
}

fn discover_reference_theme(icons_base_path: &Path, from_theme: &str) -> (PathBuf, String) {
    let ref_theme = from_theme.to_string();
    let ref_dir = icons_base_path.join(&ref_theme);
    if ref_dir.is_dir() {
        return (ref_dir, ref_theme);
    }

    let mut candidates = vec!["light".to_string(), "dark".to_string()];
    if let Ok(entries) = fs::read_dir(icons_base_path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type()
                && file_type.is_dir()
            {
                let name = entry.file_name().to_string_lossy().to_string();
                if !candidates.contains(&name) {
                    candidates.push(name);
                }
            }
        }
    }

    for cand in candidates {
        let cand_dir = icons_base_path.join(&cand);
        if cand_dir.is_dir() {
            return (cand_dir, cand);
        }
    }

    (ref_dir, ref_theme)
}

/// Batch generate all existing icons into a new theme directory.
#[allow(clippy::too_many_arguments)]
pub fn create_theme_workflow(
    theme_name: &str,
    icons_dir: Option<&str>,
    from_theme: &str,
    base_theme: &str,
    bg: Option<&str>,
    color: Option<&str>,
    border_color: Option<&str>,
    no_shadow: bool,
    scale: f64,
) -> Result<(), MacIconError> {
    let icons_base_path = get_icons_base_dir(icons_dir);
    let target_dir = icons_base_path.join(theme_name);
    let _ = fs::create_dir_all(&target_dir);

    let (ref_dir, ref_theme) = discover_reference_theme(&icons_base_path, from_theme);
    let mut existing_icons = Vec::new();
    if ref_dir.is_dir()
        && let Ok(entries) = fs::read_dir(&ref_dir)
    {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|ext| ext == "icns")
                && let Some(stem) = p.file_stem().and_then(|s| s.to_str())
            {
                existing_icons.push(stem.to_string());
            }
        }
    }
    existing_icons.sort();

    if existing_icons.is_empty() {
        return Err(MacIconError::Cli(format!(
            "No .icns icons found in reference theme '{}' ({}).",
            ref_theme,
            ref_dir.display()
        )));
    }

    let mut effective_theme = base_theme;
    if THEME_PRESETS.iter().any(|(k, _)| *k == theme_name) && bg.is_none() && base_theme == "light"
    {
        effective_theme = theme_name;
    }

    let style = compute_styling(effective_theme, bg, color, border_color);
    let shadow = !no_shadow;

    let bg_display = if style.bg_top == style.bg_bottom {
        style.bg_top.clone()
    } else {
        format!("{} -> {}", style.bg_top, style.bg_bottom)
    };

    println!(
        "\n🎨 Creating new icon theme '{}' with {} icons:",
        theme_name,
        existing_icons.len()
    );
    println!("   Target Directory : {}", target_dir.display());
    println!(
        "   Reference Theme  : {} ({} icons)",
        ref_theme,
        existing_icons.len()
    );
    println!("   Background       : {}", bg_display);
    println!("   Border           : {}", style.border);
    println!("   Symbol Color     : {}", style.symbol_color);
    println!(
        "   Shadow           : {}",
        if shadow { "Enabled" } else { "Disabled (flat)" }
    );
    println!("   Scale            : {}\n", scale);

    let themes_file = icons_base_path.join("themes.json");
    let mut themes_data = load_themes_manifest(&themes_file, &icons_base_path);
    themes_data.insert(
        theme_name.to_string(),
        ThemeConfig {
            bg_top: style.bg_top.clone(),
            bg_bottom: style.bg_bottom.clone(),
            border: style.border.clone(),
            symbol_color: style.symbol_color.clone(),
            shadow,
            scale,
        },
    );
    save_themes_manifest(&themes_file, &themes_data)?;

    let mut success_count = 0;
    let mut failed_icons = Vec::new();

    for (i, icon_name) in existing_icons.iter().enumerate() {
        print!(
            "   [{:2}/{}] Generating {}.icns ... ",
            i + 1,
            existing_icons.len(),
            icon_name
        );
        let out_file = target_dir.join(format!("{}.icns", icon_name));
        let icon_info = match fetch_icon_or_create(icon_name, true) {
            Ok(info) => info,
            Err(e) => {
                println!("✗ ({})", e);
                failed_icons.push((icon_name.clone(), e.to_string()));
                continue;
            }
        };

        match generate_single_icon(&icon_info, &out_file, &style, scale, shadow, None) {
            Ok(_) => {
                println!("✓");
                success_count += 1;
            }
            Err(e) => {
                println!("✗ ({})", e);
                failed_icons.push((icon_name.clone(), e.to_string()));
            }
        }
    }

    println!(
        "\n✨ Successfully generated {}/{} icons.",
        success_count,
        existing_icons.len()
    );
    if !failed_icons.is_empty() {
        println!("⚠️  Failed icons ({}):", failed_icons.len());
        for (name, err) in failed_icons {
            println!("   - {}: {}", name, err);
        }
    }

    Ok(())
}

/// Generate a single icon across all registered themes in themes.json.
pub fn all_themes_workflow(
    icons_dir: Option<&str>,
    icon_info: &IconInfo,
    base_name: &str,
) -> Result<(), MacIconError> {
    let icons_base_path = get_icons_base_dir(icons_dir);
    let themes_file = icons_base_path.join("themes.json");
    let themes_data = load_themes_manifest(&themes_file, &icons_base_path);

    if themes_data.is_empty() {
        return Err(MacIconError::Cli(format!(
            "No themes registered in '{}'. Run --create-theme or create theme directories first.",
            themes_file.display()
        )));
    }

    println!(
        "\n🎨 Generating '{}.icns' across all {} themes ({}) ...\n",
        base_name,
        themes_data.len(),
        themes_data.keys().cloned().collect::<Vec<_>>().join(", ")
    );

    for (theme_name, cfg) in &themes_data {
        let t_dir = icons_base_path.join(theme_name);
        let _ = fs::create_dir_all(&t_dir);
        let out_file = t_dir.join(format!("{}.icns", base_name));

        let style = IconStyle {
            bg_top: cfg.bg_top.clone(),
            bg_bottom: cfg.bg_bottom.clone(),
            border: cfg.border.clone(),
            symbol_color: cfg.symbol_color.clone(),
        };
        let scale = cfg.scale;
        let shadow = cfg.shadow;

        print!(
            "   Theme [{:12}] -> {} ... ",
            theme_name,
            out_file.display()
        );
        match generate_single_icon(icon_info, &out_file, &style, scale, shadow, None) {
            Ok(_) => println!("✓"),
            Err(e) => println!("✗ ({})", e),
        }
    }
    println!("\n✨ Completed icon generation across all themes!\n");
    Ok(())
}

/// Synchronize missing icons across all themes registered in themes.json.
pub fn sync_themes_workflow(icons_dir: Option<&str>) -> Result<(), MacIconError> {
    let icons_base_path = get_icons_base_dir(icons_dir);
    let themes_file = icons_base_path.join("themes.json");
    let themes_data = load_themes_manifest(&themes_file, &icons_base_path);

    if themes_data.is_empty() {
        return Err(MacIconError::Cli(format!(
            "No themes registered in '{}'.",
            themes_file.display()
        )));
    }

    let mut all_icon_names = HashSet::new();
    for theme_name in themes_data.keys() {
        let t_dir = icons_base_path.join(theme_name);
        if t_dir.is_dir()
            && let Ok(entries) = fs::read_dir(&t_dir)
        {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|e| e == "icns")
                    && let Some(stem) = p.file_stem().and_then(|s| s.to_str())
                {
                    all_icon_names.insert(stem.to_string());
                }
            }
        }
    }

    if all_icon_names.is_empty() {
        return Err(MacIconError::Cli(format!(
            "No .icns icons found across any themes in '{}'.",
            icons_base_path.display()
        )));
    }

    let mut all_icons_list: Vec<String> = all_icon_names.into_iter().collect();
    all_icons_list.sort();

    println!(
        "\n🔄 Syncing {} icons across {} themes ({})...\n",
        all_icons_list.len(),
        themes_data.len(),
        themes_data.keys().cloned().collect::<Vec<_>>().join(", ")
    );

    for (theme_name, cfg) in &themes_data {
        println!("📦 Theme: {}", theme_name);
        let t_dir = icons_base_path.join(theme_name);
        let _ = fs::create_dir_all(&t_dir);

        let style = IconStyle {
            bg_top: cfg.bg_top.clone(),
            bg_bottom: cfg.bg_bottom.clone(),
            border: cfg.border.clone(),
            symbol_color: cfg.symbol_color.clone(),
        };
        let scale = cfg.scale;
        let shadow = cfg.shadow;

        for icon_name in &all_icons_list {
            let out_file = t_dir.join(format!("{}.icns", icon_name));
            if !out_file.exists() {
                print!("   Adding missing {}.icns ... ", icon_name);
                let icon_info = match fetch_icon_or_create(icon_name, true) {
                    Ok(info) => info,
                    Err(e) => {
                        println!("✗ ({})", e);
                        continue;
                    }
                };
                match generate_single_icon(&icon_info, &out_file, &style, scale, shadow, None) {
                    Ok(_) => println!("✓"),
                    Err(e) => println!("✗ ({})", e),
                }
            }
        }
    }
    println!("\n✨ All themes are now synchronized!\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_icons_base_dir() {
        assert_eq!(
            get_icons_base_dir(Some("/custom/path")),
            PathBuf::from("/custom/path")
        );
        let default_dir = get_icons_base_dir(None);
        assert!(default_dir == Path::new("icons") || default_dir == Path::new("nix/icons"));
    }

    #[test]
    fn test_save_and_load_themes_manifest() {
        let tmp = tempdir().unwrap();
        let manifest_path = tmp.path().join("themes.json");

        let mut sample_themes = HashMap::new();
        sample_themes.insert(
            "custom".to_string(),
            ThemeConfig {
                bg_top: "#111111".to_string(),
                bg_bottom: "#222222".to_string(),
                border: "#333333".to_string(),
                symbol_color: "#FFFFFF".to_string(),
                shadow: true,
                scale: 1.3,
            },
        );

        save_themes_manifest(&manifest_path, &sample_themes).unwrap();
        assert!(manifest_path.is_file());

        let loaded = load_themes_manifest(&manifest_path, tmp.path());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.get("custom").unwrap().bg_top, "#111111");
        assert_eq!(loaded.get("custom").unwrap().scale, 1.3);
    }

    #[test]
    fn test_load_themes_manifest_with_directory_defaults() {
        let tmp = tempdir().unwrap();
        let light_dir = tmp.path().join("light");
        fs::create_dir_all(&light_dir).unwrap();

        let manifest_path = tmp.path().join("non_existent_themes.json");
        let loaded = load_themes_manifest(&manifest_path, tmp.path());
        assert!(loaded.contains_key("light"));
        assert_eq!(loaded.get("light").unwrap().bg_top, "#FFFFFF");
    }
}
