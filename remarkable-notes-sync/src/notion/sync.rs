//! Notion synchronization logic

use std::path::Path;
use chrono::{DateTime, Utc};
use serde_json::json;

use super::client::*;
use super::types::*;
use crate::rm_format::{RmPage, DocumentMetadata, export_svg_data_url, SvgConfig};

/// Sync a reMarkable document to Notion
pub struct NotionSync {
    client: NotionClient,
    state: NotionSyncState,
    state_path: Option<std::path::PathBuf>,
}

impl NotionSync {
    /// Create a new sync handler
    pub fn new(client: NotionClient) -> Self {
        Self {
            client,
            state: NotionSyncState::default(),
            state_path: None,
        }
    }
    
    /// Create with persistent state
    pub fn with_state_path(client: NotionClient, path: impl AsRef<Path>) -> std::io::Result<Self> {
        let state_path = path.as_ref().to_path_buf();
        
        let state = if state_path.exists() {
            let content = std::fs::read_to_string(&state_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            NotionSyncState::default()
        };
        
        Ok(Self {
            client,
            state,
            state_path: Some(state_path),
        })
    }
    
    /// Save state to disk
    pub fn save_state(&self) -> std::io::Result<()> {
        if let Some(path) = &self.state_path {
            let content = serde_json::to_string_pretty(&self.state)?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }
    
    /// Push a document to Notion
    pub async fn push_document(
        &mut self,
        pages: &[RmPage],
        metadata: &DocumentMetadata,
        remarkable_id: &str,
        parent_id: &str,
    ) -> Result<String, NotionError> {
        // Check if page already exists
        if let Some(notion_id) = self.state.page_mappings.get(remarkable_id) {
            // Update existing page
            self.update_page(notion_id, pages, metadata).await?;
            return Ok(notion_id.clone());
        }
        
        // Create new page
        let notion_id = self.create_page(pages, metadata, parent_id).await?;
        
        // Update state
        self.state.page_mappings.insert(remarkable_id.to_string(), notion_id.clone());
        self.state.last_synced.insert(remarkable_id.to_string(), Utc::now());
        self.save_state().map_err(|e| NotionError::Config(e.to_string()))?;
        
        Ok(notion_id)
    }
    
    async fn create_page(
        &self,
        pages: &[RmPage],
        metadata: &DocumentMetadata,
        parent_id: &str,
    ) -> Result<String, NotionError> {
        // Build page content
        let blocks = self.build_blocks(pages, metadata);
        
        // Create page properties
        let properties = json!({
            "title": {
                "title": [{
                    "type": "text",
                    "text": { "content": &metadata.title }
                }]
            }
        });
        
        // Determine parent
        let parent = Parent::PageId {
            page_id: parent_id.to_string(),
        };
        
        let request = CreatePageRequest {
            parent,
            properties,
            children: Some(blocks),
            icon: Some(Icon::Emoji { emoji: "📝".to_string() }),
        };
        
        let page = self.client.create_page(request).await?;
        Ok(page.id)
    }
    
    async fn update_page(
        &self,
        page_id: &str,
        pages: &[RmPage],
        metadata: &DocumentMetadata,
    ) -> Result<(), NotionError> {
        // Clear existing blocks
        self.client.clear_blocks(page_id).await?;
        
        // Add new blocks
        let blocks = self.build_blocks(pages, metadata);
        self.client.append_blocks(page_id, blocks).await?;
        
        Ok(())
    }
    
    fn build_blocks(&self, pages: &[RmPage], metadata: &DocumentMetadata) -> Vec<NotionBlock> {
        let mut blocks = Vec::new();
        
        // Metadata header
        blocks.push(NotionBlock::Heading2 {
            heading_2: HeadingBlock {
                rich_text: vec![RichText::plain(&metadata.title)],
            },
        });
        
        // Add metadata paragraph
        let mut meta_parts = Vec::new();
        if let Some(created) = metadata.created {
            meta_parts.push(format!("Created: {}", created.format("%Y-%m-%d")));
        }
        if let Some(folder) = &metadata.folder {
            meta_parts.push(format!("Folder: {}", folder));
        }
        
        if !meta_parts.is_empty() {
            blocks.push(NotionBlock::Paragraph {
                paragraph: ParagraphBlock {
                    rich_text: vec![RichText::plain(&meta_parts.join(" | "))],
                },
            });
        }
        
        blocks.push(NotionBlock::Divider {
            divider: DividerBlock {},
        });
        
        // Add page content
        let svg_config = SvgConfig::default();
        
        for (i, page) in pages.iter().enumerate() {
            if pages.len() > 1 {
                blocks.push(NotionBlock::Heading3 {
                    heading_3: HeadingBlock {
                        rich_text: vec![RichText::plain(&format!("Page {}", i + 1))],
                    },
                });
            }
            
            // Add SVG as image (data URL)
            if page.stroke_count() > 0 {
                let svg_data_url = export_svg_data_url(page, &svg_config);
                
                blocks.push(NotionBlock::Image {
                    image: ImageBlock {
                        source: ImageSource::External {
                            external: ExternalUrl {
                                url: svg_data_url,
                            },
                        },
                    },
                });
            }
            
            // Add text content
            for text in &page.text_items {
                blocks.push(NotionBlock::Paragraph {
                    paragraph: ParagraphBlock {
                        rich_text: vec![RichText::plain(&text.text)],
                    },
                });
            }
        }
        
        // Footer
        blocks.push(NotionBlock::Divider {
            divider: DividerBlock {},
        });
        
        blocks.push(NotionBlock::Paragraph {
            paragraph: ParagraphBlock {
                rich_text: vec![RichText::plain("Synced from reMarkable")],
            },
        });
        
        blocks
    }
    
    /// Delete a document from Notion
    pub async fn delete_document(&mut self, remarkable_id: &str) -> Result<(), NotionError> {
        if let Some(notion_id) = self.state.page_mappings.remove(remarkable_id) {
            self.client.archive_page(&notion_id).await?;
            self.state.last_synced.remove(remarkable_id);
            self.save_state().map_err(|e| NotionError::Config(e.to_string()))?;
        }
        Ok(())
    }
    
    /// Get the Notion page ID for a reMarkable document
    pub fn get_notion_id(&self, remarkable_id: &str) -> Option<&String> {
        self.state.page_mappings.get(remarkable_id)
    }
    
    /// Get last sync time
    pub fn last_synced(&self, remarkable_id: &str) -> Option<&DateTime<Utc>> {
        self.state.last_synced.get(remarkable_id)
    }
}

/// Map reMarkable folders to Notion databases
pub struct FolderMapper {
    mappings: std::collections::HashMap<String, String>,
}

impl FolderMapper {
    pub fn new() -> Self {
        Self {
            mappings: std::collections::HashMap::new(),
        }
    }
    
    pub fn add_mapping(&mut self, rm_folder: &str, notion_db: &str) {
        self.mappings.insert(rm_folder.to_string(), notion_db.to_string());
    }
    
    pub fn get_database(&self, rm_folder: &str) -> Option<&String> {
        self.mappings.get(rm_folder)
    }
}

impl Default for FolderMapper {
    fn default() -> Self {
        Self::new()
    }
}
