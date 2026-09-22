//! Application state

use remarkable_diff_core::{
    DocumentVersion, DocumentDiff, MergeResult, Timeline,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Application state shared across handlers
pub struct AppState {
    /// Loaded documents by ID
    pub documents: HashMap<Uuid, DocumentInfo>,
    
    /// Computed diffs by ID
    pub diffs: HashMap<Uuid, DocumentDiff>,
    
    /// Merge results by ID
    pub merges: HashMap<Uuid, MergeResult>,
    
    /// Cached timelines by document ID
    pub timelines: HashMap<Uuid, Timeline>,
}

/// Document metadata with versions
pub struct DocumentInfo {
    pub id: Uuid,
    pub name: String,
    pub versions: Vec<DocumentVersion>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            diffs: HashMap::new(),
            merges: HashMap::new(),
            timelines: HashMap::new(),
        }
    }
    
    pub fn add_document(&mut self, name: String, version: DocumentVersion) -> Uuid {
        let doc_id = version.document_id;
        
        if let Some(info) = self.documents.get_mut(&doc_id) {
            info.versions.push(version);
        } else {
            self.documents.insert(doc_id, DocumentInfo {
                id: doc_id,
                name,
                versions: vec![version],
            });
        }
        
        doc_id
    }
    
    pub fn get_document(&self, id: &Uuid) -> Option<&DocumentInfo> {
        self.documents.get(id)
    }
    
    pub fn get_version(&self, doc_id: &Uuid, version_id: &Uuid) -> Option<&DocumentVersion> {
        self.documents.get(doc_id)
            .and_then(|info| info.versions.iter().find(|v| v.id == *version_id))
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
