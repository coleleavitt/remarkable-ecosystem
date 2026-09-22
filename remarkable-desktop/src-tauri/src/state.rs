//! Application state management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::config::{ConflictInfo, DocumentState, SyncConfig, SyncStatusInfo};

/// Shared application state
pub struct AppState {
    /// Sync configuration
    pub config: Arc<RwLock<SyncConfig>>,
    
    /// Document states
    pub documents: Arc<RwLock<HashMap<String, DocumentState>>>,
    
    /// Active conflicts
    pub conflicts: Arc<RwLock<Vec<ConflictInfo>>>,
    
    /// Sync status
    pub status: Arc<RwLock<SyncStatusInfo>>,
    
    /// Shutdown signal
    pub shutdown: Arc<RwLock<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(SyncConfig::default())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            conflicts: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(SyncStatusInfo {
                is_running: false,
                is_syncing: false,
                last_sync: None,
                next_sync: None,
                documents_synced: 0,
                documents_pending: 0,
                conflicts_count: 0,
                errors: vec![],
                offline_mode: false,
            })),
            shutdown: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Update sync status
    pub async fn set_syncing(&self, syncing: bool) {
        let mut status = self.status.write().await;
        status.is_syncing = syncing;
        if !syncing {
            status.last_sync = Some(Utc::now());
        }
    }
    
    /// Set next sync time
    pub async fn set_next_sync(&self, time: DateTime<Utc>) {
        let mut status = self.status.write().await;
        status.next_sync = Some(time);
    }
    
    /// Add error
    pub async fn add_error(&self, error: String) {
        let mut status = self.status.write().await;
        status.errors.push(error);
        // Keep only last 10 errors
        if status.errors.len() > 10 {
            status.errors.remove(0);
        }
    }
    
    /// Clear errors
    pub async fn clear_errors(&self) {
        let mut status = self.status.write().await;
        status.errors.clear();
    }
    
    /// Add conflict
    pub async fn add_conflict(&self, conflict: ConflictInfo) {
        let mut conflicts = self.conflicts.write().await;
        // Remove existing conflict for same document
        conflicts.retain(|c| c.document_id != conflict.document_id);
        conflicts.push(conflict);
        
        // Update status
        let mut status = self.status.write().await;
        status.conflicts_count = conflicts.len();
    }
    
    /// Remove conflict
    pub async fn remove_conflict(&self, document_id: &str) {
        let mut conflicts = self.conflicts.write().await;
        conflicts.retain(|c| c.document_id != document_id);
        
        let mut status = self.status.write().await;
        status.conflicts_count = conflicts.len();
    }
    
    /// Update document counts
    pub async fn update_counts(&self, synced: usize, pending: usize) {
        let mut status = self.status.write().await;
        status.documents_synced = synced;
        status.documents_pending = pending;
    }
    
    /// Set offline mode
    pub async fn set_offline_mode(&self, offline: bool) {
        let mut config = self.config.write().await;
        config.offline_mode = offline;
        
        let mut status = self.status.write().await;
        status.offline_mode = offline;
    }
    
    /// Check shutdown requested
    pub async fn should_shutdown(&self) -> bool {
        *self.shutdown.read().await
    }
    
    /// Request shutdown
    pub async fn request_shutdown(&self) {
        *self.shutdown.write().await = true;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
