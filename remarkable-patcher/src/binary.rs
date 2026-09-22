//! Binary file handling and manipulation

use crate::error::PatcherError;
use crate::patch::{Patch, PatchResult, PatchStatus};
use crate::patterns::PatternScanner;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Information about a xochitl binary
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    pub path: PathBuf,
    pub size: usize,
    pub sha256: String,
    pub architecture: String,
    pub build_id: Option<String>,
    pub version: Option<String>,
}

/// Represents a loaded xochitl binary
pub struct XochitlBinary {
    data: Vec<u8>,
    info: BinaryInfo,
    original_path: PathBuf,
}

impl XochitlBinary {
    /// Open and parse a xochitl binary
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PatcherError> {
        let path = path.as_ref();
        let data = fs::read(path)?;
        let sha256 = Self::compute_sha256(&data);

        // Parse ELF header to get architecture info
        let (architecture, build_id) = Self::parse_elf_info(&data)?;

        // Try to detect version from strings
        let version = Self::detect_version(&data);

        let info = BinaryInfo {
            path: path.to_path_buf(),
            size: data.len(),
            sha256,
            architecture,
            build_id,
            version,
        };

        Ok(Self {
            data,
            info,
            original_path: path.to_path_buf(),
        })
    }

    /// Get binary information
    pub fn info(&self) -> &BinaryInfo {
        &self.info
    }

    /// Get raw data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Compute current SHA256
    pub fn checksum(&self) -> String {
        Self::compute_sha256(&self.data)
    }

    fn compute_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    fn parse_elf_info(data: &[u8]) -> Result<(String, Option<String>), PatcherError> {
        use object::{Object, ObjectSection};

        let file = object::File::parse(data)?;

        let arch = match file.architecture() {
            object::Architecture::Arm => "ARM32 (EABI5)".to_string(),
            object::Architecture::Aarch64 => "ARM64".to_string(),
            other => format!("{:?}", other),
        };

        // Try to find build ID
        let build_id = file
            .section_by_name(".note.gnu.build-id")
            .and_then(|section| section.data().ok())
            .and_then(|data| {
                // Build ID note format: name_size (4) + desc_size (4) + type (4) + name + desc
                if data.len() < 16 {
                    return None;
                }
                let name_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                let desc_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
                let desc_offset = 12 + ((name_size + 3) & !3); // Align to 4 bytes
                if data.len() < desc_offset + desc_size {
                    return None;
                }
                Some(hex::encode(&data[desc_offset..desc_offset + desc_size]))
            });

        Ok((arch, build_id))
    }

    fn detect_version(data: &[u8]) -> Option<String> {
        // Look for version pattern like "3.27.1.0" or similar
        let pattern = regex::bytes::Regex::new(r"(?:version[:\s=]+)?(\d+\.\d+\.\d+\.\d+)").ok()?;

        // Search near known version strings
        let version_markers = [
            b"sw-description" as &[u8],
            b"remarkable-production-image",
            b"REMARKABLE_VERSION",
        ];

        for marker in version_markers {
            if let Some(pos) = Self::find_bytes(data, marker) {
                // Search in a window around the marker
                let start = pos.saturating_sub(100);
                let end = (pos + 200).min(data.len());
                let window = &data[start..end];

                if let Some(caps) = pattern.captures(window) {
                    if let Some(version) = caps.get(1) {
                        return Some(String::from_utf8_lossy(version.as_bytes()).to_string());
                    }
                }
            }
        }

        // Fallback: search entire binary for version patterns
        for caps in pattern.captures_iter(data) {
            if let Some(version) = caps.get(1) {
                let v = String::from_utf8_lossy(version.as_bytes()).to_string();
                // Validate it looks like a reMarkable version
                if v.starts_with("3.") || v.starts_with("2.") {
                    return Some(v);
                }
            }
        }

        None
    }

    fn find_bytes(data: &[u8], pattern: &[u8]) -> Option<usize> {
        data.windows(pattern.len()).position(|w| w == pattern)
    }

    /// Check if this is a valid xochitl binary
    pub fn is_valid_xochitl(&self) -> bool {
        // Check for ELF magic
        if self.data.len() < 4 || &self.data[0..4] != b"\x7fELF" {
            return false;
        }

        // Check for ARM architecture
        if !self.info.architecture.contains("ARM") {
            return false;
        }

        // Check for xochitl-specific strings
        let markers = [b"xochitl" as &[u8], b"remarkable", b"xofm"];
        markers.iter().any(|m| Self::find_bytes(&self.data, m).is_some())
    }

    /// Create a backup of the original binary
    pub fn create_backup(&self) -> Result<PathBuf, PatcherError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!(
            "{}.{}.backup",
            self.original_path.file_name().unwrap().to_string_lossy(),
            timestamp
        );

        let backup_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("remarkable-patcher")
            .join("backups");

        fs::create_dir_all(&backup_dir)?;

        let backup_path = backup_dir.join(&backup_name);

        // Write backup with metadata
        let metadata = serde_json::json!({
            "original_path": self.original_path,
            "sha256": self.info.sha256,
            "timestamp": timestamp.to_string(),
            "version": self.info.version,
        });

        fs::write(&backup_path, &self.data)?;
        fs::write(
            backup_path.with_extension("backup.json"),
            serde_json::to_string_pretty(&metadata).unwrap(),
        )?;

        Ok(backup_path)
    }

    /// Restore from a backup
    pub fn restore_backup(backup_path: impl AsRef<Path>) -> Result<(), PatcherError> {
        let backup_path = backup_path.as_ref();
        let metadata_path = backup_path.with_extension("backup.json");

        if !backup_path.exists() {
            return Err(PatcherError::BackupNotFound {
                path: backup_path.to_path_buf(),
            });
        }

        // Read metadata
        let metadata: serde_json::Value = serde_json::from_str(&fs::read_to_string(&metadata_path)?)?;
        let original_path: PathBuf = serde_json::from_value(metadata["original_path"].clone())?;

        // Restore file
        fs::copy(backup_path, &original_path)?;

        Ok(())
    }

    /// Check if a patch is applied
    pub fn check_patch(&self, patch: &dyn Patch) -> Result<PatchStatus, PatcherError> {
        let scanner = PatternScanner::new(self);

        // Check if the "after" pattern exists (patch applied)
        if scanner.find_raw(patch.after_bytes()).is_some() {
            return Ok(PatchStatus::Applied);
        }

        // Check if the "before" pattern exists (patch not applied)
        if scanner.find_raw(patch.before_bytes()).is_some() {
            return Ok(PatchStatus::NotApplied);
        }

        // Neither pattern found
        Ok(PatchStatus::Unknown)
    }

    /// Apply a patch
    pub fn apply_patch(&mut self, patch: &dyn Patch) -> Result<PatchResult, PatcherError> {
        let scanner = PatternScanner::new(self);

        // Check if already applied
        if scanner.find_raw(patch.after_bytes()).is_some() {
            return Ok(PatchResult::AlreadyApplied);
        }

        // Find the pattern to patch
        let offset = match scanner.find_raw(patch.before_bytes()) {
            Some(off) => off,
            None => return Ok(PatchResult::PatternNotFound),
        };

        // Apply the patch
        let after = patch.after_bytes();
        if offset + after.len() > self.data.len() {
            return Err(PatcherError::OffsetOutOfBounds {
                offset,
                size: self.data.len(),
            });
        }

        self.data[offset..offset + after.len()].copy_from_slice(after);

        Ok(PatchResult::Applied { offset })
    }

    /// Write the modified binary to a path
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), PatcherError> {
        fs::write(path, &self.data)?;
        Ok(())
    }

    /// Extract embedded resources (QML, images, etc.)
    pub fn extract_resources(&self, output_dir: impl AsRef<Path>) -> Result<usize, PatcherError> {
        use object::{Object, ObjectSection};

        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir)?;

        let file = object::File::parse(&*self.data)?;
        let mut count = 0;

        // Look for .rodata section which contains Qt resources
        if let Some(rodata) = file.section_by_name(".rodata") {
            if let Ok(data) = rodata.data() {
                // Find QRC (Qt Resource) magic
                count += self.extract_qrc_resources(data, output_dir)?;
            }
        }

        // Also extract any PNG images found
        count += self.extract_embedded_images(output_dir)?;

        Ok(count)
    }

    fn extract_qrc_resources(&self, data: &[u8], output_dir: &Path) -> Result<usize, PatcherError> {
        let mut count = 0;

        // Qt Resource Collection format markers
        // Look for qrc path strings like ":/qt/qml/"
        let qrc_marker = b":/qt/qml/";
        let mut pos = 0;

        while let Some(offset) = Self::find_bytes(&data[pos..], qrc_marker) {
            let abs_offset = pos + offset;

            // Try to extract the full path
            let end = data[abs_offset..]
                .iter()
                .position(|&b| b == 0 || !b.is_ascii_graphic() && b != b'/')
                .unwrap_or(100);

            if end > 0 && end < 256 {
                let path_bytes = &data[abs_offset..abs_offset + end];
                if let Ok(path_str) = std::str::from_utf8(path_bytes) {
                    // Create a marker file for QML paths found
                    let marker_path = output_dir.join("qml_paths.txt");
                    let mut file = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&marker_path)?;
                    writeln!(file, "{}", path_str)?;
                    count += 1;
                }
            }

            pos = abs_offset + 1;
        }

        Ok(count)
    }

    fn extract_embedded_images(&self, output_dir: &Path) -> Result<usize, PatcherError> {
        let mut count = 0;
        let images_dir = output_dir.join("images");
        fs::create_dir_all(&images_dir)?;

        // PNG signature
        let png_magic = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let mut pos = 0;

        while pos < self.data.len() - 8 {
            if self.data[pos..pos + 8] == png_magic {
                // Try to find PNG end marker (IEND chunk)
                if let Some(end) = Self::find_png_end(&self.data[pos..]) {
                    let png_data = &self.data[pos..pos + end];
                    let filename = format!("image_{:08x}.png", pos);
                    fs::write(images_dir.join(&filename), png_data)?;
                    count += 1;
                    pos += end;
                    continue;
                }
            }
            pos += 1;
        }

        Ok(count)
    }

    fn find_png_end(data: &[u8]) -> Option<usize> {
        // PNG IEND chunk: length (4) + "IEND" (4) + CRC (4) = 12 bytes
        let iend_marker = b"IEND";
        for (i, window) in data.windows(4).enumerate() {
            if window == iend_marker && i >= 4 {
                // Check if this looks like a valid IEND chunk
                // The length should be 0 (IEND has no data)
                let len_bytes = &data[i - 4..i];
                let len = u32::from_be_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]);
                if len == 0 {
                    return Some(i + 8); // Include IEND + CRC
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"test data";
        let hash = XochitlBinary::compute_sha256(data);
        assert_eq!(hash.len(), 64); // SHA256 hex is 64 chars
    }
}
