//! Integration with remarkable-server (local sync server).
//!
//! Provides hooks for:
//! - Loading documents from the sync server
//! - Saving changes back to the server
//! - Real-time notifications via MQTT

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::crdt::CrdtDocument;

/// Errors from remarkable-server integration.
#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("Document not found: {0}")]
    DocumentNotFound(Uuid),

    #[error("Server connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid document format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Configuration for remarkable-server integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerIntegrationConfig {
    /// Base URL of remarkable-server (e.g., "http://localhost:8080").
    pub server_url: String,
    /// Path to local document storage.
    pub data_dir: PathBuf,
    /// MQTT broker URL for real-time notifications.
    pub mqtt_url: Option<String>,
    /// Authentication token (if required).
    pub auth_token: Option<String>,
}

impl Default for ServerIntegrationConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8080".to_string(),
            data_dir: PathBuf::from("/home/root/.local/share/remarkable/xochitl"),
            mqtt_url: None,
            auth_token: None,
        }
    }
}

/// Client for remarkable-server integration.
pub struct RemarkableServerClient {
    config: ServerIntegrationConfig,
}

impl RemarkableServerClient {
    pub fn new(config: ServerIntegrationConfig) -> Self {
        Self { config }
    }

    /// Load a document from the local data directory.
    ///
    /// This reads the .content, .metadata, and .rm files for a document.
    pub async fn load_document(&self, doc_id: Uuid) -> Result<CrdtDocument, IntegrationError> {
        let doc_dir = self.config.data_dir.join(doc_id.to_string());
        
        if !doc_dir.exists() {
            return Err(IntegrationError::DocumentNotFound(doc_id));
        }

        // Read metadata
        let metadata_path = doc_dir.with_extension("metadata");
        let metadata: DocumentMetadata = if metadata_path.exists() {
            let content = tokio::fs::read_to_string(&metadata_path).await?;
            serde_json::from_str(&content)?
        } else {
            DocumentMetadata::default()
        };

        // Create document from metadata
        let document = CrdtDocument::new(doc_id, metadata.visible_name);

        // TODO: Load pages and strokes from .rm files
        // This would parse the binary .rm format (version 6)

        Ok(document)
    }

    /// Save a document to the local data directory.
    pub async fn save_document(&self, doc: &CrdtDocument) -> Result<(), IntegrationError> {
        let doc_dir = self.config.data_dir.join(doc.id.to_string());
        tokio::fs::create_dir_all(&doc_dir).await?;

        // Save metadata
        let metadata = DocumentMetadata {
            visible_name: doc.name.clone(),
            last_modified: chrono::Utc::now().to_rfc3339(),
            ..Default::default()
        };
        let metadata_path = doc_dir.with_extension("metadata");
        let content = serde_json::to_string_pretty(&metadata)?;
        tokio::fs::write(&metadata_path, content).await?;

        // TODO: Convert pages/strokes to .rm format and save

        Ok(())
    }

    /// List all documents in the data directory.
    pub async fn list_documents(&self) -> Result<Vec<DocumentInfo>, IntegrationError> {
        let mut documents = Vec::new();

        let mut entries = tokio::fs::read_dir(&self.config.data_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "metadata").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(id) = Uuid::parse_str(stem) {
                        let content = tokio::fs::read_to_string(&path).await?;
                        if let Ok(metadata) = serde_json::from_str::<DocumentMetadata>(&content) {
                            documents.push(DocumentInfo {
                                id,
                                name: metadata.visible_name,
                                doc_type: metadata.doc_type,
                                last_modified: metadata.last_modified,
                            });
                        }
                    }
                }
            }
        }

        Ok(documents)
    }

    /// Notify remarkable-server of a document change.
    ///
    /// This triggers a sync if the server is connected to the cloud.
    pub async fn notify_change(&self, doc_id: Uuid) -> Result<(), IntegrationError> {
        // POST to /api/documents/{id}/sync
        let url = format!("{}/api/documents/{}/sync", self.config.server_url, doc_id);
        
        let client = reqwest::Client::new();
        let mut req = client.post(&url);
        
        if let Some(ref token) = self.config.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await.map_err(|e| {
            IntegrationError::ConnectionFailed(e.to_string())
        })?;

        if !response.status().is_success() {
            return Err(IntegrationError::ConnectionFailed(
                format!("Server returned {}", response.status())
            ));
        }

        Ok(())
    }

    /// Subscribe to real-time document changes via MQTT.
    pub async fn subscribe_changes<F>(&self, callback: F) -> Result<(), IntegrationError>
    where
        F: Fn(Uuid, DocumentChangeEvent) + Send + Sync + 'static,
    {
        let mqtt_url = self.config.mqtt_url.as_ref().ok_or_else(|| {
            IntegrationError::ConnectionFailed("MQTT URL not configured".to_string())
        })?;

        // TODO: Connect to MQTT broker and subscribe to document change topics
        // Topic format: remarkable/documents/{doc_id}/changes

        tracing::info!("Would subscribe to MQTT at {}", mqtt_url);
        
        Ok(())
    }
}

/// Document metadata from .metadata files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMetadata {
    #[serde(default)]
    pub visible_name: String,
    #[serde(default = "default_doc_type")]
    pub doc_type: String,
    #[serde(default)]
    pub parent: String,
    #[serde(default)]
    pub last_modified: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub synced: bool,
}

fn default_doc_type() -> String {
    "DocumentType".to_string()
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            visible_name: "Untitled".to_string(),
            doc_type: default_doc_type(),
            parent: String::new(),
            last_modified: chrono::Utc::now().to_rfc3339(),
            version: 1,
            synced: false,
        }
    }
}

/// Basic document info for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub id: Uuid,
    pub name: String,
    pub doc_type: String,
    pub last_modified: String,
}

/// Real-time change event from MQTT.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentChangeEvent {
    /// Document was modified.
    Modified {
        generation: u64,
    },
    /// Document was synced with cloud.
    Synced {
        hash: String,
    },
    /// Document was deleted.
    Deleted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerIntegrationConfig::default();
        assert_eq!(config.server_url, "http://localhost:8080");
    }

    #[test]
    fn test_document_metadata_serde() {
        let json = r#"{"visibleName":"Test Doc","type":"DocumentType"}"#;
        let metadata: DocumentMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.visible_name, "Test Doc");
    }
}
