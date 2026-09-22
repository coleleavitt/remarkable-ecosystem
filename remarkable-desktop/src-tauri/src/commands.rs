//! Tauri command handlers

use std::path::PathBuf;
use tauri::{AppHandle, State};
use tracing::{info, error};
use std::sync::Arc;

use crate::config::{ConflictInfo, ConflictStrategy, DocumentState, SyncConfig, SyncStatusInfo};
use crate::state::AppState;
use crate::sync::{perform_sync, SyncClient};
use crate::error::{Error, Result};

/// Get current sync status
#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatusInfo> {
    let status = state.status.read().await;
    Ok(status.clone())
}

/// Set sync configuration
#[tauri::command]
pub async fn set_sync_config(
    state: State<'_, AppState>,
    config: SyncConfig,
) -> Result<()> {
    let mut current = state.config.write().await;
    *current = config;
    info!("Sync config updated");
    Ok(())
}

/// Get sync configuration
#[tauri::command]
pub async fn get_sync_config(state: State<'_, AppState>) -> Result<SyncConfig> {
    let config = state.config.read().await;
    Ok(config.clone())
}

/// Manually trigger sync
#[tauri::command]
pub async fn start_sync(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    info!("Manual sync triggered");
    
    {
        let status = state.status.read().await;
        if status.is_syncing {
            return Err(Error::Sync("Sync already in progress".into()));
        }
    }
    
    // Clone the Arc pointers from state before spawning
    let config = Arc::clone(&state.config);
    let documents = Arc::clone(&state.documents);
    let conflicts = Arc::clone(&state.conflicts);
    let status = Arc::clone(&state.status);
    let shutdown = Arc::clone(&state.shutdown);
    
    let state_clone = AppState {
        config,
        documents,
        conflicts,
        status,
        shutdown,
    };
    
    tokio::spawn(async move {
        if let Err(e) = perform_sync(&app, &state_clone).await {
            error!("Manual sync failed: {}", e);
        }
    });
    
    Ok(())
}

/// Stop sync
#[tauri::command]
pub async fn stop_sync(state: State<'_, AppState>) -> Result<()> {
    info!("Stopping sync");
    state.request_shutdown().await;
    Ok(())
}

/// Get all documents
#[tauri::command]
pub async fn get_documents(state: State<'_, AppState>) -> Result<Vec<DocumentState>> {
    let docs = state.documents.read().await;
    Ok(docs.values().cloned().collect())
}

/// Get folders only
#[tauri::command]
pub async fn get_folders(state: State<'_, AppState>) -> Result<Vec<DocumentState>> {
    let docs = state.documents.read().await;
    Ok(docs.values()
        .filter(|d| matches!(d.doc_type, crate::config::DocumentType::Folder))
        .cloned()
        .collect())
}

/// Set selected folders to sync
#[tauri::command]
pub async fn set_selected_folders(
    state: State<'_, AppState>,
    folders: Vec<String>,
) -> Result<()> {
    let mut config = state.config.write().await;
    config.selected_folders = folders;
    info!("Selected folders updated");
    Ok(())
}

/// Get current conflicts
#[tauri::command]
pub async fn get_conflicts(state: State<'_, AppState>) -> Result<Vec<ConflictInfo>> {
    let conflicts = state.conflicts.read().await;
    Ok(conflicts.clone())
}

/// Resolve a conflict
#[tauri::command]
pub async fn resolve_conflict(
    app: AppHandle,
    state: State<'_, AppState>,
    document_id: String,
    strategy: ConflictStrategy,
) -> Result<()> {
    info!("Resolving conflict for {} with strategy {:?}", document_id, strategy);
    
    let config = state.config.read().await.clone();
    
    match strategy {
        ConflictStrategy::KeepLocal => {
            // Upload local version
            let local_path = config.local_path.join(&document_id);
            if local_path.exists() {
                let data = std::fs::read(&local_path)?;
                let client = SyncClient::new(config.clone())?;
                client.upload_file(&data, &format!("{}.zip", document_id)).await?;
            }
        }
        ConflictStrategy::KeepRemote => {
            // Download remote version (will happen on next sync)
            let mut docs = state.documents.write().await;
            if let Some(doc) = docs.get_mut(&document_id) {
                doc.local_hash = None; // Force re-download
            }
        }
        ConflictStrategy::KeepBoth => {
            // Rename local and download remote
            let local_path = config.local_path.join(&document_id);
            if local_path.exists() {
                let backup_path = config.local_path.join(format!("{}_local_backup", document_id));
                std::fs::rename(&local_path, &backup_path)?;
            }
        }
        ConflictStrategy::AskUser => {
            // Should not reach here, UI handles this
            return Err(Error::Conflict("User action required".into()));
        }
    }
    
    // Remove conflict
    state.remove_conflict(&document_id).await;
    
    // Clone state for spawn
    let config_arc = Arc::clone(&state.config);
    let documents = Arc::clone(&state.documents);
    let conflicts = Arc::clone(&state.conflicts);
    let status = Arc::clone(&state.status);
    let shutdown = Arc::clone(&state.shutdown);
    
    let state_clone = AppState {
        config: config_arc,
        documents,
        conflicts,
        status,
        shutdown,
    };
    
    tokio::spawn(async move {
        let _ = perform_sync(&app, &state_clone).await;
    });
    
    Ok(())
}

/// Set offline mode
#[tauri::command]
pub async fn set_offline_mode(
    state: State<'_, AppState>,
    offline: bool,
) -> Result<()> {
    info!("Setting offline mode: {}", offline);
    state.set_offline_mode(offline).await;
    Ok(())
}

/// Get offline mode status
#[tauri::command]
pub async fn get_offline_mode(state: State<'_, AppState>) -> Result<bool> {
    let config = state.config.read().await;
    Ok(config.offline_mode)
}

/// Test connection to server
#[tauri::command]
pub async fn test_connection(state: State<'_, AppState>) -> Result<bool> {
    let config = state.config.read().await.clone();
    let client = SyncClient::new(config)?;
    client.test_connection().await
}

/// Export document to local path
#[tauri::command]
pub async fn export_document(
    state: State<'_, AppState>,
    document_id: String,
    export_path: PathBuf,
) -> Result<()> {
    let config = state.config.read().await;
    let local_path = config.local_path.join(&document_id);
    
    if !local_path.exists() {
        return Err(Error::NotFound(format!("Document {} not found locally", document_id)));
    }
    
    std::fs::copy(&local_path, &export_path)?;
    info!("Exported {} to {:?}", document_id, export_path);
    
    Ok(())
}

/// Import document from local path
#[tauri::command]
pub async fn import_document(
    _app: AppHandle,
    state: State<'_, AppState>,
    import_path: PathBuf,
    parent_id: Option<String>,
) -> Result<String> {
    let config = state.config.read().await.clone();
    
    // Generate new document ID
    let doc_id = uuid::Uuid::new_v4().to_string();
    
    // Copy to local storage
    let local_path = config.local_path.join(&doc_id);
    std::fs::create_dir_all(&config.local_path)?;
    std::fs::copy(&import_path, &local_path)?;
    
    // Add to document list
    let doc_state = DocumentState {
        id: doc_id.clone(),
        name: import_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Imported Document".into()),
        parent_id,
        doc_type: crate::config::DocumentType::Document,
        local_hash: Some(crate::sync::compute_hash(&std::fs::read(&local_path)?)),
        remote_hash: None,
        last_synced: None,
        last_modified_local: Some(chrono::Utc::now()),
        last_modified_remote: None,
        sync_status: crate::config::SyncStatus::LocalOnly,
    };
    
    {
        let mut docs = state.documents.write().await;
        docs.insert(doc_id.clone(), doc_state);
    }
    
    info!("Imported document {} from {:?}", doc_id, import_path);
    
    // Upload to server
    let data = std::fs::read(&local_path)?;
    let client = SyncClient::new(config)?;
    client.upload_file(&data, &format!("{}.zip", doc_id)).await?;
    
    Ok(doc_id)
}
