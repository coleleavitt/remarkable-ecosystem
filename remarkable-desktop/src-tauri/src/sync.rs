//! Sync operations with reMarkable cloud/server

// use std::collections::HashMap;  // Reserved for future use
// use std::path::PathBuf;  // Reserved for future use
use chrono::{Duration, Utc};
use reqwest::Client;
use sha2::{Sha256, Digest};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, error, info, warn};

use crate::config::{ConflictInfo, DocumentState, DocumentType, SyncConfig, SyncStatus};
use crate::state::AppState;
use crate::error::{Error, Result};

/// Sync client for reMarkable API
pub struct SyncClient {
    client: Client,
    config: SyncConfig,
}

impl SyncClient {
    pub fn new(config: SyncConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        Ok(Self { client, config })
    }
    
    /// Get root sync state from server
    pub async fn get_root(&self) -> Result<SyncRoot> {
        let url = format!("{}/sync/v3/root", self.config.server_url);
        
        let mut req = self.client.get(&url);
        
        if let Some(token) = &self.config.device_token {
            req = req.bearer_auth(token);
        }
        
        let resp = req.send().await?;
        
        if !resp.status().is_success() {
            return Err(Error::Sync(format!(
                "Failed to get root: {}",
                resp.status()
            )));
        }
        
        let root: SyncRoot = resp.json().await?;
        Ok(root)
    }
    
    /// Download file by hash
    pub async fn download_file(&self, hash: &str, filename: &str) -> Result<Vec<u8>> {
        let url = format!("{}/sync/v3/files/{}", self.config.server_url, hash);
        
        let mut req = self.client.get(&url)
            .header("rm-filename", filename);
        
        if let Some(token) = &self.config.device_token {
            req = req.bearer_auth(token);
        }
        
        let resp = req.send().await?;
        
        if !resp.status().is_success() {
            return Err(Error::Sync(format!(
                "Failed to download {}: {}",
                filename,
                resp.status()
            )));
        }
        
        let data = resp.bytes().await?.to_vec();
        
        // Verify hash
        let actual_hash = compute_hash(&data);
        if actual_hash != hash {
            warn!("Hash mismatch for {}: expected {}, got {}", filename, hash, actual_hash);
        }
        
        Ok(data)
    }
    
    /// Upload file
    pub async fn upload_file(&self, data: &[u8], filename: &str) -> Result<String> {
        let hash = compute_hash(data);
        let url = format!("{}/sync/v3/files/{}", self.config.server_url, hash);
        
        let mut req = self.client.put(&url)
            .header("rm-filename", filename)
            .header("content-type", "application/octet-stream")
            .body(data.to_vec());
        
        if let Some(token) = &self.config.device_token {
            req = req.bearer_auth(token);
        }
        
        let resp = req.send().await?;
        
        if !resp.status().is_success() {
            return Err(Error::Sync(format!(
                "Failed to upload {}: {}",
                filename,
                resp.status()
            )));
        }
        
        Ok(hash)
    }
    
    /// Get document index
    pub async fn get_document_index(&self) -> Result<Vec<DocumentInfo>> {
        let root = self.get_root().await?;
        
        // Download root.docSchema
        let schema_data = self.download_file(&root.hash, "root.docSchema").await?;
        let schema: RootSchema = serde_json::from_slice(&schema_data)?;
        
        // Download index file
        let index_data = self.download_file(&schema.files[0].hash, &schema.files[0].name).await?;
        let index: DocumentIndex = serde_json::from_slice(&index_data)?;
        
        Ok(index.items)
    }
    
    /// Test connection to server
    pub async fn test_connection(&self) -> Result<bool> {
        match self.get_root().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Connection test failed: {}", e);
                Err(e)
            }
        }
    }
}

/// Sync root response
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct SyncRoot {
    pub hash: String,
    pub generation: u64,
    #[serde(rename = "schemaVersion")]
    pub schema_version: Option<u32>,
}

/// Root schema structure
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct RootSchema {
    pub files: Vec<FileRef>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct FileRef {
    pub name: String,
    pub hash: String,
}

/// Document index
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct DocumentIndex {
    pub items: Vec<DocumentInfo>,
}

/// Document info from index
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentInfo {
    pub id: String,
    #[serde(rename = "visibleName")]
    pub visible_name: String,
    #[serde(rename = "type")]
    pub doc_type: String,
    pub parent: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    pub hash: Option<String>,
}

impl DocumentInfo {
    pub fn to_state(&self) -> DocumentState {
        let doc_type = match self.doc_type.as_str() {
            "CollectionType" => DocumentType::Folder,
            "DocumentType" => DocumentType::Document,
            _ => DocumentType::Document,
        };
        
        DocumentState {
            id: self.id.clone(),
            name: self.visible_name.clone(),
            parent_id: self.parent.clone(),
            doc_type,
            local_hash: None,
            remote_hash: self.hash.clone(),
            last_synced: None,
            last_modified_local: None,
            last_modified_remote: self.last_modified.as_ref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))
            }),
            sync_status: SyncStatus::RemoteOnly,
        }
    }
}

/// Compute SHA256 hash of data
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Start background sync loop
pub async fn start_background_sync(app: AppHandle) {
    info!("Starting background sync");
    
    let state = app.state::<AppState>();
    
    // Mark as running
    {
        let mut status = state.status.write().await;
        status.is_running = true;
    }
    
    // Initial sync
    if let Err(e) = perform_sync(&app, &state).await {
        error!("Initial sync failed: {}", e);
        state.add_error(e.to_string()).await;
    }
    
    // Sync loop
    let config = state.config.read().await;
    let interval_secs = config.sync_interval_secs;
    drop(config);
    
    let mut timer = interval(TokioDuration::from_secs(interval_secs));
    
    loop {
        timer.tick().await;
        
        if state.should_shutdown().await {
            info!("Shutdown requested, stopping sync");
            break;
        }
        
        let config = state.config.read().await;
        if config.offline_mode {
            debug!("Offline mode, skipping sync");
            continue;
        }
        drop(config);
        
        if let Err(e) = perform_sync(&app, &state).await {
            error!("Sync failed: {}", e);
            state.add_error(e.to_string()).await;
            
            // Emit error event
            let _ = app.emit("sync-error", e.to_string());
        }
    }
    
    // Mark as stopped
    {
        let mut status = state.status.write().await;
        status.is_running = false;
    }
}

/// Perform a single sync operation
pub async fn perform_sync(app: &AppHandle, state: &AppState) -> Result<()> {
    info!("Starting sync");
    state.set_syncing(true).await;
    
    // Emit sync started event
    let _ = app.emit("sync-started", ());
    
    let config = state.config.read().await.clone();
    
    // Create sync client
    let client = SyncClient::new(config.clone())?;
    
    // Get remote document list
    let remote_docs = match client.get_document_index().await {
        Ok(docs) => docs,
        Err(e) => {
            state.set_syncing(false).await;
            let _ = app.emit("sync-completed", false);
            return Err(e);
        }
    };
    
    info!("Found {} remote documents", remote_docs.len());
    
    // Update document states
    let mut docs = state.documents.write().await;
    let mut synced = 0;
    let mut pending = 0;
    
    for remote_doc in &remote_docs {
        let doc_state = remote_doc.to_state();
        
        // Check if we should sync this folder
        if !config.selected_folders.is_empty() {
            if let Some(parent) = &doc_state.parent_id {
                if !config.selected_folders.contains(parent) && !config.selected_folders.contains(&doc_state.id) {
                    continue;
                }
            }
        }
        
        // Check for conflicts
        if let Some(existing) = docs.get(&doc_state.id) {
            if existing.local_hash.is_some() && existing.remote_hash != doc_state.remote_hash {
                if let (Some(local_mod), Some(remote_mod)) = (existing.last_modified_local, doc_state.last_modified_remote) {
                    if local_mod != remote_mod {
                        // Conflict detected
                        let conflict = ConflictInfo {
                            document_id: doc_state.id.clone(),
                            document_name: doc_state.name.clone(),
                            local_modified: local_mod,
                            remote_modified: remote_mod,
                            local_size: 0, // TODO: get actual sizes
                            remote_size: 0,
                        };
                        state.add_conflict(conflict).await;
                        pending += 1;
                        continue;
                    }
                }
            }
        }
        
        // Download document if needed
        if doc_state.doc_type != DocumentType::Folder {
            if let Some(hash) = &doc_state.remote_hash {
                let local_path = config.local_path.join(&doc_state.id);
                
                if !local_path.exists() || docs.get(&doc_state.id).map(|d| d.remote_hash.as_ref() != Some(hash)).unwrap_or(true) {
                    // Download
                    match client.download_file(hash, &format!("{}.zip", doc_state.id)).await {
                        Ok(data) => {
                            std::fs::create_dir_all(&config.local_path)?;
                            std::fs::write(&local_path, &data)?;
                            info!("Downloaded {} ({})", doc_state.name, doc_state.id);
                        }
                        Err(e) => {
                            warn!("Failed to download {}: {}", doc_state.name, e);
                            pending += 1;
                            continue;
                        }
                    }
                }
            }
        }
        
        synced += 1;
        docs.insert(doc_state.id.clone(), doc_state);
    }
    
    drop(docs);
    
    state.update_counts(synced, pending).await;
    state.set_syncing(false).await;
    
    // Schedule next sync
    let next_sync = Utc::now() + Duration::seconds(config.sync_interval_secs as i64);
    state.set_next_sync(next_sync).await;
    
    // Emit sync completed event
    let _ = app.emit("sync-completed", true);
    
    info!("Sync completed: {} synced, {} pending", synced, pending);
    
    // Show notification if enabled
    if config.notifications_enabled {
        let _ = app.emit("show-notification", format!(
            "Sync completed: {} documents synced", synced
        ));
    }
    
    Ok(())
}
