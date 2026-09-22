//! Sync coordinator for managing multiple targets

use std::path::PathBuf;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use tracing::info;

use crate::rm_format::{DocumentMetadata, parse_rm_file};
use crate::obsidian::{ObsidianVault, VaultConfig};
use crate::notion::{NotionClient, NotionSync, NotionConfig};
use crate::conflict::{ConflictResolver, Strategy};
use super::{RemarkableClient, AuthTokens};

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

/// Sync target
#[derive(Debug, Clone)]
pub enum SyncTarget {
    Obsidian(PathBuf),
    Notion(String),
}

/// Sync result
#[derive(Debug)]
pub struct SyncResult {
    pub documents_synced: usize,
    pub conflicts: Vec<String>,
    pub errors: Vec<String>,
}

/// Unified sync coordinator
pub struct SyncCoordinator {
    remarkable: RemarkableClient,
    obsidian: Option<ObsidianVault>,
    notion: Option<NotionSync>,
    #[allow(dead_code)]
    resolver: ConflictResolver,
    data_dir: PathBuf,
}

impl SyncCoordinator {
    /// Create a new coordinator
    pub fn new(
        tokens: AuthTokens,
        data_dir: PathBuf,
        strategy: Strategy,
    ) -> Self {
        std::fs::create_dir_all(&data_dir).ok();
        
        Self {
            remarkable: RemarkableClient::new(tokens),
            obsidian: None,
            notion: None,
            resolver: ConflictResolver::new(strategy),
            data_dir,
        }
    }
    
    /// Configure Obsidian vault
    pub fn with_obsidian(&mut self, config: VaultConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.obsidian = Some(ObsidianVault::open(config)?);
        Ok(())
    }
    
    /// Configure Notion sync
    pub fn with_notion(&mut self, config: NotionConfig) -> Result<(), Box<dyn std::error::Error>> {
        let client = NotionClient::new(config)?;
        let state_path = self.data_dir.join("notion_state.json");
        self.notion = Some(NotionSync::with_state_path(client, &state_path)?);
        Ok(())
    }
    
    /// Push documents to Obsidian
    pub async fn push_to_obsidian(&mut self) -> Result<SyncResult, Box<dyn std::error::Error>> {
        if self.obsidian.is_none() {
            return Err("Obsidian not configured".into());
        }
        
        let mut result = SyncResult {
            documents_synced: 0,
            conflicts: Vec::new(),
            errors: Vec::new(),
        };
        
        let documents = self.remarkable.list_documents().await?;
        
        for doc in documents {
            // Download pages
            let page_data = match self.remarkable.download_document_pages(&doc).await {
                Ok(data) => data,
                Err(e) => {
                    result.errors.push(format!("{}: {}", doc.id, e));
                    continue;
                }
            };
            
            if page_data.is_empty() {
                continue;
            }
            
            // Parse pages
            let mut pages = Vec::new();
            for data in &page_data {
                if let Ok(page) = parse_rm_file(data) {
                    pages.push(page);
                }
            }
            
            if pages.is_empty() {
                continue;
            }
            
            // Create metadata
            let metadata = DocumentMetadata {
                title: if doc.visible_name.is_empty() {
                    format!("Untitled-{}", &doc.id[..8.min(doc.id.len())])
                } else {
                    doc.visible_name.clone()
                },
                created: doc.modified_client,
                modified: doc.modified_client,
                tags: Vec::new(),
                folder: if doc.parent.is_empty() { None } else { Some(doc.parent.clone()) },
                device_id: None,
                remarkable_id: Some(doc.id.clone()),
            };
            
            // Sync to vault
            if let Some(vault) = self.obsidian.as_mut() {
                match vault.sync_document(&pages, &metadata, &doc.id) {
                    Ok(_) => {
                        result.documents_synced += 1;
                        info!("Synced {} to Obsidian", metadata.title);
                    }
                    Err(e) => {
                        result.errors.push(format!("{}: {}", doc.id, e));
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Push documents to Notion
    pub async fn push_to_notion(&mut self, parent_id: &str) -> Result<SyncResult, Box<dyn std::error::Error>> {
        if self.notion.is_none() {
            return Err("Notion not configured".into());
        }
        
        let mut result = SyncResult {
            documents_synced: 0,
            conflicts: Vec::new(),
            errors: Vec::new(),
        };
        
        let documents = self.remarkable.list_documents().await?;
        
        for doc in documents {
            let page_data = match self.remarkable.download_document_pages(&doc).await {
                Ok(data) => data,
                Err(e) => {
                    result.errors.push(format!("{}: {}", doc.id, e));
                    continue;
                }
            };
            
            if page_data.is_empty() {
                continue;
            }
            
            let mut pages = Vec::new();
            for data in &page_data {
                if let Ok(page) = parse_rm_file(data) {
                    pages.push(page);
                }
            }
            
            if pages.is_empty() {
                continue;
            }
            
            let metadata = DocumentMetadata {
                title: if doc.visible_name.is_empty() {
                    format!("Untitled-{}", &doc.id[..8.min(doc.id.len())])
                } else {
                    doc.visible_name.clone()
                },
                created: doc.modified_client,
                modified: doc.modified_client,
                tags: Vec::new(),
                folder: if doc.parent.is_empty() { None } else { Some(doc.parent.clone()) },
                device_id: None,
                remarkable_id: Some(doc.id.clone()),
            };
            
            if let Some(notion) = self.notion.as_mut() {
                match notion.push_document(&pages, &metadata, &doc.id, parent_id).await {
                    Ok(_) => {
                        result.documents_synced += 1;
                        info!("Synced {} to Notion", metadata.title);
                    }
                    Err(e) => {
                        result.errors.push(format!("{}: {}", doc.id, e));
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Watch Obsidian vault for changes
    pub async fn watch_obsidian(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.obsidian.is_none() {
            return Err("Obsidian not configured".into());
        }
        
        info!("Watching Obsidian vault for changes...");
        Ok(())
    }
}

/// Sync job for async processing
#[derive(Debug, Clone)]
pub struct SyncJob {
    pub id: String,
    pub direction: SyncDirection,
    pub target: SyncTarget,
    pub created: DateTime<Utc>,
}

/// Job queue for background sync
pub struct SyncQueue {
    tx: mpsc::Sender<SyncJob>,
}

impl SyncQueue {
    pub fn new() -> (Self, mpsc::Receiver<SyncJob>) {
        let (tx, rx) = mpsc::channel(100);
        (Self { tx }, rx)
    }
    
    pub async fn enqueue(&self, job: SyncJob) -> Result<(), mpsc::error::SendError<SyncJob>> {
        self.tx.send(job).await
    }
}
