//! Command-line argument definitions and flags for macicon.

use clap::Parser;

/// macicon CLI arguments structure.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "macicon",
    about = "Generate, customize, and apply Apple continuous-curvature squircle app icons on macOS.",
    version
)]
pub struct Cli {
    /// Positional icon search query or name (e.g. 'slack')
    #[arg(value_name = "QUERY")]
    pub query: Option<String>,

    /// Search term or link from Simple Icons (e.g. 'slack', 'https://simpleicons.org/?q=warp')
    #[arg(short = 'q', long)]
    pub search_query: Option<String>,

    /// Path to a local SVG file
    #[arg(short = 's', long)]
    pub svg: Option<String>,

    /// Direct SVG path string d='...'
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Generate an Apple-style typography lettermark monogram (e.g. 'S', 'G', 'AI')
    #[arg(short = 'l', long)]
    pub letter: Option<String>,

    /// Path to an existing .icns file
    #[arg(long)]
    pub icns: Option<String>,

    /// Batch generate all existing icons into a new theme folder
    #[arg(long, value_name = "THEME_NAME")]
    pub create_theme: Option<String>,

    /// Sync and regenerate missing icons across all themes registered in themes.json
    #[arg(long)]
    pub sync_themes: bool,

    /// If query is not found online, automatically fall back to an Apple lettermark monogram
    #[arg(long)]
    pub fallback_letter: bool,

    /// Generate icon for all registered themes in themes.json
    #[arg(long)]
    pub all_themes: bool,

    /// Reference theme to discover icons from when using --create-theme
    #[arg(long, default_value = "light")]
    pub from_theme: String,

    /// Base directory containing theme folders (default: './icons' or auto-detected 'nix/icons')
    #[arg(long)]
    pub icons_dir: Option<String>,

    /// Base theme preset (default: light)
    #[arg(short = 't', long, default_value = "light")]
    pub theme: String,

    /// Background color/gradient: 'white', 'dark', '#FFFFFF', or gradient '#FFFFFF,#EBECEF'
    #[arg(short = 'b', long)]
    pub bg: Option<String>,

    /// Symbol color: 'black', 'white', hex '#202022', or gradient '#00C8FF,#0072FE'
    #[arg(short = 'c', long)]
    pub color: Option<String>,

    /// Custom tile border color (hex)
    #[arg(long)]
    pub border_color: Option<String>,

    /// Disable drop shadow on the symbol (for a flat appearance)
    #[arg(long)]
    pub no_shadow: bool,

    /// Scale multiplier for the symbol (default: 1.25)
    #[arg(long, default_value_t = 1.25)]
    pub scale: f64,

    /// SVG viewBox when using --path (default: '0 0 24 24')
    #[arg(short = 'v', long, default_value = "0 0 24 24")]
    pub viewbox: String,

    /// Destination path for .icns
    #[arg(short = 'o', long)]
    pub out: Option<String>,

    /// Generate a 1024x1024 PNG preview (optional custom path, or auto /tmp/<name>-preview.png)
    #[arg(long, num_args = 0..=1)]
    pub preview: Option<Option<String>>,

    /// Target .app bundle path or app name to immediately apply the icon to
    #[arg(short = 'a', long)]
    pub apply: Option<String>,

    /// Do not restart the Dock after applying the icon
    #[arg(long)]
    pub no_dock_restart: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_positional_query() {
        let args = Cli::try_parse_from(["macicon", "slack"]).unwrap();
        assert_eq!(args.query.as_deref(), Some("slack"));
        assert_eq!(args.theme, "light");
        assert_eq!(args.scale, 1.25);
        assert!(!args.no_shadow);
        assert_eq!(args.viewbox, "0 0 24 24");
    }

    #[test]
    fn test_parse_flags() {
        let args = Cli::try_parse_from([
            "macicon",
            "-q",
            "warp",
            "-t",
            "dark",
            "-b",
            "black",
            "-c",
            "white",
            "--border-color",
            "#444444",
            "--no-shadow",
            "--scale",
            "1.5",
            "-o",
            "/tmp/out.icns",
            "-a",
            "Warp",
            "--no-dock-restart",
        ])
        .unwrap();

        assert_eq!(args.search_query.as_deref(), Some("warp"));
        assert_eq!(args.theme, "dark");
        assert_eq!(args.bg.as_deref(), Some("black"));
        assert_eq!(args.color.as_deref(), Some("white"));
        assert_eq!(args.border_color.as_deref(), Some("#444444"));
        assert!(args.no_shadow);
        assert_eq!(args.scale, 1.5);
        assert_eq!(args.out.as_deref(), Some("/tmp/out.icns"));
        assert_eq!(args.apply.as_deref(), Some("Warp"));
        assert!(args.no_dock_restart);
    }

    #[test]
    fn test_parse_theme_commands() {
        let args = Cli::try_parse_from([
            "macicon",
            "--create-theme",
            "mytheme",
            "--from-theme",
            "nord",
            "--icons-dir",
            "custom/icons",
        ])
        .unwrap();

        assert_eq!(args.create_theme.as_deref(), Some("mytheme"));
        assert_eq!(args.from_theme, "nord");
        assert_eq!(args.icons_dir.as_deref(), Some("custom/icons"));
    }

    #[test]
    fn test_parse_preview_flag() {
        let args = Cli::try_parse_from(["macicon", "slack", "--preview"]).unwrap();
        assert_eq!(args.preview, Some(None));

        let args_custom =
            Cli::try_parse_from(["macicon", "slack", "--preview", "/tmp/slack.png"]).unwrap();
        assert_eq!(
            args_custom.preview,
            Some(Some("/tmp/slack.png".to_string()))
        );
    }
}
