//! Command-line interface and dispatch for macicon.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;

use macicon::app_icon::apply_icon_to_app;
use macicon::fetcher::{IconInfo, extract_path_from_svg, fetch_icon_or_create};
use macicon::renderer::generate_single_icon;
use macicon::themes::{
    IconStyle, THEME_PRESETS, ThemeConfig, compute_styling, get_icons_base_dir,
    load_themes_manifest, save_themes_manifest,
};

#[derive(Parser, Debug)]
#[command(
    name = "macicon",
    about = "Generate, customize, and apply Apple continuous-curvature squircle app icons on macOS.",
    version
)]
pub struct Cli {
    /// Positional icon search query or name (e.g. 'slack')
    #[arg(value_name = "QUERY")]
    pub query: Option<String>,

    /// Search term or link from Simple Icons (e.g. 'slack', 'https://simpleicons.org/?q=warp')
    #[arg(short = 'q', long)]
    pub search_query: Option<String>,

    /// Path to a local SVG file
    #[arg(short = 's', long)]
    pub svg: Option<String>,

    /// Direct SVG path string d='...'
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Generate an Apple-style typography lettermark monogram (e.g. 'S', 'G', 'AI')
    #[arg(short = 'l', long)]
    pub letter: Option<String>,

    /// Path to an existing .icns file
    #[arg(long)]
    pub icns: Option<String>,

    /// Batch generate all existing icons into a new theme folder
    #[arg(long, value_name = "THEME_NAME")]
    pub create_theme: Option<String>,

    /// Sync and regenerate missing icons across all themes registered in themes.json
    #[arg(long)]
    pub sync_themes: bool,

    /// If query is not found online, automatically fall back to an Apple lettermark monogram
    #[arg(long)]
    pub fallback_letter: bool,

    /// Generate icon for all registered themes in themes.json
    #[arg(long)]
    pub all_themes: bool,

    /// Reference theme to discover icons from when using --create-theme
    #[arg(long, default_value = "light")]
    pub from_theme: String,

    /// Base directory containing theme folders (default: './icons' or auto-detected 'nix/icons')
    #[arg(long)]
    pub icons_dir: Option<String>,

    /// Base theme preset (default: light)
    #[arg(short = 't', long, default_value = "light")]
    pub theme: String,

    /// Background color/gradient: 'white', 'dark', '#FFFFFF', or gradient '#FFFFFF,#EBECEF'
    #[arg(short = 'b', long)]
    pub bg: Option<String>,

    /// Symbol color: 'black', 'white', hex '#202022', or gradient '#00C8FF,#0072FE'
    #[arg(short = 'c', long)]
    pub color: Option<String>,

    /// Custom tile border color (hex)
    #[arg(long)]
    pub border_color: Option<String>,

    /// Disable drop shadow on the symbol (for a flat appearance)
    #[arg(long)]
    pub no_shadow: bool,

    /// Scale multiplier for the symbol (default: 1.25)
    #[arg(long, default_value_t = 1.25)]
    pub scale: f64,

    /// SVG viewBox when using --path (default: '0 0 24 24')
    #[arg(short = 'v', long, default_value = "0 0 24 24")]
    pub viewbox: String,

    /// Destination path for .icns
    #[arg(short = 'o', long)]
    pub out: Option<String>,

    /// Generate a 1024x1024 PNG preview (optional custom path, or auto /tmp/<name>-preview.png)
    #[arg(long, num_args = 0..=1)]
    pub preview: Option<Option<String>>,

    /// Target .app bundle path or app name to immediately apply the icon to
    #[arg(short = 'a', long)]
    pub apply: Option<String>,

    /// Do not restart the Dock after applying the icon
    #[arg(long)]
    pub no_dock_restart: bool,
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

fn handle_create_theme(args: &Cli, theme_name: &str) {
    let icons_base_path = get_icons_base_dir(args.icons_dir.as_deref());
    let target_dir = icons_base_path.join(theme_name);
    let _ = fs::create_dir_all(&target_dir);

    let (ref_dir, ref_theme) = discover_reference_theme(&icons_base_path, &args.from_theme);
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
        eprintln!(
            "⚠️  Error: No .icns icons found in reference theme '{}' ({}).",
            ref_theme,
            ref_dir.display()
        );
        std::process::exit(1);
    }

    let mut base_theme = args.theme.as_str();
    if THEME_PRESETS.iter().any(|(k, _)| *k == theme_name)
        && args.bg.is_none()
        && args.theme == "light"
    {
        base_theme = theme_name;
    }

    let style = compute_styling(
        base_theme,
        args.bg.as_deref(),
        args.color.as_deref(),
        args.border_color.as_deref(),
    );
    let shadow = !args.no_shadow;
    let scale = args.scale;

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
    let _ = save_themes_manifest(&themes_file, &themes_data);

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
        let icon_info = fetch_icon_or_create(icon_name, true);
        match generate_single_icon(&icon_info, &out_file, &style, scale, shadow, None) {
            Ok(_) => {
                println!("✓");
                success_count += 1;
            }
            Err(e) => {
                println!("✗ ({})", e);
                failed_icons.push((icon_name.clone(), e));
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
}

fn handle_all_themes(args: &Cli, icon_info: &IconInfo, base_name: &str) {
    let icons_base_path = get_icons_base_dir(args.icons_dir.as_deref());
    let themes_file = icons_base_path.join("themes.json");
    let themes_data = load_themes_manifest(&themes_file, &icons_base_path);

    if themes_data.is_empty() {
        eprintln!(
            "⚠️  Error: No themes registered in '{}'. Run --create-theme or create theme directories first.",
            themes_file.display()
        );
        std::process::exit(1);
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
}

fn handle_sync_themes(args: &Cli) {
    let icons_base_path = get_icons_base_dir(args.icons_dir.as_deref());
    let themes_file = icons_base_path.join("themes.json");
    let themes_data = load_themes_manifest(&themes_file, &icons_base_path);

    if themes_data.is_empty() {
        eprintln!(
            "⚠️  Error: No themes registered in '{}'.",
            themes_file.display()
        );
        std::process::exit(1);
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
        eprintln!(
            "⚠️  Error: No .icns icons found across any themes in '{}'.",
            icons_base_path.display()
        );
        std::process::exit(1);
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
                let icon_info = fetch_icon_or_create(icon_name, true);
                match generate_single_icon(&icon_info, &out_file, &style, scale, shadow, None) {
                    Ok(_) => println!("✓"),
                    Err(e) => println!("✗ ({})", e),
                }
            }
        }
    }
    println!("\n✨ All themes are now synchronized!\n");
}

fn handle_icns_action(args: &Cli, icns_path_str: &str) {
    let icns_path = PathBuf::from(icns_path_str);
    if !icns_path.exists() {
        eprintln!("⚠️  Error: Icon file '{}' does not exist.", icns_path_str);
        std::process::exit(1);
    }

    let base_name = icns_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("custom");

    if let Some(ref prev_opt) = args.preview {
        let icns_preview = if let Some(custom_preview) = prev_opt {
            PathBuf::from(custom_preview)
        } else {
            std::env::temp_dir().join(format!("{}-preview.png", base_name))
        };
        let _ = Command::new("sips")
            .args([
                "-s",
                "format",
                "png",
                icns_path.to_str().unwrap(),
                "--out",
                icns_preview.to_str().unwrap(),
            ])
            .output();
        println!("Saved PNG preview: {}", icns_preview.display());
    }

    if let Some(ref out_dest) = args.out {
        let dest = PathBuf::from(out_dest);
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::copy(&icns_path, &dest) {
            eprintln!("Failed to copy icon: {}", e);
            std::process::exit(1);
        }
        println!("Copied icon to: {}", dest.display());
    }

    if let Some(ref app) = args.apply {
        if let Err(e) = apply_icon_to_app(app, &icns_path, !args.no_dock_restart) {
            eprintln!("⚠️  Error: {}", e);
            std::process::exit(1);
        }
    } else if args.preview.is_none() && args.out.is_none() {
        println!(
            "Icon verified at '{}'. Pass --apply <app_path> to apply it to an application.",
            icns_path.display()
        );
    }
}

fn resolve_input_source(args: &Cli) -> (IconInfo, String) {
    let query = args.search_query.as_ref().or(args.query.as_ref());

    if let Some(q) = query {
        let icon_info = fetch_icon_or_create(q, args.fallback_letter);
        let base_name = icon_info.name().to_string();
        (icon_info, base_name)
    } else if let Some(ref letter) = args.letter {
        let icon_info = IconInfo::Letter {
            name: format!("letter-{}", letter.to_lowercase()),
            letter: letter.clone(),
        };
        let base_name = format!("letter-{}", letter.to_lowercase());
        (icon_info, base_name)
    } else if let Some(ref svg_path_str) = args.svg {
        let svg_path = PathBuf::from(svg_path_str);
        let svg_content = fs::read_to_string(&svg_path).unwrap_or_else(|e| {
            eprintln!("⚠️  Error reading SVG '{}': {}", svg_path_str, e);
            std::process::exit(1);
        });
        match extract_path_from_svg(&svg_content) {
            Ok((path_d, viewbox, fill_rule)) => {
                let stem = svg_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("custom")
                    .to_lowercase();
                let icon_info = IconInfo::Path {
                    name: stem.clone(),
                    path_d,
                    viewbox,
                    fill_rule,
                };
                (icon_info, stem)
            }
            Err(e) => {
                eprintln!("⚠️  Error parsing SVG '{}': {}", svg_path_str, e);
                std::process::exit(1);
            }
        }
    } else if let Some(ref path_d) = args.path {
        let icon_info = IconInfo::Path {
            name: "custom".to_string(),
            path_d: path_d.clone(),
            viewbox: args.viewbox.clone(),
            fill_rule: "evenodd".to_string(),
        };
        (icon_info, "custom".to_string())
    } else {
        eprintln!(
            "⚠️  Error: Must specify an icon source (e.g. macicon slack, --query, --letter, --svg, or --path)."
        );
        std::process::exit(1);
    }
}

fn main() {
    let args = Cli::parse();

    if let Some(ref theme_name) = args.create_theme {
        handle_create_theme(&args, theme_name);
        return;
    }

    if args.sync_themes {
        handle_sync_themes(&args);
        return;
    }

    if let Some(ref icns_path) = args.icns {
        handle_icns_action(&args, icns_path);
        return;
    }

    let (icon_info, base_name) = resolve_input_source(&args);

    if args.all_themes {
        handle_all_themes(&args, &icon_info, &base_name);
        return;
    }

    let mut out_path: Option<PathBuf> = args.out.as_ref().map(PathBuf::from);

    if out_path.is_none() {
        let icons_dir = get_icons_base_dir(args.icons_dir.as_deref());
        let theme_dir = icons_dir.join(&args.theme);
        if theme_dir.is_dir() {
            out_path = Some(theme_dir.join(format!("{}.icns", base_name)));
        } else if let Some(ref app) = args.apply {
            let stem = Path::new(app)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("app")
                .to_lowercase()
                .replace(' ', "-");
            out_path = Some(std::env::temp_dir().join(format!("{}.icns", stem)));
        } else {
            out_path = Some(PathBuf::from(format!("./{}.icns", base_name)));
        }
    }

    let final_out = out_path.unwrap();

    let preview_path = args.preview.as_ref().map(|prev_opt| {
        if let Some(custom_preview) = prev_opt {
            PathBuf::from(custom_preview)
        } else {
            std::env::temp_dir().join(format!("{}-preview.png", base_name))
        }
    });

    let style = compute_styling(
        &args.theme,
        args.bg.as_deref(),
        args.color.as_deref(),
        args.border_color.as_deref(),
    );

    let shadow = !args.no_shadow;
    match generate_single_icon(
        &icon_info,
        &final_out,
        &style,
        args.scale,
        shadow,
        preview_path.as_deref(),
    ) {
        Ok(icns_result) => {
            if let Some(ref app) = args.apply
                && let Err(e) = apply_icon_to_app(app, &icns_result, !args.no_dock_restart)
            {
                eprintln!("⚠️  Error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("⚠️  Error generating icon: {}", e);
            std::process::exit(1);
        }
    }
}
