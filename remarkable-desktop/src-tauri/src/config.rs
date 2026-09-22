//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Server URL (cloud or local)
    pub server_url: String,
    
    /// Device token for authentication
    pub device_token: Option<String>,
    
    /// Sync interval in seconds
    pub sync_interval_secs: u64,
    
    /// Local storage path
    pub local_path: PathBuf,
    
    /// Selected folders to sync (empty = all)
    pub selected_folders: Vec<String>,
    
    /// Enable offline mode
    pub offline_mode: bool,
    
    /// Auto-start sync on launch
    pub auto_start: bool,
    
    /// Show notifications
    pub notifications_enabled: bool,
    
    /// Conflict resolution strategy
    pub conflict_strategy: ConflictStrategy,
}

impl Default for SyncConfig {
    fn default() -> Self {
        let local_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("remarkable-sync")
            .join("documents");
        
        Self {
            server_url: "https://tectonic.remarkable.com".into(),
            device_token: None,
            sync_interval_secs: 300, // 5 minutes
            local_path,
            selected_folders: vec![],
            offline_mode: false,
            auto_start: true,
            notifications_enabled: true,
            conflict_strategy: ConflictStrategy::AskUser,
        }
    }
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    /// Ask user to resolve
    AskUser,
    /// Keep local version
    KeepLocal,
    /// Keep remote version  
    KeepRemote,
    /// Keep both (create copy)
    KeepBoth,
}

/// Document sync state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub doc_type: DocumentType,
    pub local_hash: Option<String>,
    pub remote_hash: Option<String>,
    pub last_synced: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified_local: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified_remote: Option<chrono::DateTime<chrono::Utc>>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Folder,
    Document,
    Pdf,
    Epub,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Synced,
    LocalOnly,
    RemoteOnly,
    Modified,
    Conflict,
    Syncing,
    Error,
}

/// Conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub document_id: String,
    pub document_name: String,
    pub local_modified: chrono::DateTime<chrono::Utc>,
    pub remote_modified: chrono::DateTime<chrono::Utc>,
    pub local_size: u64,
    pub remote_size: u64,
}

/// Overall sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusInfo {
    pub is_running: bool,
    pub is_syncing: bool,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub next_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub documents_synced: usize,
    pub documents_pending: usize,
    pub conflicts_count: usize,
    pub errors: Vec<String>,
    pub offline_mode: bool,
}
