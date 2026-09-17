//! Multi-resolution iconset compilation and .icns bundle generation.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::InfraError;
use crate::infra::macos::tools::{
    iconutil_compile, mask_png_with_swift, render_svg_with_quicklook, sips_resize,
};

/// Render SVG with QuickLook and strictly mask outer border to transparent alpha.
pub fn render_and_mask(
    svg_path: &Path,
    temp_dir: &Path,
    shadow: bool,
) -> Result<PathBuf, InfraError> {
    let rendered_png = render_svg_with_quicklook(svg_path, temp_dir)?;
    let masked_png = temp_dir.join("masked.png");

    mask_png_with_swift(&rendered_png, &masked_png, temp_dir, shadow)?;
    Ok(masked_png)
}

/// Construct multi-resolution iconset and compile into .icns bundle.
pub fn compile_icns(masked_png: &Path, out_icns: &Path, temp_dir: &Path) -> Result<(), InfraError> {
    let iconset_dir = temp_dir.join("App.iconset");
    fs::create_dir_all(&iconset_dir).map_err(InfraError::Io)?;

    let sizes: &[(u32, &str)] = &[
        (16, "icon_16x16.png"),
        (32, "icon_16x16@2x.png"),
        (32, "icon_32x32.png"),
        (64, "icon_32x32@2x.png"),
        (128, "icon_128x128.png"),
        (256, "icon_128x128@2x.png"),
        (256, "icon_256x256.png"),
        (512, "icon_256x256@2x.png"),
        (512, "icon_512x512.png"),
        (1024, "icon_512x512@2x.png"),
    ];

    for &(sz, filename) in sizes {
        let dest = iconset_dir.join(filename);
        sips_resize(masked_png, &dest, sz)?;
    }

    if let Some(parent) = out_icns.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }

    iconutil_compile(&iconset_dir, out_icns)?;
    Ok(())
}
