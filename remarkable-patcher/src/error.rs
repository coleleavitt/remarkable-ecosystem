#![allow(dead_code)]
//! Error types for the patcher

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatcherError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Binary at {path} is not a valid xochitl executable")]
    InvalidBinary { path: PathBuf },

    #[error("Unsupported architecture: {arch}")]
    UnsupportedArch { arch: String },

    #[error("Pattern not found for patch: {patch_name}")]
    PatternNotFound { patch_name: String },

    #[error("Patch failed: {reason}")]
    PatchFailed { reason: String },

    #[error("Backup not found: {path}")]
    BackupNotFound { path: PathBuf },

    #[error("Version not recognized: {sha256}")]
    UnknownVersion { sha256: String },

    #[error("Object parse error: {0}")]
    ObjectParse(#[from] object::read::Error),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Offset out of bounds: {offset} >= {size}")]
    OffsetOutOfBounds { offset: usize, size: usize },
}
