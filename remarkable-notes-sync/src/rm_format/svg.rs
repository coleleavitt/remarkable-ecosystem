//! SVG export for reMarkable strokes

use std::fmt::Write;
use super::types::*;

/// Configuration for SVG export
#[derive(Debug, Clone)]
pub struct SvgConfig {
    /// Include background rectangle
    pub background: bool,
    /// Background color
    pub bg_color: String,
    /// Padding around content
    pub padding: f64,
    /// Scale factor
    pub scale: f64,
    /// Stroke width multiplier
    pub stroke_width_scale: f64,
    /// Use pressure for variable width
    pub pressure_sensitive: bool,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            background: true,
            bg_color: "#f5f5f0".to_string(),
            padding: 50.0,
            scale: 1.0,
            stroke_width_scale: 1.0,
            pressure_sensitive: true,
        }
    }
}

/// Export strokes to SVG
pub fn export_svg(page: &RmPage, config: &SvgConfig) -> String {
    let (min_x, min_y, max_x, max_y) = page.bounds();
    
    let width = (max_x - min_x + config.padding * 2.0) * config.scale;
    let height = (max_y - min_y + config.padding * 2.0) * config.scale;
    let view_box = format!(
        "{} {} {} {}",
        min_x - config.padding,
        min_y - config.padding,
        max_x - min_x + config.padding * 2.0,
        max_y - min_y + config.padding * 2.0
    );

    let mut svg = String::new();
    
    // Header
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{}\" width=\"{:.0}\" height=\"{:.0}\">\n",
        view_box, width, height
    ));
    
    // Background
    if config.background {
        svg.push_str(&format!("  <rect width=\"100%\" height=\"100%\" fill=\"{}\"/>\n", config.bg_color));
    }
    
    // Strokes grouped by layer
    for layer in &page.layers {
        if !layer.visible {
            continue;
        }
        
        svg.push_str(&format!("  <g id=\"layer-{}-{}\">\n", layer.id.part1, layer.id.part2));
        
        for stroke in &layer.strokes {
            let path = stroke_to_path(stroke, config);
            if !path.is_empty() {
                svg.push_str(&path);
                svg.push('\n');
            }
        }
        
        svg.push_str("  </g>\n");
    }
    
    // Text items
    for text in &page.text_items {
        svg.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{:.1}\" fill=\"#000\">{}</text>\n",
            text.x, text.y, text.font_size, escape_xml(&text.text)
        ));
    }
    
    svg.push_str("</svg>");
    svg
}

/// Convert a stroke to SVG path element
fn stroke_to_path(stroke: &Stroke, config: &SvgConfig) -> String {
    if stroke.points.len() < 2 {
        return String::new();
    }
    
    // Get color
    let color = stroke.rgba
        .map(|(r, g, b, _)| format!("#{:02x}{:02x}{:02x}", r, g, b))
        .unwrap_or_else(|| stroke.color.to_rgb().to_string());
    
    // Calculate stroke width
    let base_width = stroke.thickness_scale * config.stroke_width_scale;
    let stroke_width = if config.pressure_sensitive {
        base_width * stroke.avg_pressure().max(0.1) * 2.0
    } else {
        base_width.max(1.0)
    };
    
    // Opacity for highlighters
    let opacity = if stroke.tool.is_highlighter() { 0.5 } else { 1.0 };
    
    // Build path
    let mut path_d = String::new();
    let first = &stroke.points[0];
    write!(path_d, "M {:.2} {:.2}", first.x, first.y).unwrap();
    
    // Use quadratic curves for smoother lines
    if stroke.points.len() == 2 {
        let last = &stroke.points[1];
        write!(path_d, " L {:.2} {:.2}", last.x, last.y).unwrap();
    } else {
        for i in 1..stroke.points.len() - 1 {
            let p1 = &stroke.points[i];
            let p2 = &stroke.points[i + 1];
            let mid_x = (p1.x + p2.x) / 2.0;
            let mid_y = (p1.y + p2.y) / 2.0;
            write!(path_d, " Q {:.2} {:.2} {:.2} {:.2}", p1.x, p1.y, mid_x, mid_y).unwrap();
        }
        let last = stroke.points.last().unwrap();
        write!(path_d, " L {:.2} {:.2}", last.x, last.y).unwrap();
    }
    
    format!(
        "    <path d=\"{}\" stroke=\"{}\" stroke-width=\"{:.2}\" fill=\"none\" stroke-linecap=\"round\" stroke-linejoin=\"round\" opacity=\"{}\"/>",
        path_d, color, stroke_width, opacity
    )
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Export to inline SVG data URL
pub fn export_svg_data_url(page: &RmPage, config: &SvgConfig) -> String {
    let svg = export_svg(page, config);
    let encoded = base64_encode(svg.as_bytes());
    format!("data:image/svg+xml;base64,{}", encoded)
}

// Simple base64 encoder
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        
        let combined = (b0 << 16) | (b1 << 8) | b2;
        
        result.push(ALPHABET[(combined >> 18) as usize & 0x3F] as char);
        result.push(ALPHABET[(combined >> 12) as usize & 0x3F] as char);
        
        if chunk.len() > 1 {
            result.push(ALPHABET[(combined >> 6) as usize & 0x3F] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(ALPHABET[combined as usize & 0x3F] as char);
        } else {
            result.push('=');
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_page() {
        let page = RmPage::new();
        let config = SvgConfig::default();
        let svg = export_svg(&page, &config);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }
}
