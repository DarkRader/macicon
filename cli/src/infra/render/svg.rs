//! SVG template generation and vector path extraction for macicon.

use regex::Regex;

use crate::domain::geometry::{
    CANVAS_SIZE, CORNER_RADIUS, TILE_SIZE, TILE_X, TILE_Y, calculate_lettermark_metrics,
    calculate_symbol_transform,
};
use crate::domain::theme::IconStyle;
use crate::error::InfraError;

/// Generate standard Apple HIG squircle SVG markup with an embedded vector symbol.
pub fn build_svg(
    path_d: &str,
    viewbox: &str,
    style: &IconStyle,
    scale: f64,
    fill_rule: &str,
    shadow: bool,
) -> String {
    let vb_cleaned = viewbox.replace(',', " ");
    let vb_parts: Vec<f64> = vb_cleaned
        .split_whitespace()
        .filter_map(|x| x.parse::<f64>().ok())
        .collect();

    let (min_x, min_y, vb_w, vb_h) = if vb_parts.len() == 4 {
        (vb_parts[0], vb_parts[1], vb_parts[2], vb_parts[3])
    } else if vb_parts.len() == 2 {
        (0.0, 0.0, vb_parts[0], vb_parts[1])
    } else {
        (0.0, 0.0, 24.0, 24.0)
    };

    let transform = calculate_symbol_transform(min_x, min_y, vb_w, vb_h, scale);

    let filter_attr = if shadow {
        r#" filter="url(#symbol-shadow)""#
    } else {
        ""
    };

    let mut sym_grad_def = String::new();
    let fill_attr: String;

    if style.symbol_color.contains(',') {
        let parts: Vec<&str> = style.symbol_color.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            sym_grad_def = format!(
                r#"    <linearGradient id="symbol-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{}"/>
      <stop offset="100%" stop-color="{}"/>
    </linearGradient>"#,
                parts[0], parts[1]
            );
            fill_attr = "url(#symbol-grad)".to_string();
        } else {
            fill_attr = style.symbol_color.clone();
        }
    } else {
        fill_attr = style.symbol_color.clone();
    }

    format!(
        r##"<svg width="{canv}" height="{canv}" viewBox="0 0 {canv} {canv}" xmlns="http://www.w3.org/2000/svg">
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
  <rect x="{tile_x}" y="{tile_y}" width="{tile_sz}" height="{tile_sz}" rx="{corner_rad}" fill="url(#bg-grad)" stroke="{border}" stroke-width="1.5"/>

  <!-- Centered Symbol -->
  <g transform="translate(512, 504) scale({calc_scale}) translate({neg_cx}, {neg_cy})"{filter_attr}>
    <path fill="{fill_attr}" fill-rule="{fill_rule}" d="{path_d}"/>
  </g>
</svg>"##,
        canv = CANVAS_SIZE,
        bg_top = style.bg_top,
        bg_bottom = style.bg_bottom,
        tile_x = TILE_X,
        tile_y = TILE_Y,
        tile_sz = TILE_SIZE,
        corner_rad = CORNER_RADIUS,
        border = style.border,
        calc_scale = transform.scale,
        neg_cx = -transform.center_x,
        neg_cy = -transform.center_y,
        filter_attr = filter_attr,
        fill_attr = fill_attr,
        fill_rule = fill_rule,
        path_d = path_d,
        sym_grad_def = sym_grad_def,
    )
}

/// Generate an Apple-style typography monogram lettermark SVG.
pub fn build_letter_svg(letter: &str, style: &IconStyle, scale: f64, shadow: bool) -> String {
    let metrics = calculate_lettermark_metrics(letter, scale);
    let filter_attr = if shadow {
        r#" filter="url(#symbol-shadow)""#
    } else {
        ""
    };

    let mut sym_grad_def = String::new();
    let fill_attr: String;

    if style.symbol_color.contains(',') {
        let parts: Vec<&str> = style.symbol_color.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            sym_grad_def = format!(
                r#"    <linearGradient id="symbol-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{}"/>
      <stop offset="100%" stop-color="{}"/>
    </linearGradient>"#,
                parts[0], parts[1]
            );
            fill_attr = "url(#symbol-grad)".to_string();
        } else {
            fill_attr = style.symbol_color.clone();
        }
    } else {
        fill_attr = style.symbol_color.clone();
    }

    format!(
        r##"<svg width="{canv}" height="{canv}" viewBox="0 0 {canv} {canv}" xmlns="http://www.w3.org/2000/svg">
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
  <rect x="{tile_x}" y="{tile_y}" width="{tile_sz}" height="{tile_sz}" rx="{corner_rad}" fill="url(#bg-grad)" stroke="{border}" stroke-width="1.5"/>

  <!-- Centered Lettermark -->
  <text x="512" y="{y_pos}" font-family="-apple-system, 'SF Pro Display', system-ui, sans-serif" font-size="{font_size}" font-weight="800" text-anchor="middle" fill="{fill_attr}"{filter_attr}>{letter}</text>
</svg>"##,
        canv = CANVAS_SIZE,
        bg_top = style.bg_top,
        bg_bottom = style.bg_bottom,
        tile_x = TILE_X,
        tile_y = TILE_Y,
        tile_sz = TILE_SIZE,
        corner_rad = CORNER_RADIUS,
        border = style.border,
        letter = letter,
        y_pos = metrics.y_pos,
        font_size = metrics.font_size,
        filter_attr = filter_attr,
        fill_attr = fill_attr,
        sym_grad_def = sym_grad_def,
    )
}

/// Extract combined `<path d='...'>`, viewBox, and fill-rule from SVG markup.
pub fn extract_path_from_svg(svg_content: &str) -> Result<(String, String, String), InfraError> {
    let re_vb = Regex::new(r#"viewBox=["']([^"']+)["']"#)
        .map_err(|e| InfraError::SvgParse(e.to_string()))?;
    let viewbox = if let Some(caps) = re_vb.captures(svg_content) {
        caps.get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "0 0 24 24".to_string())
    } else {
        let re_w = Regex::new(r#"<svg[^>]*\bwidth=["']([0-9.]+)["']"#)
            .map_err(|e| InfraError::SvgParse(e.to_string()))?;
        let re_h = Regex::new(r#"<svg[^>]*\bheight=["']([0-9.]+)["']"#)
            .map_err(|e| InfraError::SvgParse(e.to_string()))?;
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

    let re_fr = Regex::new(r#"\bfill-rule=["']([^"']+)["']"#)
        .map_err(|e| InfraError::SvgParse(e.to_string()))?;
    let fill_rule = re_fr
        .captures(svg_content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "evenodd".to_string());

    let re_path = Regex::new(r#"<path[^>]*\bd=["']([^"']+)["']"#)
        .map_err(|e| InfraError::SvgParse(e.to_string()))?;
    let mut paths = Vec::new();
    for cap in re_path.captures_iter(svg_content) {
        if let Some(d) = cap.get(1) {
            paths.push(d.as_str());
        }
    }

    if paths.is_empty() {
        return Err(InfraError::SvgParse(
            "No <path d=\"...\"> found in the SVG.".to_string(),
        ));
    }

    Ok((paths.join(" "), viewbox, fill_rule))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_svg() {
        let style = IconStyle {
            bg_top: "#FFFFFF".to_string(),
            bg_bottom: "#000000".to_string(),
            border: "#CCCCCC".to_string(),
            symbol_color: "#FF0000".to_string(),
        };
        let svg = build_svg("M0 0 L10 10", "0 0 24 24", &style, 1.25, "evenodd", true);
        assert!(svg.contains("rx=\"185\""));
        assert!(svg.contains("d=\"M0 0 L10 10\""));
        assert!(svg.contains("filter=\"url(#symbol-shadow)\""));
    }

    #[test]
    fn test_extract_path() {
        let svg = r#"<svg viewBox="0 0 48 48"><path d="M10 10 L20 20"/></svg>"#;
        let (path_d, vb, fr) = extract_path_from_svg(svg).unwrap();
        assert_eq!(path_d, "M10 10 L20 20");
        assert_eq!(vb, "0 0 48 48");
        assert_eq!(fr, "evenodd");
    }
}
