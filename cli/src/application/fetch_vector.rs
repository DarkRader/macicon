//! Application use case: Remote or local vector resolution and lettermark fallbacks.

use crate::domain::icon::{IconInfo, derive_candidate_slugs, find_known_icon};
use crate::error::MacIconError;
use crate::infra::network::cdn_fetcher::{fetch_from_url, search_online_icons};

/// Resolve vector icon from direct URL, built-in vector catalog, online CDNs, or monogram lettermark.
pub fn fetch_icon_or_create(query: &str, fallback_letter: bool) -> Result<IconInfo, MacIconError> {
    let mut query_str = query.trim().to_string();

    if query_str.starts_with("http://") || query_str.starts_with("https://") {
        let (direct_icon, resolved_q) = fetch_from_url(&query_str)?;
        if let Some(icon) = direct_icon {
            return Ok(icon);
        }
        query_str = resolved_q;
    }

    let (slug, candidate_slugs) = derive_candidate_slugs(&query_str);

    if let Some(known) = find_known_icon(&slug) {
        return Ok(known);
    }

    if let Some(online_icon) = search_online_icons(&candidate_slugs, &slug) {
        return Ok(online_icon);
    }

    if fallback_letter {
        let letter = if query_str.chars().count() <= 2 {
            query_str.to_uppercase()
        } else {
            query_str
                .chars()
                .next()
                .unwrap_or('A')
                .to_uppercase()
                .to_string()
        };
        println!(
            "Icon '{}' not found online. Generating Apple-style lettermark '{}'...",
            query_str, letter
        );
        return Ok(IconInfo::Letter {
            name: format!("letter-{}", letter.to_lowercase()),
            letter,
        });
    }

    let details = format!(
        "Recommended solutions:\n  \
        1. Check https://simpleicons.org for the exact slug (e.g. 'googlegemini' instead of 'gemini').\n  \
        2. Provide the direct simpleicons link (e.g. https://simpleicons.org/?q={}).\n  \
        3. Use --fallback-letter to auto-create a clean Apple lettermark icon.\n  \
        4. Provide a local vector file with --svg <path>.",
        query_str
    );

    Err(MacIconError::NotFound {
        query: query_str,
        details,
    })
}
