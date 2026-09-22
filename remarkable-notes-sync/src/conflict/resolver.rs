//! Timestamp-based conflict resolution

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Represents the state of a document at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    /// Document identifier
    pub id: String,
    /// Source system (remarkable, obsidian, notion)
    pub source: String,
    /// Last modification time
    pub modified: DateTime<Utc>,
    /// Content hash for change detection
    pub content_hash: String,
    /// Version number (for CRDT resolution)
    pub version: u64,
}

impl DocumentState {
    pub fn new(id: &str, source: &str, modified: DateTime<Utc>, content_hash: &str) -> Self {
        Self {
            id: id.to_string(),
            source: source.to_string(),
            modified,
            content_hash: content_hash.to_string(),
            version: 1,
        }
    }
}

/// Result of conflict resolution
#[derive(Debug, Clone)]
pub enum Resolution {
    /// Use the local version
    KeepLocal,
    /// Use the remote version
    KeepRemote,
    /// Conflict requires manual merge
    Conflict {
        local: DocumentState,
        remote: DocumentState,
    },
    /// No changes needed
    NoChange,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    /// Most recent modification wins
    LastWriteWins,
    /// reMarkable version always wins
    RemarkableFirst,
    /// Local (Obsidian/Notion) version always wins
    LocalFirst,
    /// Prompt user on conflict
    Manual,
}

impl Default for Strategy {
    fn default() -> Self {
        Self::LastWriteWins
    }
}

/// Conflict resolver
#[derive(Debug, Clone)]
pub struct ConflictResolver {
    strategy: Strategy,
    /// Grace period in seconds for near-simultaneous edits
    grace_period_secs: i64,
}

impl ConflictResolver {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            grace_period_secs: 5,
        }
    }
    
    pub fn with_grace_period(mut self, secs: i64) -> Self {
        self.grace_period_secs = secs;
        self
    }
    
    /// Resolve a conflict between local and remote states
    pub fn resolve(&self, local: &DocumentState, remote: &DocumentState) -> Resolution {
        // Check if either is unchanged
        if local.content_hash == remote.content_hash {
            return Resolution::NoChange;
        }
        
        // Apply strategy
        match self.strategy {
            Strategy::LastWriteWins => self.last_write_wins(local, remote),
            Strategy::RemarkableFirst => {
                if local.source == "remarkable" {
                    Resolution::KeepLocal
                } else {
                    Resolution::KeepRemote
                }
            }
            Strategy::LocalFirst => Resolution::KeepLocal,
            Strategy::Manual => Resolution::Conflict {
                local: local.clone(),
                remote: remote.clone(),
            },
        }
    }
    
    fn last_write_wins(&self, local: &DocumentState, remote: &DocumentState) -> Resolution {
        let diff = (local.modified - remote.modified).num_seconds();
        
        if diff.abs() <= self.grace_period_secs {
            // Within grace period - potential conflict
            // Fall back to version comparison
            match local.version.cmp(&remote.version) {
                Ordering::Greater => Resolution::KeepLocal,
                Ordering::Less => Resolution::KeepRemote,
                Ordering::Equal => Resolution::Conflict {
                    local: local.clone(),
                    remote: remote.clone(),
                },
            }
        } else if diff > 0 {
            // Local is newer
            Resolution::KeepLocal
        } else {
            // Remote is newer
            Resolution::KeepRemote
        }
    }
}

/// Sync state tracking for conflict detection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncState {
    /// Last known states per document
    pub documents: std::collections::HashMap<String, DocumentState>,
    /// Last full sync time
    pub last_sync: Option<DateTime<Utc>>,
}

impl SyncState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update state for a document
    pub fn update(&mut self, state: DocumentState) {
        self.documents.insert(state.id.clone(), state);
        self.last_sync = Some(Utc::now());
    }
    
    /// Get state for a document
    pub fn get(&self, id: &str) -> Option<&DocumentState> {
        self.documents.get(id)
    }
    
    /// Check if a document has changed since last sync
    pub fn has_changed(&self, id: &str, current_hash: &str) -> bool {
        self.documents
            .get(id)
            .map(|s| s.content_hash != current_hash)
            .unwrap_or(true) // New document = changed
    }
    
    /// Save to file
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }
    
    /// Load from file
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Calculate content hash
pub fn hash_content(content: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_last_write_wins() {
        let resolver = ConflictResolver::new(Strategy::LastWriteWins);
        
        let local = DocumentState {
            id: "test".to_string(),
            source: "obsidian".to_string(),
            modified: Utc::now(),
            content_hash: "abc".to_string(),
            version: 1,
        };
        
        let remote = DocumentState {
            id: "test".to_string(),
            source: "remarkable".to_string(),
            modified: Utc::now() - chrono::Duration::hours(1),
            content_hash: "def".to_string(),
            version: 1,
        };
        
        match resolver.resolve(&local, &remote) {
            Resolution::KeepLocal => {}
            _ => panic!("Expected KeepLocal"),
        }
    }
    
    #[test]
    fn test_no_change() {
        let resolver = ConflictResolver::new(Strategy::LastWriteWins);
        
        let local = DocumentState {
            id: "test".to_string(),
            source: "obsidian".to_string(),
            modified: Utc::now(),
            content_hash: "same".to_string(),
            version: 1,
        };
        
        let remote = DocumentState {
            id: "test".to_string(),
            source: "remarkable".to_string(),
            modified: Utc::now() - chrono::Duration::hours(1),
            content_hash: "same".to_string(),
            version: 1,
        };
        
        match resolver.resolve(&local, &remote) {
            Resolution::NoChange => {}
            _ => panic!("Expected NoChange"),
        }
    }
}
