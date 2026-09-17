//! CLI execution handlers and subcommand dispatchers.

use std::fs;
use std::path::PathBuf;

use crate::application::apply_icon::apply_icon_to_app;
use crate::application::fetch_vector::fetch_icon_or_create;
use crate::application::generate_icon::generate_single_icon;
use crate::application::sync_themes::{
    all_themes_workflow, create_theme_workflow, get_icons_base_dir, sync_themes_workflow,
};
use crate::cli::args::Cli;
use crate::domain::app::MacApp;
use crate::domain::icon::IconInfo;
use crate::domain::theme::compute_styling;
use crate::error::{InfraError, MacIconError};
use crate::infra::macos::tools::sips_convert_icns_to_png;
use crate::infra::render::svg::extract_path_from_svg;

/// Main CLI execution entrypoint.
pub fn run(args: Cli) -> Result<(), MacIconError> {
    if let Some(ref theme_name) = args.create_theme {
        return create_theme_workflow(
            theme_name,
            args.icons_dir.as_deref(),
            &args.from_theme,
            &args.theme,
            args.bg.as_deref(),
            args.color.as_deref(),
            args.border_color.as_deref(),
            args.no_shadow,
            args.scale,
        );
    }

    if args.sync_themes {
        return sync_themes_workflow(args.icons_dir.as_deref());
    }

    if let Some(ref icns_path_str) = args.icns {
        return handle_icns_action(&args, icns_path_str);
    }

    let (icon_info, base_name) = resolve_input_source(&args)?;

    if args.all_themes {
        return all_themes_workflow(args.icons_dir.as_deref(), &icon_info, &base_name);
    }

    let mut out_path: Option<PathBuf> = args.out.as_ref().map(PathBuf::from);

    if out_path.is_none() {
        let icons_dir = get_icons_base_dir(args.icons_dir.as_deref());
        let theme_dir = icons_dir.join(&args.theme);
        if theme_dir.is_dir() {
            out_path = Some(theme_dir.join(format!("{}.icns", base_name)));
        } else if let Some(ref app) = args.apply {
            let app_model = MacApp::new(app);
            out_path = Some(std::env::temp_dir().join(format!("{}.icns", app_model.safe_stem())));
        } else {
            out_path = Some(PathBuf::from(format!("./{}.icns", base_name)));
        }
    }

    let final_out = out_path.expect("Output path must be determined");

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
    let icns_result = generate_single_icon(
        &icon_info,
        &final_out,
        &style,
        args.scale,
        shadow,
        preview_path.as_deref(),
    )?;

    if let Some(ref app) = args.apply {
        apply_icon_to_app(app, &icns_result, !args.no_dock_restart)?;
    }

    Ok(())
}

fn handle_icns_action(args: &Cli, icns_path_str: &str) -> Result<(), MacIconError> {
    let icns_path = PathBuf::from(icns_path_str);
    if !icns_path.exists() {
        return Err(MacIconError::Infra(InfraError::IconFileNotFound(
            icns_path_str.to_string(),
        )));
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
        sips_convert_icns_to_png(&icns_path, &icns_preview)?;
        println!("Saved PNG preview: {}", icns_preview.display());
    }

    if let Some(ref out_dest) = args.out {
        let dest = PathBuf::from(out_dest);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&icns_path, &dest)?;
        println!("Copied icon to: {}", dest.display());
    }

    if let Some(ref app) = args.apply {
        apply_icon_to_app(app, &icns_path, !args.no_dock_restart)?;
    } else if args.preview.is_none() && args.out.is_none() {
        println!(
            "Icon verified at '{}'. Pass --apply <app_path> to apply it to an application.",
            icns_path.display()
        );
    }

    Ok(())
}

fn resolve_input_source(args: &Cli) -> Result<(IconInfo, String), MacIconError> {
    let query = args.search_query.as_ref().or(args.query.as_ref());

    if let Some(q) = query {
        let icon_info = fetch_icon_or_create(q, args.fallback_letter)?;
        let base_name = icon_info.name().to_string();
        Ok((icon_info, base_name))
    } else if let Some(ref letter) = args.letter {
        let icon_info = IconInfo::Letter {
            name: format!("letter-{}", letter.to_lowercase()),
            letter: letter.clone(),
        };
        let base_name = format!("letter-{}", letter.to_lowercase());
        Ok((icon_info, base_name))
    } else if let Some(ref svg_path_str) = args.svg {
        let svg_path = PathBuf::from(svg_path_str);
        let svg_content = fs::read_to_string(&svg_path).map_err(|e| {
            MacIconError::Cli(format!("Error reading SVG '{}': {}", svg_path_str, e))
        })?;
        let (path_d, viewbox, fill_rule) = extract_path_from_svg(&svg_content).map_err(|e| {
            MacIconError::Cli(format!("Error parsing SVG '{}': {}", svg_path_str, e))
        })?;
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
        Ok((icon_info, stem))
    } else if let Some(ref path_d) = args.path {
        let icon_info = IconInfo::Path {
            name: "custom".to_string(),
            path_d: path_d.clone(),
            viewbox: args.viewbox.clone(),
            fill_rule: "evenodd".to_string(),
        };
        Ok((icon_info, "custom".to_string()))
    } else {
        Err(MacIconError::Cli(
            "Must specify an icon source (e.g. macicon slack, --query, --letter, --svg, or --path)."
                .to_string(),
        ))
    }
}
