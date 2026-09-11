# AGENTS.md

## Repository Overview

`macicon` is a standalone macOS CLI application designed to generate, customize, theme, and apply Apple continuous-curvature squircle application icons (`.icns`).

The core codebase is written in pure Rust (2021 edition) and interfaces with macOS native APIs and system utilities (`qlmanage`, `swift`, `sips`, `iconutil`, and `osascript`). Project toolchains and tasks are orchestrated from the repository root using `mise`.

## Architectural Invariants & Apple HIG Specifications

- **Canvas Dimensions**: Standard 1024x1024 canvas.
- **Squircle Geometry**: 832x832 squircle tile located at `x = 96`, `y = 88` with corner radius `r = 185` adhering strictly to Apple Human Interface Guidelines (HIG).
- **Cocoa Coordinate Conversion**: Cocoa/AppKit uses bottom-left origin, corresponding to `y = 104` for clipping mask alignment.
- **Corner Curvature**: Must preserve continuous curvature (superellipse / squircle) without sharp corner artifacts or rectangular fallbacks.
- **Zero External C Dependencies**: Pure Rust and macOS subsystem utilities only; avoid binding external C/C++ graphics dependencies.

## Important Paths

- `cli/`: Rust package directory (`Cargo.toml`, `Cargo.lock`).
  - `cli/src/main.rs`: CLI binary entrypoint (argument parsing and high-level commands).
  - `cli/src/lib.rs`: Public library module definitions.
  - `cli/src/renderer.rs`: Squircle geometry, SVG rendering, and `.icns` compilation.
  - `cli/src/themes.rs`: Preset and custom theme computation engine.
  - `cli/src/fetcher.rs`: Simple Icons and Dashboard Icons vector fetching and caching.
  - `cli/src/app_icon.rs`: Native macOS app bundle icon application and Dock refresh.
  - `cli/src/constants.rs`: Dimensions, colors, and default theme presets.
  - `cli/tests/`: Integration test suite (`test_basic.rs`).
- `mise.toml`: Unified toolchain definition and development task runner.
- `.githooks/pre-commit`: Native git pre-commit hook (file hygiene, formatting, clippy, tests).
- `.github/workflows/`:
  - `test.yml`: Independent parallel CI jobs for `fmt`, `clippy`, and `test`.
  - `build.yml`: Release binary build verification on `macos-latest`.
- `README.md`: User documentation, installation methods, and CLI usage reference.

## Setup and Toolchain

Prerequisites:
- [`mise`](https://mise.jdx.dev/) (orchestrates the Rust toolchain).

Install repository git pre-commit hooks:

```bash
mise run setup-hooks
```

## Development and Verification

Always invoke tasks from the repository root using `mise`:

```bash
mise run build        # Build debug binary in cli/target/debug
mise run release      # Build optimized release binary in cli/target/release
mise run fmt          # Format Rust code with rustfmt
mise run fmt-check    # Check Rust formatting without modifying files
mise run clippy       # Run clippy with strict warnings (-D warnings)
mise run test         # Run unit and integration tests
mise run check        # Run full quality suite (fmt-check, clippy, test)
mise run pre-commit   # Run full pre-commit pipeline (file hygiene + check)
```

Before claiming any code or documentation change is complete:
1. Run `mise run check` to guarantee formatting, linter warnings, and test coverage pass cleanly.
2. Run `git status --short` and `git diff --check` to ensure no whitespace defects or untracked debris exist.

## Change Workflow

1. Keep the repository root reserved for orchestration, workflows, and documentation; all Rust code belongs inside `cli/`.
2. Modify existing tests or add new tests under `cli/tests/` when modifying rendering, theming, or CLI arguments.
3. Preserve existing documentation integrity and comments unless explicitly modifying that behavior.
4. Verify changes locally with `mise run check` prior to proposing completion.

## Safety and Code Style

- **Do not modify global user environment**: Never edit shell configuration files (`~/.zshrc`, `~/.bashrc`) or alter global system `PATH`. All tools must run via `mise`.
- **Do not commit build artifacts**: Keep `target/`, `cli/target/`, temporary `.icns`, `.iconset`, or `.png` files untracked.
- **Pure dependencies**: Rely on the Rust standard library, well-established crates (`clap`, `serde`, `reqwest`), and native macOS subsystem tools.

## Commit and Push Policy

Always follow [Conventional Commits](https://www.conventionalcommits.org/):

```text
<type>(<scope>): <short imperative description>
```

Use these recognized types when applicable: `feat`, `chore`, `fix`, `deps`, `refactor`, `docs`, `test`, and `ci`. When a change is localized, include a specific scope in parentheses after the commit type (e.g., `cli`, `renderer`, `themes`, `ci`, `hooks`, `docs`).

> [!IMPORTANT]
> **Approval required**: Never create a git commit or push to remote branches without explicit user approval.
> After completing a requested change, suggest the relevant scoped commit message and ask the user for confirmation first.

## Pull Request Policy

- **Do not modify the PR description for updates**: When appending changes, progress, or follow-ups to an existing Pull Request, never overwrite or edit the original PR description. Always post a new comment on the PR detailing the additions and linking relevant issues (e.g., `Resolves #<issue>`).
