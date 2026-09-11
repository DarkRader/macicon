//! SVG generation, image masking, and .icns bundle compilation for macicon.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

use crate::constants::{
    CANVAS_SIZE, CORNER_RADIUS, DEFAULT_SYMBOL_SIZE, TILE_SIZE, TILE_X, TILE_Y,
};
use crate::fetcher::IconInfo;
use crate::themes::IconStyle;

/// Generate standard Apple HIG squircle SVG markup with embedded vector symbol.
pub fn build_svg(
    path_d: &str,
    viewbox: &str,
    style: &IconStyle,
    scale: f64,
    fill_rule: &str,
    shadow: bool,
) -> String {
    let vb_cleaned = viewbox.replace(',', " ");
    let vb_parts: Vec<f64> = vb_cleaned
        .split_whitespace()
        .filter_map(|x| x.parse::<f64>().ok())
        .collect();

    let (min_x, min_y, vb_w, vb_h) = if vb_parts.len() == 4 {
        (vb_parts[0], vb_parts[1], vb_parts[2], vb_parts[3])
    } else if vb_parts.len() == 2 {
        (0.0, 0.0, vb_parts[0], vb_parts[1])
    } else {
        (0.0, 0.0, 24.0, 24.0)
    };

    let center_x = min_x + (vb_w / 2.0);
    let center_y = min_y + (vb_h / 2.0);
    let max_dim = vb_w.max(vb_h).max(1.0);
    let calc_scale = (DEFAULT_SYMBOL_SIZE / max_dim) * scale;

    let filter_attr = if shadow {
        r#" filter="url(#symbol-shadow)""#
    } else {
        ""
    };

    let mut sym_grad_def = String::new();
    let fill_attr: String;

    if style.symbol_color.contains(',') {
        let parts: Vec<&str> = style.symbol_color.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            sym_grad_def = format!(
                r#"    <linearGradient id="symbol-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{}"/>
      <stop offset="100%" stop-color="{}"/>
    </linearGradient>"#,
                parts[0], parts[1]
            );
            fill_attr = "url(#symbol-grad)".to_string();
        } else {
            fill_attr = style.symbol_color.clone();
        }
    } else {
        fill_attr = style.symbol_color.clone();
    }

    format!(
        r##"<svg width="{canv}" height="{canv}" viewBox="0 0 {canv} {canv}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <filter id="symbol-shadow" x="-50%" y="-50%" width="200%" height="200%">
      <feDropShadow dx="0" dy="4" stdDeviation="6" flood-color="#000000" flood-opacity="0.18"/>
      <feDropShadow dx="0" dy="1.5" stdDeviation="2" flood-color="#000000" flood-opacity="0.12"/>
    </filter>
    <linearGradient id="bg-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{bg_top}"/>
      <stop offset="100%" stop-color="{bg_bottom}"/>
    </linearGradient>
{sym_grad_def}
  </defs>

  <!-- Base Squircle -->
  <rect x="{tile_x}" y="{tile_y}" width="{tile_sz}" height="{tile_sz}" rx="{corner_rad}" fill="url(#bg-grad)" stroke="{border}" stroke-width="1.5"/>

  <!-- Centered Symbol -->
  <g transform="translate(512, 504) scale({calc_scale}) translate({neg_cx}, {neg_cy})"{filter_attr}>
    <path fill="{fill_attr}" fill-rule="{fill_rule}" d="{path_d}"/>
  </g>
</svg>"##,
        canv = CANVAS_SIZE,
        bg_top = style.bg_top,
        bg_bottom = style.bg_bottom,
        tile_x = TILE_X,
        tile_y = TILE_Y,
        tile_sz = TILE_SIZE,
        corner_rad = CORNER_RADIUS,
        border = style.border,
        calc_scale = calc_scale,
        neg_cx = -center_x,
        neg_cy = -center_y,
        filter_attr = filter_attr,
        fill_attr = fill_attr,
        fill_rule = fill_rule,
        path_d = path_d,
        sym_grad_def = sym_grad_def,
    )
}

/// Generate an Apple-style typography monogram lettermark SVG.
pub fn build_letter_svg(letter: &str, style: &IconStyle, scale: f64, shadow: bool) -> String {
    let font_size = if letter.chars().count() == 1 {
        (400.0 * scale) as u32
    } else {
        (280.0 * scale) as u32
    };
    let y_pos = if letter.chars().count() == 1 {
        635
    } else {
        600
    };
    let filter_attr = if shadow {
        r#" filter="url(#symbol-shadow)""#
    } else {
        ""
    };

    let mut sym_grad_def = String::new();
    let fill_attr: String;

    if style.symbol_color.contains(',') {
        let parts: Vec<&str> = style.symbol_color.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            sym_grad_def = format!(
                r#"    <linearGradient id="symbol-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{}"/>
      <stop offset="100%" stop-color="{}"/>
    </linearGradient>"#,
                parts[0], parts[1]
            );
            fill_attr = "url(#symbol-grad)".to_string();
        } else {
            fill_attr = style.symbol_color.clone();
        }
    } else {
        fill_attr = style.symbol_color.clone();
    }

    format!(
        r##"<svg width="{canv}" height="{canv}" viewBox="0 0 {canv} {canv}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <filter id="symbol-shadow" x="-50%" y="-50%" width="200%" height="200%">
      <feDropShadow dx="0" dy="4" stdDeviation="6" flood-color="#000000" flood-opacity="0.18"/>
      <feDropShadow dx="0" dy="1.5" stdDeviation="2" flood-color="#000000" flood-opacity="0.12"/>
    </filter>
    <linearGradient id="bg-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{bg_top}"/>
      <stop offset="100%" stop-color="{bg_bottom}"/>
    </linearGradient>
{sym_grad_def}
  </defs>

  <!-- Base Squircle -->
  <rect x="{tile_x}" y="{tile_y}" width="{tile_sz}" height="{tile_sz}" rx="{corner_rad}" fill="url(#bg-grad)" stroke="{border}" stroke-width="1.5"/>

  <!-- Centered Lettermark -->
  <text x="512" y="{y_pos}" font-family="-apple-system, 'SF Pro Display', system-ui, sans-serif" font-size="{font_size}" font-weight="800" text-anchor="middle" fill="{fill_attr}"{filter_attr}>{letter}</text>
</svg>"##,
        canv = CANVAS_SIZE,
        bg_top = style.bg_top,
        bg_bottom = style.bg_bottom,
        tile_x = TILE_X,
        tile_y = TILE_Y,
        tile_sz = TILE_SIZE,
        corner_rad = CORNER_RADIUS,
        border = style.border,
        letter = letter,
        y_pos = y_pos,
        font_size = font_size,
        filter_attr = filter_attr,
        fill_attr = fill_attr,
        sym_grad_def = sym_grad_def,
    )
}

/// Render SVG with QuickLook and strictly mask outer border to transparent alpha.
pub fn render_and_mask(svg_path: &Path, temp_dir: &Path, shadow: bool) -> Result<PathBuf, String> {
    let status = Command::new("qlmanage")
        .args([
            "-t",
            "-s",
            "1024",
            "-o",
            temp_dir.to_str().unwrap(),
            svg_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run qlmanage: {}", e))?;

    if !status.status.success() {
        return Err("qlmanage failed to render SVG".to_string());
    }

    let file_name = svg_path.file_name().unwrap().to_str().unwrap();
    let rendered_png = temp_dir.join(format!("{}.png", file_name));
    let masked_png = temp_dir.join("masked.png");

    let swift_script_path = temp_dir.join("mask.swift");
    let swift_code = r#"import Cocoa

let srcURL = URL(fileURLWithPath: CommandLine.arguments[1])
let outURL = URL(fileURLWithPath: CommandLine.arguments[2])
let hasShadow = (CommandLine.arguments.count > 3 && CommandLine.arguments[3] == "1")

guard let rawImg = NSImage(contentsOf: srcURL) else { fatalError("Raw image load failed") }

let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil,
    pixelsWide: 1024,
    pixelsHigh: 1024,
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
ctx.clear(CGRect(x: 0, y: 0, width: 1024, height: 1024))

let squircleRect = CGRect(x: 96, y: 104, width: 832, height: 832)

if hasShadow {
    ctx.saveGState()
    ctx.setShadow(offset: CGSize(width: 0, height: -16), blur: 18, color: CGColor(red: 0, green: 0, blue: 0, alpha: 0.24))
    let shadowPath = CGPath(roundedRect: squircleRect, cornerWidth: 185, cornerHeight: 185, transform: nil)
    ctx.addPath(shadowPath)
    ctx.fillPath()
    ctx.restoreGState()
}

ctx.saveGState()
let clipPath = CGPath(roundedRect: squircleRect, cornerWidth: 185, cornerHeight: 185, transform: nil)
ctx.addPath(clipPath)
ctx.clip()

rawImg.draw(in: NSRect(x: 0, y: 0, width: 1024, height: 1024))
ctx.restoreGState()

NSGraphicsContext.restoreGraphicsState()

let data = rep.representation(using: NSBitmapImageRep.FileType.png, properties: [:])!
try! data.write(to: outURL)
"#;
    fs::write(&swift_script_path, swift_code)
        .map_err(|e| format!("Failed to write mask.swift: {}", e))?;

    let shadow_arg = if shadow { "1" } else { "0" };
    let mask_status = Command::new("swift")
        .args([
            swift_script_path.to_str().unwrap(),
            rendered_png.to_str().unwrap(),
            masked_png.to_str().unwrap(),
            shadow_arg,
        ])
        .status()
        .map_err(|e| format!("Failed to run swift mask script: {}", e))?;

    if !mask_status.success() {
        return Err("Swift masking script failed".to_string());
    }

    Ok(masked_png)
}

/// Construct multi-resolution iconset and compile into .icns bundle.
pub fn compile_icns(masked_png: &Path, out_icns: &Path, temp_dir: &Path) -> Result<(), String> {
    let iconset_dir = temp_dir.join("App.iconset");
    fs::create_dir_all(&iconset_dir).map_err(|e| format!("Failed to create iconset dir: {}", e))?;

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
        let sips_status = Command::new("sips")
            .args([
                "-z",
                &sz.to_string(),
                &sz.to_string(),
                masked_png.to_str().unwrap(),
                "--out",
                dest.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to execute sips: {}", e))?;

        if !sips_status.status.success() {
            return Err(format!("sips failed for size {}", sz));
        }
    }

    if let Some(parent) = out_icns.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent dir: {}", e))?;
    }

    let iconutil_status = Command::new("iconutil")
        .args([
            "-c",
            "icns",
            iconset_dir.to_str().unwrap(),
            "-o",
            out_icns.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format!("Failed to execute iconutil: {}", e))?;

    if !iconutil_status.success() {
        return Err("iconutil failed to compile icns".to_string());
    }

    println!("Compiled ICNS: {}", out_icns.display());
    Ok(())
}

/// Generate a complete squircle .icns package from vector info.
pub fn generate_single_icon(
    icon_info: &IconInfo,
    out_icns: &Path,
    style: &IconStyle,
    scale: f64,
    shadow: bool,
    preview_path: Option<&Path>,
) -> Result<PathBuf, String> {
    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
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
    fs::write(&svg_file, &svg_markup).map_err(|e| format!("Failed to write SVG: {}", e))?;

    let masked_png = render_and_mask(&svg_file, t_dir, shadow)?;

    if let Some(prev) = preview_path {
        if let Some(parent) = prev.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::copy(&masked_png, prev).map_err(|e| format!("Failed to copy preview PNG: {}", e))?;
        println!("Saved PNG preview: {}", prev.display());
    }

    compile_icns(&masked_png, out_icns, t_dir)?;
    Ok(out_icns.to_path_buf())
}
