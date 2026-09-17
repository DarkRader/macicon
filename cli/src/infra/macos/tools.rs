//! Low-level wrappers around macOS subsystem utilities (qlmanage, swift, sips, iconutil).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::domain::geometry::{CANVAS_SIZE, COCOA_TILE_Y, CORNER_RADIUS, TILE_SIZE, TILE_X};
use crate::error::InfraError;

/// Render an SVG file to a PNG using macOS QuickLook generator (qlmanage).
pub fn render_svg_with_quicklook(svg_path: &Path, temp_dir: &Path) -> Result<PathBuf, InfraError> {
    let output = Command::new("qlmanage")
        .args([
            "-t",
            "-s",
            &CANVAS_SIZE.to_string(),
            "-o",
            temp_dir.to_str().unwrap_or(""),
            svg_path.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| InfraError::ToolExecution {
            tool: "qlmanage".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(InfraError::ToolExecution {
            tool: "qlmanage".to_string(),
            message: "Failed to render SVG to PNG".to_string(),
        });
    }

    let file_name = svg_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("composed.svg");
    let rendered_png = temp_dir.join(format!("{}.png", file_name));

    if !rendered_png.exists() {
        return Err(InfraError::ToolExecution {
            tool: "qlmanage".to_string(),
            message: format!(
                "Expected output file '{}' was not found",
                rendered_png.display()
            ),
        });
    }

    Ok(rendered_png)
}

/// Execute native Cocoa Swift script to apply Apple squircle clipping and drop shadows.
pub fn mask_png_with_swift(
    rendered_png: &Path,
    masked_png: &Path,
    temp_dir: &Path,
    shadow: bool,
) -> Result<(), InfraError> {
    let swift_script_path = temp_dir.join("mask.swift");
    let swift_code = format!(
        r#"import Cocoa

let srcURL = URL(fileURLWithPath: CommandLine.arguments[1])
let outURL = URL(fileURLWithPath: CommandLine.arguments[2])
let hasShadow = (CommandLine.arguments.count > 3 && CommandLine.arguments[3] == "1")

guard let rawImg = NSImage(contentsOf: srcURL) else {{ fatalError("Raw image load failed") }}

let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil,
    pixelsWide: {canv},
    pixelsHigh: {canv},
    bitsPerSample: 8,
    samplesPerPixel: 4,
    hasAlpha: true,
    isPlanar: false,
    colorSpaceName: NSColorSpaceName.deviceRGB,
    bytesPerRow: 0,
    bitsPerPixel: 0
)!

let gCtx = NSGraphicsContext(bitmapImageRep: rep)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = gCtx
let ctx = gCtx.cgContext
ctx.clear(CGRect(x: 0, y: 0, width: {canv}, height: {canv}))

let squircleRect = CGRect(x: {tile_x}, y: {cocoa_y}, width: {tile_sz}, height: {tile_sz})

if hasShadow {{
    ctx.saveGState()
    ctx.setShadow(offset: CGSize(width: 0, height: -16), blur: 18, color: CGColor(red: 0, green: 0, blue: 0, alpha: 0.24))
    let shadowPath = CGPath(roundedRect: squircleRect, cornerWidth: {corner_rad}, cornerHeight: {corner_rad}, transform: nil)
    ctx.addPath(shadowPath)
    ctx.fillPath()
    ctx.restoreGState()
}}

ctx.saveGState()
let clipPath = CGPath(roundedRect: squircleRect, cornerWidth: {corner_rad}, cornerHeight: {corner_rad}, transform: nil)
ctx.addPath(clipPath)
ctx.clip()

rawImg.draw(in: NSRect(x: 0, y: 0, width: {canv}, height: {canv}))
ctx.restoreGState()

NSGraphicsContext.restoreGraphicsState()

let data = rep.representation(using: NSBitmapImageRep.FileType.png, properties: [:])!
try! data.write(to: outURL)
"#,
        canv = CANVAS_SIZE,
        tile_x = TILE_X,
        cocoa_y = COCOA_TILE_Y,
        tile_sz = TILE_SIZE,
        corner_rad = CORNER_RADIUS
    );

    fs::write(&swift_script_path, swift_code).map_err(InfraError::Io)?;

    let shadow_arg = if shadow { "1" } else { "0" };
    let mask_status = Command::new("swift")
        .args([
            swift_script_path.to_str().unwrap_or(""),
            rendered_png.to_str().unwrap_or(""),
            masked_png.to_str().unwrap_or(""),
            shadow_arg,
        ])
        .status()
        .map_err(|e| InfraError::ToolExecution {
            tool: "swift".to_string(),
            message: e.to_string(),
        })?;

    if !mask_status.success() {
        return Err(InfraError::ToolExecution {
            tool: "swift".to_string(),
            message: "Swift squircle masking script failed".to_string(),
        });
    }

    Ok(())
}

/// Resize an image using sips.
pub fn sips_resize(src_png: &Path, dest_png: &Path, size: u32) -> Result<(), InfraError> {
    let status = Command::new("sips")
        .args([
            "-z",
            &size.to_string(),
            &size.to_string(),
            src_png.to_str().unwrap_or(""),
            "--out",
            dest_png.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| InfraError::ToolExecution {
            tool: "sips".to_string(),
            message: e.to_string(),
        })?;

    if !status.status.success() {
        return Err(InfraError::ToolExecution {
            tool: "sips".to_string(),
            message: format!("Failed resizing to {}x{}", size, size),
        });
    }

    Ok(())
}

/// Convert an .icns file to a PNG preview using sips.
pub fn sips_convert_icns_to_png(icns_path: &Path, preview_path: &Path) -> Result<(), InfraError> {
    let output = Command::new("sips")
        .args([
            "-s",
            "format",
            "png",
            icns_path.to_str().unwrap_or(""),
            "--out",
            preview_path.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| InfraError::ToolExecution {
            tool: "sips".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(InfraError::ToolExecution {
            tool: "sips".to_string(),
            message: "Failed to convert ICNS to PNG preview".to_string(),
        });
    }

    Ok(())
}

/// Compile a `.iconset` directory into a `.icns` file using iconutil.
pub fn iconutil_compile(iconset_dir: &Path, out_icns: &Path) -> Result<(), InfraError> {
    let status = Command::new("iconutil")
        .args([
            "-c",
            "icns",
            iconset_dir.to_str().unwrap_or(""),
            "-o",
            out_icns.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| InfraError::ToolExecution {
            tool: "iconutil".to_string(),
            message: e.to_string(),
        })?;

    if !status.success() {
        return Err(InfraError::ToolExecution {
            tool: "iconutil".to_string(),
            message: "iconutil failed to compile .iconset to .icns".to_string(),
        });
    }

    Ok(())
}

/// Touch a file or app bundle to invalidate Finder cache.
pub fn touch_path(path: &Path) -> Result<(), InfraError> {
    let _ = Command::new("touch")
        .arg(path.to_str().unwrap_or(""))
        .status();
    Ok(())
}
