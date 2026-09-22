//! reMarkable Notes Sync - Bidirectional sync with Obsidian and Notion
//!
//! This library provides tools for synchronizing reMarkable tablet documents
//! with Obsidian vaults and Notion workspaces.

pub mod rm_format;
pub mod sync;
pub mod obsidian;
pub mod notion;
pub mod conflict;

// Re-exports for convenience
pub use rm_format::{RmPage, Stroke, Point, PenType, PenColor, parse_rm_file};
pub use sync::{RemarkableClient, AuthTokens, SyncCoordinator};
pub use obsidian::{ObsidianVault, VaultConfig};
pub use notion::{NotionClient, NotionConfig, NotionSync};
pub use conflict::{ConflictResolver, Strategy, Resolution};
