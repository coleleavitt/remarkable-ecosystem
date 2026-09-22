//! Markdown export for Obsidian compatibility

use super::types::*;
use super::svg::{export_svg, SvgConfig};

/// Configuration for Markdown export
#[derive(Debug, Clone)]
pub struct MarkdownConfig {
    /// SVG configuration
    pub svg_config: SvgConfig,
    /// Embed SVG inline (true) or link to file (false)
    pub embed_svg: bool,
    /// Include frontmatter
    pub frontmatter: bool,
    /// Convert text items to markdown
    pub include_text: bool,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            svg_config: SvgConfig::default(),
            embed_svg: true,
            frontmatter: true,
            include_text: true,
        }
    }
}

/// Document metadata for Markdown frontmatter
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub title: String,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Vec<String>,
    pub folder: Option<String>,
    pub device_id: Option<String>,
    pub remarkable_id: Option<String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            created: None,
            modified: None,
            tags: Vec::new(),
            folder: None,
            device_id: None,
            remarkable_id: None,
        }
    }
}

/// Export a page to Markdown with embedded SVG
pub fn export_markdown(
    page: &RmPage,
    metadata: &DocumentMetadata,
    config: &MarkdownConfig,
) -> String {
    let mut md = String::new();
    
    // Frontmatter
    if config.frontmatter {
        md.push_str("---\n");
        md.push_str(&format!("title: \"{}\n", escape_yaml(&metadata.title)));
        
        if let Some(created) = &metadata.created {
            md.push_str(&format!("created: {}\n", created.format("%Y-%m-%dT%H:%M:%S")));
        }
        if let Some(modified) = &metadata.modified {
            md.push_str(&format!("modified: {}\n", modified.format("%Y-%m-%dT%H:%M:%S")));
        }
        
        if !metadata.tags.is_empty() {
            md.push_str("tags:\n");
            for tag in &metadata.tags {
                md.push_str(&format!("  - {}\n", tag));
            }
        }
        
        if let Some(folder) = &metadata.folder {
            md.push_str(&format!("remarkable_folder: {}\n", folder));
        }
        
        if let Some(id) = &metadata.remarkable_id {
            md.push_str(&format!("remarkable_id: {}\n", id));
        }
        
        md.push_str("source: reMarkable\n");
        md.push_str("---\n\n");
    }
    
    // Title
    md.push_str(&format!("# {}\n\n", metadata.title));
    
    // Embedded SVG or placeholder
    if page.stroke_count() > 0 || !page.text_items.is_empty() {
        if config.embed_svg {
            let svg = export_svg(page, &config.svg_config);
            md.push_str(&svg);
            md.push_str("\n\n");
        } else {
            // Reference to external SVG file
            let svg_name = sanitize_filename(&metadata.title);
            md.push_str(&format!("![[{}.svg]]\n\n", svg_name));
        }
    }
    
    // Text content
    if config.include_text && !page.text_items.is_empty() {
        md.push_str("## Text\n\n");
        for text in &page.text_items {
            md.push_str(&text.text);
            md.push_str("\n\n");
        }
    }
    
    // Metadata footer
    md.push_str("---\n");
    md.push_str("*Synced from reMarkable*");
    if let Some(modified) = &metadata.modified {
        md.push_str(&format!(" | Last modified: {}", modified.format("%Y-%m-%d %H:%M")));
    }
    md.push_str("\n");
    
    md
}

/// Export a multi-page document to Markdown
pub fn export_document_markdown(
    pages: &[RmPage],
    metadata: &DocumentMetadata,
    config: &MarkdownConfig,
) -> String {
    let mut md = String::new();
    
    // Frontmatter
    if config.frontmatter {
        md.push_str("---\n");
        md.push_str(&format!("title: \"{}\n", escape_yaml(&metadata.title)));
        
        if let Some(created) = &metadata.created {
            md.push_str(&format!("created: {}\n", created.format("%Y-%m-%dT%H:%M:%S")));
        }
        if let Some(modified) = &metadata.modified {
            md.push_str(&format!("modified: {}\n", modified.format("%Y-%m-%dT%H:%M:%S")));
        }
        
        md.push_str(&format!("pages: {}\n", pages.len()));
        md.push_str("source: reMarkable\n");
        md.push_str("---\n\n");
    }
    
    // Title
    md.push_str(&format!("# {}\n\n", metadata.title));
    
    // Pages
    for (i, page) in pages.iter().enumerate() {
        if pages.len() > 1 {
            md.push_str(&format!("## Page {}\n\n", i + 1));
        }
        
        if page.stroke_count() > 0 || !page.text_items.is_empty() {
            if config.embed_svg {
                let svg = export_svg(page, &config.svg_config);
                md.push_str(&svg);
                md.push_str("\n\n");
            } else {
                let svg_name = format!("{}-page-{}", sanitize_filename(&metadata.title), i + 1);
                md.push_str(&format!("![[{}.svg]]\n\n", svg_name));
            }
        }
        
        // Text content
        if config.include_text && !page.text_items.is_empty() {
            for text in &page.text_items {
                md.push_str(&text.text);
                md.push_str("\n\n");
            }
        }
    }
    
    md
}

/// Parse wikilinks from Markdown content
pub fn extract_wikilinks(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut chars = content.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '[' && chars.peek() == Some(&'[') {
            chars.next(); // consume second [
            let mut link = String::new();
            
            while let Some(c) = chars.next() {
                if c == ']' && chars.peek() == Some(&']') {
                    chars.next();
                    if !link.is_empty() {
                        // Handle display text: [[link|display]]
                        let link_part = link.split('|').next().unwrap_or(&link);
                        links.push(link_part.to_string());
                    }
                    break;
                }
                link.push(c);
            }
        }
    }
    
    links
}

/// Sanitize a string for use as a filename
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Escape string for YAML
fn escape_yaml(s: &str) -> String {
    if s.contains('\n') || s.contains(':') || s.contains('#') {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_wikilinks() {
        let content = "Check [[My Note]] and [[Other|Display Text]]";
        let links = extract_wikilinks(content);
        assert_eq!(links, vec!["My Note", "Other"]);
    }
    
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My/Note:Test"), "My-Note-Test");
        assert_eq!(sanitize_filename("Normal Name"), "Normal Name");
    }
}
