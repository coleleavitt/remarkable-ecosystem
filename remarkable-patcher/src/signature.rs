#![allow(dead_code)]
//! Signature database for known firmware versions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a firmware version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FirmwareStatus {
    Official,
    Beta,
    Patched,
    Unknown,
}

impl std::fmt::Display for FirmwareStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirmwareStatus::Official => write!(f, "Official"),
            FirmwareStatus::Beta => write!(f, "Beta"),
            FirmwareStatus::Patched => write!(f, "Patched"),
            FirmwareStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Entry in the signature database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureEntry {
    pub sha256: String,
    pub version: String,
    pub device: String,
    pub status: FirmwareStatus,
    pub build_id: Option<String>,
    pub notes: Option<String>,
}

/// Database of known firmware signatures
#[derive(Debug, Clone, Default)]
pub struct SignatureDatabase {
    entries: HashMap<String, SignatureEntry>,
}

impl SignatureDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the built-in signature database
    pub fn load_builtin() -> Self {
        let mut db = Self::new();

        // Add known firmware signatures
        // These would normally come from the IDA database analysis
        db.add(SignatureEntry {
            sha256: "32a37ab26f85ba8583aa03a6124764781ab31ae9".to_string(),
            version: "3.29.0.148".to_string(),
            device: "rm2".to_string(),
            status: FirmwareStatus::Official,
            build_id: Some("32a37ab26f85ba8583aa03a6124764781ab31ae9".to_string()),
            notes: Some("Latest stable for RM2".to_string()),
        });

        db.add(SignatureEntry {
            sha256: "5c36b5bc41d9ca8e77720da64d3791737040fe1e".to_string(),
            version: "3.27.1.0".to_string(),
            device: "rm2".to_string(),
            status: FirmwareStatus::Official,
            build_id: Some("5c36b5bc41d9ca8e77720da64d3791737040fe1e".to_string()),
            notes: None,
        });

        db.add(SignatureEntry {
            sha256: "placeholder_3.28.0.172_rm2".to_string(),
            version: "3.28.0.172".to_string(),
            device: "rm2".to_string(),
            status: FirmwareStatus::Official,
            build_id: None,
            notes: None,
        });

        db.add(SignatureEntry {
            sha256: "placeholder_3.27.3.0_rm2".to_string(),
            version: "3.27.3.0".to_string(),
            device: "rm2".to_string(),
            status: FirmwareStatus::Official,
            build_id: None,
            notes: None,
        });

        // Add more known versions...
        let versions = [
            ("3.26.0.68", "rm2"),
            ("3.26.0.68", "rm1"),
            ("3.26.0.68", "ferrari"),
            ("3.25.1.1", "rm2"),
            ("3.25.1.1", "rm1"),
            ("3.24.0.149", "chiappa"),
            ("3.23.0.64", "rm2"),
            ("3.12.4.4", "rm2"),
            ("3.11.2.5", "rm2"),
            ("3.10.2.2063", "rm2"),
        ];

        for (version, device) in versions {
            db.add(SignatureEntry {
                sha256: format!("placeholder_{}_{}", version, device),
                version: version.to_string(),
                device: device.to_string(),
                status: FirmwareStatus::Official,
                build_id: None,
                notes: None,
            });
        }

        db
    }

    /// Add an entry to the database
    pub fn add(&mut self, entry: SignatureEntry) {
        self.entries.insert(entry.sha256.clone(), entry);
    }

    /// Look up a signature
    pub fn lookup(&self, sha256: &str) -> Option<&SignatureEntry> {
        self.entries.get(sha256)
    }

    /// Look up by version
    pub fn lookup_version(&self, version: &str) -> Vec<&SignatureEntry> {
        self.entries
            .values()
            .filter(|e| e.version == version)
            .collect()
    }

    /// Get all entries
    pub fn all(&self) -> impl Iterator<Item = &SignatureEntry> {
        self.entries.values()
    }

    /// Load from a JSON file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let entries: Vec<SignatureEntry> = serde_json::from_str(&content)?;
        let mut db = Self::new();
        for entry in entries {
            db.add(entry);
        }
        Ok(db)
    }

    /// Save to a JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let entries: Vec<_> = self.entries.values().collect();
        let content = serde_json::to_string_pretty(&entries)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Patch offset database for known versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOffsets {
    pub version: String,
    pub device: String,
    pub offsets: HashMap<String, u64>,
}

/// Database of known patch offsets
#[derive(Debug, Clone, Default)]
pub struct OffsetDatabase {
    entries: HashMap<String, PatchOffsets>,
}

impl OffsetDatabase {
    /// Create empty database
    pub fn new() -> Self {
        Self::default()
    }

    /// Load built-in offsets from IDA analysis
    pub fn load_builtin() -> Self {
        let mut db = Self::new();

        // These offsets would come from IDA database analysis
        // Format: (version, device, patch_name, offset)

        // Example offsets for 3.27.1.0
        let mut offsets_3_27 = HashMap::new();
        offsets_3_27.insert("insecure_settings_check".to_string(), 0x00123456);
        offsets_3_27.insert("telemetry_enable".to_string(), 0x00234567);
        offsets_3_27.insert("subscription_check".to_string(), 0x00345678);

        db.add(PatchOffsets {
            version: "3.27.1.0".to_string(),
            device: "rm2".to_string(),
            offsets: offsets_3_27,
        });

        db
    }

    /// Add offsets for a version
    pub fn add(&mut self, offsets: PatchOffsets) {
        let key = format!("{}_{}", offsets.version, offsets.device);
        self.entries.insert(key, offsets);
    }

    /// Get offsets for a version
    pub fn get(&self, version: &str, device: &str) -> Option<&PatchOffsets> {
        let key = format!("{}_{}", version, device);
        self.entries.get(&key)
    }

    /// Get a specific offset
    pub fn get_offset(&self, version: &str, device: &str, patch: &str) -> Option<u64> {
        self.get(version, device)?.offsets.get(patch).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_database() {
        let db = SignatureDatabase::load_builtin();
        assert!(db.all().count() > 0);
    }

    #[test]
    fn test_offset_database() {
        let db = OffsetDatabase::load_builtin();
        assert!(db.get("3.27.1.0", "rm2").is_some());
    }
}
