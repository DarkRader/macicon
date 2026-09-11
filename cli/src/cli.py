"""Command-line interface and dispatch for macicon."""

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

from app_icon import apply_icon_to_app
from fetcher import extract_path_from_svg, fetch_icon_or_create
from renderer import generate_single_icon
from themes import (
    THEME_PRESETS,
    compute_styling,
    get_icons_base_dir,
    load_themes_manifest,
    save_themes_manifest,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="macicon",
        description="Generate, customize, and apply Apple continuous-curvature squircle app icons on macOS.",
    )
    # Source options
    source_group = parser.add_mutually_exclusive_group(required=True)
    source_group.add_argument(
        "--query",
        "-q",
        help="Search term or link from Simple Icons (e.g. 'slack', 'https://simpleicons.org/?q=warp', or direct SVG link)",
    )
    source_group.add_argument("--svg", "-s", help="Path to a local SVG file")
    source_group.add_argument("--path", "-p", help="Direct SVG path string d='...'")
    source_group.add_argument(
        "--letter",
        "-l",
        help="Generate an Apple-style typography lettermark monogram (e.g. 'S', 'G', 'AI')",
    )
    source_group.add_argument(
        "--icns",
        help="Path to an existing .icns file (typically used with --apply to apply an existing icon to an app)",
    )
    source_group.add_argument(
        "--create-theme",
        metavar="THEME_NAME",
        help="Batch generate all existing icons into a new theme folder under <icons_dir>/<THEME_NAME>",
    )
    source_group.add_argument(
        "--sync-themes",
        action="store_true",
        help="Sync and regenerate missing icons across all themes registered in themes.json",
    )

    parser.add_argument(
        "--fallback-letter",
        action="store_true",
        help="If --query is not found online, automatically fall back to an Apple lettermark monogram",
    )
    parser.add_argument(
        "--all-themes",
        action="store_true",
        help="When generating a single icon (--query, --svg, etc.), generate it for all registered themes in themes.json",
    )
    parser.add_argument(
        "--from-theme",
        default="light",
        help="Reference theme to discover icons from when using --create-theme (default: 'light')",
    )
    parser.add_argument(
        "--icons-dir",
        help="Base directory containing theme folders (default: './icons' or auto-detected 'nix/icons')",
    )

    # Style options
    parser.add_argument(
        "--theme",
        "-t",
        choices=list(THEME_PRESETS.keys()),
        default="light",
        help="Base theme preset (default: light)",
    )
    parser.add_argument(
        "--bg",
        "-b",
        help="Background color/gradient: 'white', 'dark', '#FFFFFF', or gradient '#FFFFFF,#EBECEF' (default: theme default)",
    )
    parser.add_argument(
        "--color",
        "-c",
        help="Symbol color: 'black', 'white', hex '#202022', or gradient '#00C8FF,#0072FE' (default: theme default)",
    )
    parser.add_argument("--border-color", help="Custom tile border color (hex)")
    parser.add_argument(
        "--shadow",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Enable or disable drop shadow on the symbol (default: --shadow, use --no-shadow for flat)",
    )
    parser.add_argument(
        "--scale",
        type=float,
        default=1.25,
        help="Scale multiplier for the symbol (default: 1.25, tuned for macOS Dock)",
    )
    parser.add_argument(
        "--viewbox",
        "-v",
        default="0 0 24 24",
        help="SVG viewBox when using --path (default: '0 0 24 24')",
    )

    # Output & Action options
    parser.add_argument(
        "--out",
        "-o",
        help="Destination path for .icns (optional: auto-derived from query or app name)",
    )
    parser.add_argument(
        "--preview",
        nargs="?",
        const=True,
        default=None,
        help="Generate a 1024x1024 PNG preview (optional custom path, or auto /tmp/<name>-preview.png)",
    )
    parser.add_argument(
        "--apply", "-a", help="Target .app bundle path or app name to immediately apply the icon to"
    )
    parser.add_argument(
        "--no-dock-restart",
        action="store_true",
        help="Do not restart the Dock after applying the icon",
    )

    return parser.parse_args()


def handle_create_theme(args: argparse.Namespace) -> None:
    theme_name = args.create_theme.strip().lower()
    icons_base_path = Path(get_icons_base_dir(args.icons_dir))
    target_dir = icons_base_path / theme_name
    target_dir.mkdir(parents=True, exist_ok=True)

    ref_theme = args.from_theme or "light"
    ref_dir = icons_base_path / ref_theme
    if not ref_dir.is_dir():
        candidates = ["light", "dark"] + [
            p.name for p in icons_base_path.iterdir() if p.is_dir() and not p.name.startswith(".")
        ]
        for c in candidates:
            c_dir = icons_base_path / c
            if c_dir.is_dir() and any(f.suffix == ".icns" for f in c_dir.iterdir()):
                ref_dir = c_dir
                ref_theme = c
                break

    if not ref_dir.is_dir():
        sys.exit(f"⚠️  Error: Could not find reference theme directory under '{icons_base_path}'.")

    existing_icons = sorted(
        [
            f.stem
            for f in ref_dir.iterdir()
            if f.is_file() and f.suffix == ".icns" and not f.name.startswith(".")
        ]
    )

    if not existing_icons:
        sys.exit(f"⚠️  Error: No .icns icons found in reference theme '{ref_theme}' ({ref_dir}).")

    if theme_name in THEME_PRESETS and not args.bg and args.theme == "light":
        args.theme = theme_name

    bg_top, bg_bottom, border, symbol_color = compute_styling(
        args.theme, args.bg, args.color, args.border_color
    )
    scale = args.scale
    shadow = args.shadow

    bg_display = bg_top if bg_top == bg_bottom else f"{bg_top} -> {bg_bottom}"
    print(f"\n🎨 Creating new icon theme '{theme_name}' with {len(existing_icons)} icons:")
    print(f"   Target Directory : {target_dir}")
    print(f"   Reference Theme  : {ref_theme} ({len(existing_icons)} icons)")
    print(f"   Background       : {bg_display}")
    print(f"   Border           : {border}")
    print(f"   Symbol Color     : {symbol_color}")
    print(f"   Shadow           : {'Enabled' if shadow else 'Disabled (flat)'}")
    print(f"   Scale            : {scale}\n")

    themes_file = icons_base_path / "themes.json"
    themes_data = load_themes_manifest(str(themes_file), str(icons_base_path))
    themes_data[theme_name] = {
        "bg_top": bg_top,
        "bg_bottom": bg_bottom,
        "border": border,
        "symbol_color": symbol_color,
        "shadow": shadow,
        "scale": scale,
    }
    save_themes_manifest(str(themes_file), themes_data)

    success_count = 0
    failed_icons = []

    for i, icon_name in enumerate(existing_icons, start=1):
        print(
            f"   [{i:2d}/{len(existing_icons)}] Generating {icon_name}.icns ... ",
            end="",
            flush=True,
        )
        out_file = target_dir / f"{icon_name}.icns"
        try:
            icon_info = fetch_icon_or_create(icon_name, fallback_letter=True)
            generate_single_icon(
                icon_info=icon_info,
                out_icns=str(out_file),
                bg_top=bg_top,
                bg_bottom=bg_bottom,
                border=border,
                symbol_color=symbol_color,
                scale=scale,
                shadow=shadow,
            )
            print("✓")
            success_count += 1
        except Exception as e:  # noqa: BLE001
            print(f"✗ ({e})")
            failed_icons.append((icon_name, str(e)))

    print(
        f"\n✨ Theme '{theme_name}' successfully generated ({success_count}/{len(existing_icons)} icons in '{target_dir}')."
    )
    if failed_icons:
        print(f"⚠️  {len(failed_icons)} icons failed: {', '.join(k for k, _ in failed_icons)}")


def handle_all_themes(args: argparse.Namespace, icon_info: dict[str, str], base_name: str) -> None:
    icons_base_path = Path(get_icons_base_dir(args.icons_dir))
    themes_file = icons_base_path / "themes.json"
    themes_data = load_themes_manifest(str(themes_file), str(icons_base_path))

    print(f"\n🌐 Generating icon '{base_name}.icns' across {len(themes_data)} registered theme(s):")
    success_count = 0
    for theme_name, cfg in themes_data.items():
        theme_dir = icons_base_path / theme_name
        theme_dir.mkdir(parents=True, exist_ok=True)
        out_file = theme_dir / f"{base_name}.icns"

        bg_top = str(cfg.get("bg_top", "#FFFFFF"))
        bg_bottom = str(cfg.get("bg_bottom", bg_top))
        border = str(cfg.get("border", "#D8D9DC"))
        symbol_color = str(cfg.get("symbol_color", "#202022"))
        scale = float(str(cfg.get("scale", args.scale)))
        shadow = bool(cfg.get("shadow", True))

        print(f"   • {theme_name:<14} -> {theme_name}/{base_name}.icns ... ", end="", flush=True)
        try:
            generate_single_icon(
                icon_info=icon_info,
                out_icns=str(out_file),
                bg_top=bg_top,
                bg_bottom=bg_bottom,
                border=border,
                symbol_color=symbol_color,
                scale=scale,
                shadow=shadow,
            )
            print("✓")
            success_count += 1
        except Exception as e:  # noqa: BLE001
            print(f"✗ ({e})")

    print(
        f"\n✨ Successfully generated '{base_name}.icns' across {success_count}/{len(themes_data)} themes.\n"
    )


def handle_sync_themes(args: argparse.Namespace) -> None:
    icons_base_path = Path(get_icons_base_dir(args.icons_dir))
    themes_file = icons_base_path / "themes.json"
    themes_data = load_themes_manifest(str(themes_file), str(icons_base_path))

    all_icon_names = set()
    for theme_name in themes_data:
        t_dir = icons_base_path / theme_name
        if t_dir.is_dir():
            for f in t_dir.iterdir():
                if f.is_file() and f.suffix == ".icns" and not f.name.startswith("."):
                    all_icon_names.add(f.stem)

    if not all_icon_names:
        sys.exit(f"⚠️  Error: No .icns icons found across any themes in '{icons_base_path}'.")

    all_icons_list = sorted(all_icon_names)
    print(
        f"\n🔄 Syncing {len(all_icons_list)} icons across {len(themes_data)} themes ({', '.join(themes_data.keys())})...\n"
    )

    for theme_name, cfg in themes_data.items():
        print(f"📦 Theme: {theme_name}")
        t_dir = icons_base_path / theme_name
        t_dir.mkdir(parents=True, exist_ok=True)
        bg_top = str(cfg.get("bg_top", "#FFFFFF"))
        bg_bottom = str(cfg.get("bg_bottom", bg_top))
        border = str(cfg.get("border", "#D8D9DC"))
        symbol_color = str(cfg.get("symbol_color", "#202022"))
        scale = float(str(cfg.get("scale", 1.25)))
        shadow = bool(cfg.get("shadow", True))

        for icon_name in all_icons_list:
            out_file = t_dir / f"{icon_name}.icns"
            if not out_file.exists():
                print(f"   Adding missing {icon_name}.icns ... ", end="", flush=True)
                try:
                    icon_info = fetch_icon_or_create(icon_name, fallback_letter=True)
                    generate_single_icon(
                        icon_info=icon_info,
                        out_icns=str(out_file),
                        bg_top=bg_top,
                        bg_bottom=bg_bottom,
                        border=border,
                        symbol_color=symbol_color,
                        scale=scale,
                        shadow=shadow,
                    )
                    print("✓")
                except Exception as e:  # noqa: BLE001
                    print(f"✗ ({e})")
    print("\n✨ All themes are now synchronized!\n")


def main() -> None:
    args = parse_args()

    if args.create_theme:
        handle_create_theme(args)
        return

    if args.sync_themes:
        handle_sync_themes(args)
        return

    if args.icns:
        icns_path = Path(args.icns).expanduser().resolve()
        if not icns_path.exists():
            sys.exit(f"⚠️  Error: .icns file not found at '{icns_path}'")

        base_name = icns_path.stem

        if args.preview:
            icns_preview = (
                Path(f"/tmp/{base_name}-preview.png")
                if args.preview is True
                else Path(args.preview).expanduser().resolve()
            )
            subprocess.run(
                ["sips", "-s", "format", "png", str(icns_path), "--out", str(icns_preview)],
                check=True,
                stdout=subprocess.DEVNULL,
            )
            print(f"Saved PNG preview: {icns_preview}")

        if args.out:
            out_dest = Path(args.out).expanduser().resolve()
            out_dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(icns_path, out_dest)
            print(f"Copied icon to: {out_dest}")

        if args.apply:
            apply_icon_to_app(args.apply, str(icns_path), restart_dock=not args.no_dock_restart)
        elif not args.preview and not args.out:
            print(
                f"Icon verified at '{icns_path}'. Pass --apply <app_path> to apply it to an application."
            )
        return

    # Determine input source
    if args.query:
        icon_info = fetch_icon_or_create(args.query, fallback_letter=args.fallback_letter)
        base_name = icon_info.get("name") or "custom"
    elif args.letter:
        icon_info = {"type": "letter", "letter": args.letter}
        base_name = f"letter-{args.letter.lower()}"
    elif args.svg:
        svg_path = Path(args.svg)
        with svg_path.open(encoding="utf-8") as f:
            svg_content = f.read()
        path_d, viewbox, fill_rule = extract_path_from_svg(svg_content)
        icon_info = {"type": "path", "path_d": path_d, "viewbox": viewbox, "fill_rule": fill_rule}
        base_name = svg_path.stem.lower()
    else:
        icon_info = {
            "type": "path",
            "path_d": args.path,
            "viewbox": args.viewbox,
            "fill_rule": "evenodd",
        }
        base_name = "custom"

    if args.all_themes:
        handle_all_themes(args, icon_info, base_name)
        return

    # Automatically derive destination path if omitted
    if not args.out:
        icons_dir = Path(get_icons_base_dir(args.icons_dir))
        theme_dir = icons_dir / args.theme
        if theme_dir.is_dir():
            args.out = str(theme_dir / f"{base_name}.icns")
        elif args.apply:
            app_stem = Path(args.apply).stem.lower().replace(" ", "-")
            args.out = f"/tmp/{app_stem}.icns"
        else:
            args.out = f"./{base_name}.icns"

    preview_path: str | None = None
    if args.preview is True:
        preview_path = f"/tmp/{base_name}-preview.png"
    elif isinstance(args.preview, str):
        preview_path = str(Path(args.preview).expanduser().resolve())

    bg_top, bg_bottom, border, symbol_color = compute_styling(
        args.theme, args.bg, args.color, args.border_color
    )

    out_icns = generate_single_icon(
        icon_info=icon_info,
        out_icns=args.out,
        bg_top=bg_top,
        bg_bottom=bg_bottom,
        border=border,
        symbol_color=symbol_color,
        scale=args.scale,
        shadow=args.shadow,
        preview_path=preview_path,
    )

    if args.apply:
        apply_icon_to_app(args.apply, out_icns, restart_dock=not args.no_dock_restart)


if __name__ == "__main__":
    main()
