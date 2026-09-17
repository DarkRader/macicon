//! macicon library: domain-driven Apple squircle icon generation and macOS customization.

pub mod application;
pub mod cli;
pub mod domain;
pub mod error;
pub mod infra;

/// Backwards compatibility shims for legacy callers.
pub mod constants {
    pub use crate::domain::geometry::{
        CANVAS_SIZE, CORNER_RADIUS, DEFAULT_SYMBOL_SIZE, TILE_SIZE, TILE_X, TILE_Y,
    };
    pub use crate::domain::icon::{COMMON_ALIASES, KNOWN_ICONS, KnownIcon};
}

pub mod themes {
    pub use crate::application::sync_themes::{
        get_icons_base_dir, load_themes_manifest, save_themes_manifest,
    };
    pub use crate::domain::color::{COLOR_SHORTCUTS, SYMBOL_SHORTCUTS, is_color_dark};
    pub use crate::domain::theme::{
        IconStyle, THEME_PRESETS, ThemeConfig, ThemePreset, compute_styling,
    };
}

pub mod fetcher {
    pub use crate::application::fetch_vector::fetch_icon_or_create;
    pub use crate::domain::icon::IconInfo;
    pub use crate::infra::render::svg::extract_path_from_svg;
}

pub mod renderer {
    pub use crate::application::generate_icon::generate_single_icon;
    pub use crate::infra::render::icns::{compile_icns, render_and_mask};
    pub use crate::infra::render::svg::{build_letter_svg, build_svg};
}

pub mod app_icon {
    pub use crate::application::apply_icon::apply_icon_to_app;
    pub use crate::infra::macos::workspace::find_app_bundle;
}
