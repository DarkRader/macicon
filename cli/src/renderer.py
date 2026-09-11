"""SVG composition, Apple squircle masking, and ICNS compilation for macicon."""

import shutil
import subprocess
import tempfile
from pathlib import Path

from constants import (
    CANVAS_SIZE,
    CORNER_RADIUS,
    DEFAULT_SYMBOL_SIZE,
    TILE_SIZE,
    TILE_X,
    TILE_Y,
)

VIEWBOX_4_TUPLE = 4
VIEWBOX_2_TUPLE = 2


def build_svg(
    path_d: str,
    viewbox: str,
    bg_top: str,
    bg_bottom: str,
    border: str,
    symbol_color: str | tuple[str, str],
    scale: float,
    *,
    fill_rule: str = "evenodd",
    shadow: bool = True,
) -> str:
    """Generate standard Apple HIG squircle SVG markup with embedded symbol."""
    vb_parts = [float(x) for x in viewbox.replace(",", " ").split() if x]
    if len(vb_parts) == VIEWBOX_4_TUPLE:
        min_x, min_y, vb_w, vb_h = vb_parts
    elif len(vb_parts) == VIEWBOX_2_TUPLE:
        min_x, min_y, vb_w, vb_h = 0.0, 0.0, vb_parts[0], vb_parts[1]
    else:
        min_x, min_y, vb_w, vb_h = 0.0, 0.0, 24.0, 24.0

    center_x = min_x + (vb_w / 2.0)
    center_y = min_y + (vb_h / 2.0)
    max_dim = max(vb_w, vb_h)
    calc_scale = (DEFAULT_SYMBOL_SIZE / max_dim) * scale

    filter_attr = ' filter="url(#symbol-shadow)"' if shadow else ""

    # Support gradient symbol fill
    sym_grad_def = ""
    fill_attr = symbol_color
    if isinstance(symbol_color, (list, tuple)):
        sym_top, sym_bottom = symbol_color[0], symbol_color[1]
    elif "," in str(symbol_color):
        parts = [p.strip() for p in str(symbol_color).split(",")]
        sym_top, sym_bottom = parts[0], parts[1]
    else:
        sym_top, sym_bottom = None, None

    if sym_top and sym_bottom:
        sym_grad_def = f"""    <linearGradient id="symbol-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{sym_top}"/>
      <stop offset="100%" stop-color="{sym_bottom}"/>
    </linearGradient>"""
        fill_attr = "url(#symbol-grad)"

    return f"""<svg width="{CANVAS_SIZE}" height="{CANVAS_SIZE}" viewBox="0 0 {CANVAS_SIZE} {CANVAS_SIZE}" xmlns="http://www.w3.org/2000/svg">
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
  <rect x="{TILE_X}" y="{TILE_Y}" width="{TILE_SIZE}" height="{TILE_SIZE}" rx="{CORNER_RADIUS}" fill="url(#bg-grad)" stroke="{border}" stroke-width="1.5"/>

  <!-- Centered Symbol -->
  <g transform="translate(512, 504) scale({calc_scale}) translate({-center_x}, {-center_y})"{filter_attr}>
    <path fill="{fill_attr}" fill-rule="{fill_rule}" d="{path_d}"/>
  </g>
</svg>"""


def build_letter_svg(
    letter: str,
    bg_top: str,
    bg_bottom: str,
    border: str,
    symbol_color: str | tuple[str, str],
    scale: float,
    *,
    shadow: bool = True,
) -> str:
    """Generate an Apple-style typography monogram lettermark SVG."""
    font_size = int(400 * scale) if len(letter) == 1 else int(280 * scale)
    y_pos = 635 if len(letter) == 1 else 600
    filter_attr = ' filter="url(#symbol-shadow)"' if shadow else ""

    sym_grad_def = ""
    fill_attr = symbol_color
    if isinstance(symbol_color, (list, tuple)):
        sym_top, sym_bottom = symbol_color[0], symbol_color[1]
    elif "," in str(symbol_color):
        parts = [p.strip() for p in str(symbol_color).split(",")]
        sym_top, sym_bottom = parts[0], parts[1]
    else:
        sym_top, sym_bottom = None, None

    if sym_top and sym_bottom:
        sym_grad_def = f"""    <linearGradient id="symbol-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{sym_top}"/>
      <stop offset="100%" stop-color="{sym_bottom}"/>
    </linearGradient>"""
        fill_attr = "url(#symbol-grad)"

    return f"""<svg width="{CANVAS_SIZE}" height="{CANVAS_SIZE}" viewBox="0 0 {CANVAS_SIZE} {CANVAS_SIZE}" xmlns="http://www.w3.org/2000/svg">
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
  <rect x="{TILE_X}" y="{TILE_Y}" width="{TILE_SIZE}" height="{TILE_SIZE}" rx="{CORNER_RADIUS}" fill="url(#bg-grad)" stroke="{border}" stroke-width="1.5"/>

  <!-- Centered Lettermark -->
  <text x="512" y="{y_pos}" font-family="-apple-system, 'SF Pro Display', system-ui, sans-serif" font-size="{font_size}" font-weight="800" text-anchor="middle" fill="{fill_attr}"{filter_attr}>{letter}</text>
</svg>"""


def render_and_mask(svg_path: str, temp_dir: str, *, shadow: bool = True) -> str:
    """Render SVG with QuickLook and strictly mask outer border to transparent alpha."""
    t_dir = Path(temp_dir)
    subprocess.run(
        ["qlmanage", "-t", "-s", "1024", "-o", str(t_dir), svg_path],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    rendered_png = t_dir / f"{Path(svg_path).name}.png"
    masked_png = t_dir / "masked.png"

    swift_script = t_dir / "mask.swift"
    with swift_script.open("w", encoding="utf-8") as f:
        f.write("""import Cocoa

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

// In standard Cocoa coordinates (origin at bottom-left):
// The squircle top margin in SVG is 88px, height is 832px.
// Cocoa coordinates: y = 1024 - 88 - 832 = 104.
let squircleRect = CGRect(x: 96, y: 104, width: 832, height: 832)

if hasShadow {
    ctx.saveGState()
    ctx.setShadow(offset: CGSize(width: 0, height: -16), blur: 18, color: CGColor(red: 0, green: 0, blue: 0, alpha: 0.24))
    let shadowPath = CGPath(roundedRect: squircleRect, cornerWidth: 185, cornerHeight: 185, transform: nil)
    ctx.addPath(shadowPath)
    ctx.fillPath()
    ctx.restoreGState()
}

// Clip strictly to squircle to eliminate QuickLook white background
ctx.saveGState()
let clipPath = CGPath(roundedRect: squircleRect, cornerWidth: 185, cornerHeight: 185, transform: nil)
ctx.addPath(clipPath)
ctx.clip()

rawImg.draw(in: NSRect(x: 0, y: 0, width: 1024, height: 1024))
ctx.restoreGState()

NSGraphicsContext.restoreGraphicsState()

let data = rep.representation(using: NSBitmapImageRep.FileType.png, properties: [:])!
try! data.write(to: outURL)
""")
    shadow_arg = "1" if shadow else "0"
    subprocess.run(
        ["swift", str(swift_script), str(rendered_png), str(masked_png), shadow_arg],
        check=True,
    )
    return str(masked_png)


def compile_icns(masked_png: str, out_icns: str, temp_dir: str) -> None:
    """Construct multi-resolution iconset and compile into .icns bundle."""
    t_dir = Path(temp_dir)
    iconset_dir = t_dir / "App.iconset"
    iconset_dir.mkdir(parents=True, exist_ok=True)

    sizes = [
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
    ]

    for sz, filename in sizes:
        dest = iconset_dir / filename
        subprocess.run(
            ["sips", "-z", str(sz), str(sz), masked_png, "--out", str(dest)],
            check=True,
            stdout=subprocess.DEVNULL,
        )

    target_icns = Path(out_icns).expanduser().resolve()
    target_icns.parent.mkdir(parents=True, exist_ok=True)

    subprocess.run(["iconutil", "-c", "icns", str(iconset_dir), "-o", str(target_icns)], check=True)
    print(f"Compiled ICNS: {target_icns}")


def generate_single_icon(
    icon_info: dict[str, str],
    out_icns: str,
    bg_top: str,
    bg_bottom: str,
    border: str,
    symbol_color: str | tuple[str, str],
    *,
    scale: float = 1.25,
    shadow: bool = True,
    preview_path: str | None = None,
) -> str:
    """Generate a complete squircle .icns package from vector info."""
    temp_dir = tempfile.mkdtemp(prefix="macicon_")
    t_dir = Path(temp_dir)
    try:
        if str(icon_info.get("type")) == "letter":
            svg_markup = build_letter_svg(
                str(icon_info["letter"]),
                bg_top,
                bg_bottom,
                border,
                symbol_color,
                scale,
                shadow=shadow,
            )
        else:
            svg_markup = build_svg(
                str(icon_info["path_d"]),
                str(icon_info["viewbox"]),
                bg_top,
                bg_bottom,
                border,
                symbol_color,
                scale,
                fill_rule=str(icon_info.get("fill_rule", "evenodd")),
                shadow=shadow,
            )

        svg_file = t_dir / "composed.svg"
        with svg_file.open("w", encoding="utf-8") as f:
            f.write(svg_markup)

        masked_png = render_and_mask(str(svg_file), str(t_dir), shadow=shadow)

        if preview_path:
            shutil.copy(masked_png, preview_path)
            print(f"Saved PNG preview: {preview_path}")

        target_icns = Path(out_icns).expanduser().resolve()
        compile_icns(masked_png, str(target_icns), str(t_dir))
        return str(target_icns)
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)
