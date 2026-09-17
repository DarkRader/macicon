//! Application use case: Single icon generation workflow.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::domain::icon::IconInfo;
use crate::domain::theme::IconStyle;
use crate::error::MacIconError;
use crate::infra::render::icns::{compile_icns, render_and_mask};
use crate::infra::render::svg::{build_letter_svg, build_svg};

/// Generate a complete squircle .icns package from vector or monogram info.
pub fn generate_single_icon(
    icon_info: &IconInfo,
    out_icns: &Path,
    style: &IconStyle,
    scale: f64,
    shadow: bool,
    preview_path: Option<&Path>,
) -> Result<PathBuf, MacIconError> {
    let temp_dir = TempDir::new().map_err(|e| MacIconError::Infra(e.into()))?;
    let t_dir = temp_dir.path();

    let svg_markup = match icon_info {
        IconInfo::Letter { letter, .. } => build_letter_svg(letter, style, scale, shadow),
        IconInfo::Path {
            path_d,
            viewbox,
            fill_rule,
            ..
        } => build_svg(path_d, viewbox, style, scale, fill_rule, shadow),
    };

    let svg_file = t_dir.join("composed.svg");
    fs::write(&svg_file, &svg_markup)?;

    let masked_png = render_and_mask(&svg_file, t_dir, shadow)?;

    if let Some(prev) = preview_path {
        if let Some(parent) = prev.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::copy(&masked_png, prev)?;
        println!("Saved PNG preview: {}", prev.display());
    }

    compile_icns(&masked_png, out_icns, t_dir)?;
    println!("Compiled ICNS: {}", out_icns.display());
    Ok(out_icns.to_path_buf())
}
