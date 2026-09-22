//! Notion API types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Notion API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionConfig {
    /// API integration token
    pub token: String,
    /// Default parent page or database ID
    pub parent_id: Option<String>,
    /// Create pages in database (true) or as subpages (false)
    pub use_database: bool,
    /// Database ID for folder -> database mapping
    pub database_mappings: std::collections::HashMap<String, String>,
}

impl NotionConfig {
    pub fn new(token: String) -> Self {
        Self {
            token,
            parent_id: None,
            use_database: true,
            database_mappings: std::collections::HashMap::new(),
        }
    }
}

/// Notion block types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotionBlock {
    Paragraph {
        paragraph: ParagraphBlock,
    },
    Heading1 {
        heading_1: HeadingBlock,
    },
    Heading2 {
        heading_2: HeadingBlock,
    },
    Heading3 {
        heading_3: HeadingBlock,
    },
    BulletedListItem {
        bulleted_list_item: ListItemBlock,
    },
    NumberedListItem {
        numbered_list_item: ListItemBlock,
    },
    Image {
        image: ImageBlock,
    },
    Embed {
        embed: EmbedBlock,
    },
    Divider {
        divider: DividerBlock,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphBlock {
    pub rich_text: Vec<RichText>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingBlock {
    pub rich_text: Vec<RichText>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItemBlock {
    pub rich_text: Vec<RichText>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    #[serde(flatten)]
    pub source: ImageSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageSource {
    External { external: ExternalUrl },
    File { file: NotionFile },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionFile {
    pub url: String,
    pub expiry_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedBlock {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividerBlock {}

/// Rich text element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichText {
    #[serde(rename = "type")]
    pub text_type: String,
    pub text: TextContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Annotations {
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub strikethrough: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub code: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl RichText {
    pub fn plain(content: &str) -> Self {
        Self {
            text_type: "text".to_string(),
            text: TextContent {
                content: content.to_string(),
                link: None,
            },
            annotations: None,
        }
    }
    
    pub fn bold(content: &str) -> Self {
        Self {
            text_type: "text".to_string(),
            text: TextContent {
                content: content.to_string(),
                link: None,
            },
            annotations: Some(Annotations {
                bold: true,
                ..Default::default()
            }),
        }
    }
}

/// Notion page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionPage {
    pub id: String,
    pub created_time: DateTime<Utc>,
    pub last_edited_time: DateTime<Utc>,
    pub archived: bool,
    pub properties: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Parent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Parent {
    DatabaseId { database_id: String },
    PageId { page_id: String },
    Workspace { workspace: bool },
}

/// Create page request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageRequest {
    pub parent: Parent,
    pub properties: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NotionBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Icon {
    Emoji { emoji: String },
    External { external: ExternalUrl },
}

/// Database query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorts: Option<Vec<Sort>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub property: String,
    pub direction: String,
}

/// Sync state for tracking Notion pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionSyncState {
    /// Mapping: remarkable_id -> notion_page_id
    pub page_mappings: std::collections::HashMap<String, String>,
    /// Last sync timestamps
    pub last_synced: std::collections::HashMap<String, DateTime<Utc>>,
}

impl Default for NotionSyncState {
    fn default() -> Self {
        Self {
            page_mappings: std::collections::HashMap::new(),
            last_synced: std::collections::HashMap::new(),
        }
    }
}
