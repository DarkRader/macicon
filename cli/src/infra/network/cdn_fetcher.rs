//! Network client for downloading vectors from CDNs and remote URLs.

use std::path::Path;
use std::time::Duration;

use url::Url;

use crate::domain::icon::IconInfo;
use crate::error::InfraError;
use crate::infra::render::svg::extract_path_from_svg;

/// Fetch an SVG directly from a remote HTTP/HTTPS URL.
pub fn fetch_from_url(query_str: &str) -> Result<(Option<IconInfo>, String), InfraError> {
    if let Ok(parsed) = Url::parse(query_str) {
        for (k, v) in parsed.query_pairs() {
            if k == "q" {
                return Ok((None, v.into_owned()));
            }
        }

        let path = parsed.path();
        let host = parsed.host_str().unwrap_or("");
        if path.ends_with(".svg")
            || host.contains("cdn.simpleicons.org")
            || host.contains("jsdelivr.net")
        {
            let config = ureq::config::Config::builder()
                .timeout_global(Some(Duration::from_secs(8)))
                .user_agent("macicon-cli")
                .build();
            let agent = ureq::Agent::new_with_config(config);

            let mut resp = agent.get(query_str).call().map_err(|e| {
                InfraError::Network(format!(
                    "Failed to fetch SVG from URL '{}': {}",
                    query_str, e
                ))
            })?;

            let content = resp.body_mut().read_to_string().map_err(|e| {
                InfraError::Network(format!(
                    "Failed to read response from '{}': {}",
                    query_str, e
                ))
            })?;

            println!("Fetched SVG from URL: {}", query_str);
            let (path_d, viewbox, fill_rule) = extract_path_from_svg(&content)?;
            let stem = Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("custom")
                .to_string();

            return Ok((
                Some(IconInfo::Path {
                    name: stem,
                    path_d,
                    viewbox,
                    fill_rule,
                }),
                query_str.to_string(),
            ));
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
        return Ok((None, resolved));
    }

    Ok((None, query_str.to_string()))
}

/// Search for icons across Simple Icons and Dashboard Icons CDNs.
pub fn search_online_icons(candidate_slugs: &[String], slug: &str) -> Option<IconInfo> {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(6)))
        .user_agent("macicon-cli")
        .build();
    let agent = ureq::Agent::new_with_config(config);

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
            if let Ok(mut resp) = agent.get(&url).call()
                && let Ok(content) = resp.body_mut().read_to_string()
            {
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
    None
}
