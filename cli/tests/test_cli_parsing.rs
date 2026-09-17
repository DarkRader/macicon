//! Integration tests for CLI command-line argument parsing, validation, and flag combinations.

use clap::Parser;
use macicon::cli::args::Cli;

#[test]
fn test_cli_positional_and_query_flags() {
    let cli = Cli::try_parse_from(["macicon", "telegram"]).unwrap();
    assert_eq!(cli.query.as_deref(), Some("telegram"));
    assert!(cli.search_query.is_none());

    let cli_search = Cli::try_parse_from(["macicon", "-q", "telegram"]).unwrap();
    assert_eq!(cli_search.search_query.as_deref(), Some("telegram"));
}

#[test]
fn test_cli_lettermark_input() {
    let cli = Cli::try_parse_from(["macicon", "-l", "RS", "--no-shadow"]).unwrap();
    assert_eq!(cli.letter.as_deref(), Some("RS"));
    assert!(cli.no_shadow);
}

#[test]
fn test_cli_svg_and_path_inputs() {
    let cli_svg = Cli::try_parse_from(["macicon", "-s", "assets/icon.svg"]).unwrap();
    assert_eq!(cli_svg.svg.as_deref(), Some("assets/icon.svg"));

    let cli_path =
        Cli::try_parse_from(["macicon", "-p", "M0 0 L10 10", "-v", "0 0 100 100"]).unwrap();
    assert_eq!(cli_path.path.as_deref(), Some("M0 0 L10 10"));
    assert_eq!(cli_path.viewbox, "0 0 100 100");
}

#[test]
fn test_cli_theme_and_styling_flags() {
    let cli = Cli::try_parse_from([
        "macicon",
        "slack",
        "-t",
        "catppuccin",
        "-b",
        "#111111,#222222",
        "-c",
        "cyan",
        "--border-color",
        "#333333",
        "--scale",
        "1.4",
    ])
    .unwrap();

    assert_eq!(cli.theme, "catppuccin");
    assert_eq!(cli.bg.as_deref(), Some("#111111,#222222"));
    assert_eq!(cli.color.as_deref(), Some("cyan"));
    assert_eq!(cli.border_color.as_deref(), Some("#333333"));
    assert_eq!(cli.scale, 1.4);
}

#[test]
fn test_cli_batch_theme_sync_flags() {
    let cli_sync =
        Cli::try_parse_from(["macicon", "--sync-themes", "--icons-dir", "custom_dir"]).unwrap();
    assert!(cli_sync.sync_themes);
    assert_eq!(cli_sync.icons_dir.as_deref(), Some("custom_dir"));

    let cli_all = Cli::try_parse_from(["macicon", "slack", "--all-themes"]).unwrap();
    assert!(cli_all.all_themes);
}
