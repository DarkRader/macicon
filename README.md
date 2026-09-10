# macicon 🍎🎗

[![macOS](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](https://www.apple.com/macos/)
[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![Managed with uv](https://img.shields.io/badge/managed%20with-uv-purple.svg)](https://github.com/astral-sh/uv)

A modern, standalone macOS command-line tool to generate, customize, theme, and apply Apple continuous-curvature squircle application icons (`.icns`).

---

## ✨ Features

- **Apple Human Interface Guidelines (HIG) Squircle Geometry**: Matches macOS Big Sur through Sequoia icon specifications (1024x1024 canvas, 832x832 squircle tile, 185 corner radius, smooth elevation drop shadows).
- **100% Transparent Outer Corners**: Utilizes macOS Cocoa `NSGraphicsContext` clipping to guarantee zero white box artifacts or QuickLook rendering defects.
- **Auto-Fetching from CDN**: Fetches official vectors automatically from [Simple Icons](https://simpleicons.org) and [Dashboard Icons](https://github.com/homarr-labs/dashboard-icons) by name or URL.
- **Safe 1-Step Application**: Applies icons to running macOS `.app` bundles using native `NSWorkspace.setIcon` without modifying application binaries or breaking Apple code signatures.
- **Comprehensive Theming Engine**: Comes with 12+ built-in presets (`light`, `dark`, `nord`, `catppuccin`, `dracula`, `slate`, `rose-pine`, `apple`, etc.), supporting custom linear gradients, border styling, symbol elevation shadows, and batch multi-theme generation (`themes.json`).
- **Typography Monograms / Lettermarks**: Generates sleek Apple-style lettermarks with San Francisco typography (`--letter "AI"`).
- **Zero Python Dependencies**: Written entirely in pure Python standard library leveraging native macOS subsystem tools (`qlmanage`, `swift`, `sips`, `iconutil`, and `osascript`).

---

## 🚀 Installation

### Using `uv` (Recommended)

Install globally as an isolated CLI tool:

```bash
# From local repository
uv tool install /Users/Artyom_1/Git/DarkRader/macicon

# Or in editable mode for local development
uv tool install --editable /Users/Artyom_1/Git/DarkRader/macicon
```

### Using `pipx`

```bash
pipx install /Users/Artyom_1/Git/DarkRader/macicon
```

### Using `pip`

```bash
pip install -e /Users/Artyom_1/Git/DarkRader/macicon
```

Verify installation:
```bash
macicon --help
```

---

## 🔑 CLI Usage & Examples

### 1. Auto-Fetch & Apply Directly to App

Fetch an icon from Simple Icons and immediately apply it to the target application, automatically refreshing the macOS Dock:

```bash
macicon --query slack --apply "/Applications/Slack.app"
```

*(Tip: You can also pass bare names like `--apply Slack`; macicon auto-discovers apps in `/Applications`)*.

### 2. Custom Color Gradient & Styled Symbol

Create an icon with a subtle vertical background gradient and custom accent color:

```bash
macicon \
  --query discord \
  --bg "#F8FAFC,#E2E8F0" \
  --color "#5865F2" \
  --scale 1.35 \
  --out discord.icns
```

### 3. Apple-Style Typography Monogram

Create an Apple-style typography lettermark monogram:

```bash
macicon \
  --letter "AI" \
  --theme dark \
  --color "#00C8FF,#0072FE" \
  --apply "/Applications/Gemini.app"
```

### 4. Apply an Existing `.icns` File

If you already have an `.icns` file and want to apply it safely without regenerating:

```bash
macicon --icns ./slack.icns --apply "/Applications/Slack.app"
```

### 5. Generate PNG Preview

Quickly preview what the icon looks like without compiling the full multi-resolution iconset:

```bash
macicon --query linear --theme nord --preview /tmp/linear-preview.png
```

### 6. Batch Theme Generation & Synchronization

Generate icons across multiple aesthetic themes managed through `themes.json`:

```bash
# Create a new theme directory with all existing icons styled to Nord
macicon --create-theme nord --bg "#2E3440" --color "#ECEFF4" --no-shadow

# Generate a new app icon across all registered themes simultaneously
macicon --query obsidian --all-themes

# Synchronize missing icons across all configured themes
macicon --sync-themes
```

---

## 🎛 CLI Options Reference

| Option | Flag | Description | Default |
| :--- | :--- | :--- | :--- |
| **Query** | `-q, --query` | App name or Simple Icons URL | — |
| **Letter** | `-l, --letter` | Apple-style lettermark / monogram | — |
| **SVG** | `-s, --svg` | Local SVG file path | — |
| **Path** | `-p, --path` | Direct SVG path `d="..."` | — |
| **ICNS** | `--icns` | Path to existing `.icns` to apply/preview | — |
| **Create Theme** | `--create-theme` | Batch generate icons for a new theme | — |
| **Sync Themes** | `--sync-themes` | Synchronize icons across all themes | `false` |
| **All Themes** | `--all-themes` | Generate icon across all registered themes | `false` |
| **Apply** | `-a, --apply` | Target app bundle to apply icon & restart Dock | — |
| **Theme** | `-t, --theme` | Preset (`light`, `dark`, `nord`, `catppuccin`, etc.) | `light` |
| **Background** | `-b, --bg` | Solid hex or 2-stop gradient (`#FFF,#EEE`) | Theme default |
| **Color** | `-c, --color` | Symbol fill color or gradient | Theme default |
| **Shadow** | `--shadow / --no-shadow` | Toggle symbol drop shadow | `--shadow` |
| **Scale** | `--scale` | Scale multiplier for center symbol | `1.25` |
| **Preview** | `--preview` | Output 1024x1024 preview PNG | Optional |
| **Out** | `-o, --out` | Output `.icns` destination | Auto-derived |

---

## 🛠️ Nix-Darwin Integration

If you use `nix-darwin` and `nix-darwin-custom-icons`:

```nix
environment.customIcons = {
  enable = true;
  icons = [
    {
      path = "/Applications/Discord.app";
      icon = ./icons/light/discord.icns;
    }
  ];
};
```

Generate your theme icons directly into your nix directory:
```bash
macicon --query discord --icons-dir ~/dotfiles/nix/icons --all-themes
```

---

## 📄 License

MIT License. Copyright (c) 2026 Artem Kuznetsov.
