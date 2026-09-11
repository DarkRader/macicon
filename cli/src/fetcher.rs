//! Icon searching, SVG downloading, and path extraction for macicon.

use std::path::Path;
use std::time::Duration;

use regex::Regex;
use url::Url;

use crate::constants::{COMMON_ALIASES, KNOWN_ICONS};

#[derive(Debug, Clone, PartialEq)]
pub enum IconInfo {
    Path {
        name: String,
        path_d: String,
        viewbox: String,
        fill_rule: String,
    },
    Letter {
        name: String,
        letter: String,
    },
}

impl IconInfo {
    pub fn name(&self) -> &str {
        match self {
            IconInfo::Path { name, .. } => name,
            IconInfo::Letter { name, .. } => name,
        }
    }
}

/// Extract combined `<path d='...'>`, viewBox, and fill-rule from SVG markup.
pub fn extract_path_from_svg(svg_content: &str) -> Result<(String, String, String), String> {
    let re_vb = Regex::new(r#"viewBox=["']([^"']+)["']"#).map_err(|e| e.to_string())?;
    let viewbox = if let Some(caps) = re_vb.captures(svg_content) {
        caps.get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "0 0 24 24".to_string())
    } else {
        let re_w =
            Regex::new(r#"<svg[^>]*\bwidth=["']([0-9.]+)["']"#).map_err(|e| e.to_string())?;
        let re_h =
            Regex::new(r#"<svg[^>]*\bheight=["']([0-9.]+)["']"#).map_err(|e| e.to_string())?;
        let w = re_w
            .captures(svg_content)
            .and_then(|c| c.get(1).map(|m| m.as_str()));
        let h = re_h
            .captures(svg_content)
            .and_then(|c| c.get(1).map(|m| m.as_str()));
        match (w, h) {
            (Some(w_val), Some(h_val)) => format!("0 0 {} {}", w_val, h_val),
            _ => "0 0 24 24".to_string(),
        }
    };

    let re_fr = Regex::new(r#"\bfill-rule=["']([^"']+)["']"#).map_err(|e| e.to_string())?;
    let fill_rule = re_fr
        .captures(svg_content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "evenodd".to_string());

    let re_path = Regex::new(r#"<path[^>]*\bd=["']([^"']+)["']"#).map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    for cap in re_path.captures_iter(svg_content) {
        if let Some(d) = cap.get(1) {
            paths.push(d.as_str());
        }
    }

    if paths.is_empty() {
        return Err("No <path d=\"...\"> found in the SVG.".to_string());
    }

    Ok((paths.join(" "), viewbox, fill_rule))
}

fn fetch_from_url(query_str: &str) -> (Option<IconInfo>, String) {
    if let Ok(parsed) = Url::parse(query_str) {
        for (k, v) in parsed.query_pairs() {
            if k == "q" {
                return (None, v.into_owned());
            }
        }

        let path = parsed.path();
        let host = parsed.host_str().unwrap_or("");
        if path.ends_with(".svg")
            || host.contains("cdn.simpleicons.org")
            || host.contains("jsdelivr.net")
        {
            let agent = ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(8))
                .user_agent("macicon-cli")
                .build();

            match agent.get(query_str).call() {
                Ok(resp) => {
                    if let Ok(content) = resp.into_string() {
                        println!("Fetched SVG from URL: {}", query_str);
                        match extract_path_from_svg(&content) {
                            Ok((path_d, viewbox, fill_rule)) => {
                                let stem = Path::new(path)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("custom")
                                    .to_string();
                                return (
                                    Some(IconInfo::Path {
                                        name: stem,
                                        path_d,
                                        viewbox,
                                        fill_rule,
                                    }),
                                    query_str.to_string(),
                                );
                            }
                            Err(e) => {
                                eprintln!("⚠️  Error parsing SVG from URL '{}': {}", query_str, e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Error fetching SVG from URL '{}': {}", query_str, e);
                    std::process::exit(1);
                }
            }
        }

        let path_parts: Vec<&str> = path
            .split('/')
            .filter(|p| !p.is_empty() && *p != "icons")
            .collect();
        let resolved = if let Some(last) = path_parts.last() {
            last.replace(".svg", "")
        } else {
            query_str.to_string()
        };
        return (None, resolved);
    }

    (None, query_str.to_string())
}

fn search_online_icons(candidate_slugs: &[String], slug: &str) -> Option<IconInfo> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .user_agent("macicon-cli")
        .build();

    for cand in candidate_slugs {
        let sources = [
            (
                "Simple Icons",
                format!(
                    "https://cdn.jsdelivr.net/npm/simple-icons@latest/icons/{}.svg",
                    cand
                ),
            ),
            (
                "Dashboard Icons",
                format!(
                    "https://raw.githubusercontent.com/homarr-labs/dashboard-icons/main/svg/{}.svg",
                    cand
                ),
            ),
        ];

        for (source_name, url) in sources {
            if let Ok(resp) = agent.get(&url).call() {
                if let Ok(content) = resp.into_string() {
                    println!("Found icon on {}: {}", source_name, url);
                    if let Ok((path_d, viewbox, fill_rule)) = extract_path_from_svg(&content) {
                        return Some(IconInfo::Path {
                            name: slug.to_string(),
                            path_d,
                            viewbox,
                            fill_rule,
                        });
                    }
                }
            }
        }
    }
    None
}

/// Resolve icon from direct URL, built-in vector, Simple Icons, or lettermark monogram.
pub fn fetch_icon_or_create(query: &str, fallback_letter: bool) -> IconInfo {
    let mut query_str = query.trim().to_string();

    if query_str.starts_with("http://") || query_str.starts_with("https://") {
        let (direct_icon, resolved_q) = fetch_from_url(&query_str);
        if let Some(icon) = direct_icon {
            return icon;
        }
        query_str = resolved_q;
    }

    let mut slug = query_str.to_lowercase().replace([' ', '-', '_'], "");
    let slug_hyphen = query_str.to_lowercase().replace([' ', '_'], "-");

    if let Some(&(_, alias)) = COMMON_ALIASES.iter().find(|(k, _)| *k == slug) {
        if KNOWN_ICONS.iter().any(|(k, _)| *k == alias) {
            slug = alias.to_string();
        }
    } else if let Some(&(_, alias)) = COMMON_ALIASES.iter().find(|(k, _)| *k == slug_hyphen) {
        if KNOWN_ICONS.iter().any(|(k, _)| *k == alias) {
            slug = alias.to_string();
        }
    }

    if let Some((_, icon)) = KNOWN_ICONS.iter().find(|(k, _)| *k == slug) {
        return IconInfo::Path {
            name: icon.name.to_string(),
            path_d: icon.path_d.to_string(),
            viewbox: icon.viewbox.to_string(),
            fill_rule: icon.fill_rule.to_string(),
        };
    }

    let mut candidate_slugs = vec![slug.clone()];
    if !candidate_slugs.contains(&slug_hyphen) {
        candidate_slugs.push(slug_hyphen.clone());
    }
    if let Some(&(_, alias)) = COMMON_ALIASES.iter().find(|(k, _)| *k == slug) {
        let s = alias.to_string();
        if !candidate_slugs.contains(&s) {
            candidate_slugs.push(s);
        }
    }
    if let Some(&(_, alias)) = COMMON_ALIASES.iter().find(|(k, _)| *k == slug_hyphen) {
        let s = alias.to_string();
        if !candidate_slugs.contains(&s) {
            candidate_slugs.push(s);
        }
    }

    for suffix in ["industries", "editor", "app"] {
        let s = format!("{}{}", slug, suffix);
        if !candidate_slugs.contains(&s) {
            candidate_slugs.push(s);
        }
    }
    let s_hyphen = format!("{}-app", slug_hyphen);
    if !candidate_slugs.contains(&s_hyphen) {
        candidate_slugs.push(s_hyphen);
    }

    if let Some(online_icon) = search_online_icons(&candidate_slugs, &slug) {
        return online_icon;
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
        return IconInfo::Letter {
            name: format!("letter-{}", letter.to_lowercase()),
            letter,
        };
    }

    eprintln!(
        "⚠️  Error: Could not find icon '{}' on Simple Icons or Dashboard Icons.\n\n\
        Recommended solutions:\n  \
        1. Check https://simpleicons.org for the exact slug (e.g. 'googlegemini' instead of 'gemini').\n  \
        2. Provide the direct simpleicons link (e.g. https://simpleicons.org/?q={}).\n  \
        3. Use --fallback-letter to auto-create a clean Apple lettermark icon.\n  \
        4. Provide a local vector file with --svg <path>.",
        query_str, query_str
    );
    std::process::exit(1);
}
