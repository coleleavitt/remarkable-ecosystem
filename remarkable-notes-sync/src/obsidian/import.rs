//! Import Markdown back to reMarkable format

use std::path::Path;
use std::fs;
use thiserror::Error;

use crate::rm_format::{RmPage, TextItem, CrdtId, Layer, SCREEN_WIDTH, SCREEN_HEIGHT};

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Unsupported format")]
    UnsupportedFormat,
}

/// Configuration for Markdown import
#[derive(Debug, Clone)]
pub struct ImportConfig {
    /// Font size for text
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Left margin
    pub margin_left: f64,
    /// Top margin
    pub margin_top: f64,
    /// Maximum line width
    pub max_width: f64,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            font_size: 24.0,
            line_height: 1.5,
            margin_left: 50.0,
            margin_top: 100.0,
            max_width: SCREEN_WIDTH - 100.0,
        }
    }
}

/// Parsed Markdown content
#[derive(Debug, Clone)]
pub struct ParsedMarkdown {
    /// Frontmatter key-value pairs
    pub frontmatter: std::collections::HashMap<String, String>,
    /// Main content
    pub content: String,
    /// Extracted title
    pub title: Option<String>,
    /// reMarkable ID from frontmatter
    pub remarkable_id: Option<String>,
}

/// Parse a Markdown file
pub fn parse_markdown(content: &str) -> Result<ParsedMarkdown, ImportError> {
    let mut result = ParsedMarkdown {
        frontmatter: std::collections::HashMap::new(),
        content: String::new(),
        title: None,
        remarkable_id: None,
    };
    
    let content = content.trim();
    
    // Parse frontmatter if present
    if content.starts_with("---") {
        if let Some(end_pos) = content[3..].find("---") {
            let frontmatter_str = &content[3..3 + end_pos];
            
            for line in frontmatter_str.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_string();
                    let value = value.trim().trim_matches('"').to_string();
                    
                    if key == "remarkable_id" {
                        result.remarkable_id = Some(value.clone());
                    }
                    if key == "title" {
                        result.title = Some(value.clone());
                    }
                    
                    result.frontmatter.insert(key, value);
                }
            }
            
            result.content = content[3 + end_pos + 3..].trim().to_string();
        } else {
            result.content = content.to_string();
        }
    } else {
        result.content = content.to_string();
    }
    
    // Extract title from first heading if not in frontmatter
    if result.title.is_none() {
        for line in result.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                result.title = Some(trimmed[2..].trim().to_string());
                break;
            }
        }
    }
    
    Ok(result)
}

/// Convert Markdown content to reMarkable page with text items
pub fn markdown_to_rm(parsed: &ParsedMarkdown, config: &ImportConfig) -> RmPage {
    let mut page = RmPage::new();
    page.layers = vec![Layer::default()];
    
    let mut y = config.margin_top;
    let mut item_id: u64 = 1;
    
    // Split content into lines and create text items
    for line in parsed.content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines but add spacing
        if trimmed.is_empty() {
            y += config.font_size as f64 * config.line_height as f64;
            continue;
        }
        
        // Determine text style based on Markdown syntax
        let (text, font_size) = if trimmed.starts_with("# ") {
            (trimmed[2..].to_string(), config.font_size * 1.5)
        } else if trimmed.starts_with("## ") {
            (trimmed[3..].to_string(), config.font_size * 1.3)
        } else if trimmed.starts_with("### ") {
            (trimmed[4..].to_string(), config.font_size * 1.15)
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            (format!("• {}", &trimmed[2..]), config.font_size)
        } else {
            // Remove inline formatting for plain text
            let clean = trimmed
                .replace("**", "")
                .replace("__", "")
                .replace("*", "")
                .replace("_", "")
                .replace("`", "");
            (clean, config.font_size)
        };
        
        // Skip SVG embeds and image references
        if text.starts_with("<svg") || text.starts_with("![[") || text.starts_with("![") {
            continue;
        }
        
        // Create text item
        let text_item = TextItem {
            id: CrdtId::new(1, item_id),
            text,
            x: config.margin_left,
            y,
            width: config.max_width,
            height: font_size as f64 * config.line_height as f64,
            font_size,
        };
        
        page.text_items.push(text_item);
        y += font_size as f64 * config.line_height as f64;
        item_id += 1;
        
        // Check if we need a new page
        if y > SCREEN_HEIGHT - 100.0 {
            // In a real implementation, this would create multiple pages
            break;
        }
    }
    
    page
}

/// Import a Markdown file and convert to reMarkable pages
pub fn import_markdown_file(path: &Path, config: &ImportConfig) -> Result<(ParsedMarkdown, Vec<RmPage>), ImportError> {
    let content = fs::read_to_string(path)?;
    let parsed = parse_markdown(&content)?;
    let page = markdown_to_rm(&parsed, config);
    
    Ok((parsed, vec![page]))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_markdown() {
        let content = r#"---
title: Test Note
remarkable_id: abc123
---

# Test Note

This is a test."#;
        
        let parsed = parse_markdown(content).unwrap();
        assert_eq!(parsed.title, Some("Test Note".to_string()));
        assert_eq!(parsed.remarkable_id, Some("abc123".to_string()));
    }
    
    #[test]
    fn test_markdown_to_rm() {
        let content = "# Hello\n\nWorld";
        let parsed = parse_markdown(content).unwrap();
        let config = ImportConfig::default();
        let page = markdown_to_rm(&parsed, &config);
        
        assert!(!page.text_items.is_empty());
    }
}
