//! reMarkable cloud sync client

use std::path::Path;
use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};

const DISCOVERY_URL: &str = "https://internal.cloud.remarkable.com/discovery/v1/endpoints";
const AUTH_URL: &str = "https://webapp-prod.cloud.remarkable.engineering/token/json/2";

#[derive(Error, Debug)]
pub enum RemarkableError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Auth error: {0}")]
    Auth(String),
    
    #[error("Sync error: {0}")]
    Sync(String),
    
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Service endpoints from discovery
#[derive(Debug, Clone, Deserialize)]
pub struct Endpoints {
    pub notifications: String,
    pub webapp: String,
    pub mqttbroker: String,
}

/// Sync root state
#[derive(Debug, Clone, Deserialize)]
pub struct SyncRoot {
    pub hash: String,
    pub generation: u64,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
}

/// Document metadata from sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemarkableDocument {
    pub id: String,
    pub hash: String,
    #[serde(rename = "type")]
    pub doc_type: String,
    #[serde(rename = "visibleName")]
    pub visible_name: String,
    pub parent: String,
    #[serde(rename = "modifiedClient")]
    pub modified_client: Option<DateTime<Utc>>,
    pub version: u32,
    #[serde(rename = "currentPage")]
    pub current_page: u32,
    pub bookmarked: bool,
}

/// Authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub device_token: String,
    pub user_token: Option<String>,
    pub tectonic_region: Option<String>,
}

impl AuthTokens {
    pub fn new(device_token: String) -> Self {
        Self {
            device_token,
            user_token: None,
            tectonic_region: None,
        }
    }
}

/// reMarkable sync client
pub struct RemarkableClient {
    client: Client,
    tokens: AuthTokens,
    endpoints: Option<Endpoints>,
    tectonic_url: Option<String>,
}

impl RemarkableClient {
    /// Create a new client with device token
    pub fn new(tokens: AuthTokens) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            tokens,
            endpoints: None,
            tectonic_url: None,
        }
    }
    
    /// Discover service endpoints
    pub async fn discover(&mut self) -> Result<(), RemarkableError> {
        let response = self.client
            .get(DISCOVERY_URL)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(RemarkableError::Sync("Discovery failed".to_string()));
        }
        
        self.endpoints = Some(response.json().await?);
        Ok(())
    }
    
    /// Refresh user token
    pub async fn refresh_token(&mut self) -> Result<(), RemarkableError> {
        let url = format!("{}/user/new", AUTH_URL);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.tokens.device_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(RemarkableError::Auth("Token refresh failed".to_string()));
        }
        
        let user_token = response.text().await?;
        
        // Extract tectonic region from JWT
        if let Some(region) = extract_tectonic_region(&user_token) {
            self.tectonic_url = Some(format!("https://{}.tectonic.remarkable.com", region));
            self.tokens.tectonic_region = Some(region);
        }
        
        self.tokens.user_token = Some(user_token);
        Ok(())
    }
    
    /// Ensure we have a valid user token
    pub async fn ensure_authenticated(&mut self) -> Result<(), RemarkableError> {
        if self.tokens.user_token.is_none() {
            self.refresh_token().await?;
        }
        Ok(())
    }
    
    /// Get sync root
    pub async fn get_root(&mut self) -> Result<SyncRoot, RemarkableError> {
        self.ensure_authenticated().await?;
        
        let tectonic = self.tectonic_url.as_ref()
            .ok_or_else(|| RemarkableError::Auth("No tectonic URL".to_string()))?;
        
        let user_token = self.tokens.user_token.as_ref()
            .ok_or_else(|| RemarkableError::Auth("No user token".to_string()))?;
        
        let url = format!("{}/sync/v3/root", tectonic);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", user_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(RemarkableError::Sync("Failed to get root".to_string()));
        }
        
        Ok(response.json().await?)
    }
    
    /// Download a file by hash
    pub async fn download_file(&mut self, hash: &str, filename: &str) -> Result<Vec<u8>, RemarkableError> {
        self.ensure_authenticated().await?;
        
        let tectonic = self.tectonic_url.as_ref()
            .ok_or_else(|| RemarkableError::Auth("No tectonic URL".to_string()))?;
        
        let user_token = self.tokens.user_token.as_ref()
            .ok_or_else(|| RemarkableError::Auth("No user token".to_string()))?;
        
        let url = format!("{}/sync/v3/files/{}", tectonic, hash);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", user_token))
            .header("rm-filename", filename)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(RemarkableError::NotFound(hash.to_string()));
        }
        
        Ok(response.bytes().await?.to_vec())
    }
    
    /// Parse root blob into document list
    pub fn parse_root_blob(&self, data: &[u8]) -> Result<Vec<RemarkableDocument>, RemarkableError> {
        let content = String::from_utf8_lossy(data);
        let mut documents = Vec::new();
        
        for line in content.lines().skip(1) { // Skip version line
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 5 {
                // hash:type:uuid:version:size
                documents.push(RemarkableDocument {
                    id: parts[2].to_string(),
                    hash: parts[0].to_string(),
                    doc_type: parts[1].to_string(),
                    visible_name: String::new(),
                    parent: String::new(),
                    modified_client: None,
                    version: parts[3].parse().unwrap_or(0),
                    current_page: 0,
                    bookmarked: false,
                });
            }
        }
        
        Ok(documents)
    }
    
    /// List all documents
    pub async fn list_documents(&mut self) -> Result<Vec<RemarkableDocument>, RemarkableError> {
        let root = self.get_root().await?;
        let root_blob = self.download_file(&root.hash, "root").await?;
        self.parse_root_blob(&root_blob)
    }
    
    /// Download document pages (.rm files)
    pub async fn download_document_pages(
        &mut self,
        doc: &RemarkableDocument,
    ) -> Result<Vec<Vec<u8>>, RemarkableError> {
        let doc_blob = self.download_file(&doc.hash, &doc.id).await?;
        let content = String::from_utf8_lossy(&doc_blob);
        
        let mut pages = Vec::new();
        
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 && parts[1].ends_with(".rm") {
                let page_data = self.download_file(parts[0], parts[1]).await?;
                pages.push(page_data);
            }
        }
        
        Ok(pages)
    }
}

/// Extract tectonic region from JWT
fn extract_tectonic_region(token: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    
    // Decode payload (base64url)
    let payload = parts[1];
    let decoded = base64_decode_url(payload)?;
    
    // Parse JSON
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    
    // Extract tectonic region from scopes or directly
    json.get("tectonic")
        .or_else(|| json.get("region"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn base64_decode_url(input: &str) -> Option<Vec<u8>> {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD.decode(input).ok()
}

/// Local document cache
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentCache {
    pub documents: HashMap<String, RemarkableDocument>,
    pub last_sync: Option<DateTime<Utc>>,
}

impl DocumentCache {
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }
    
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
