//! remarkable-collab: Real-time collaboration for reMarkable documents.
//!
//! This crate provides:
//! - CRDT-based conflict resolution using Lamport timestamps
//! - WebSocket server for real-time sync
//! - Presence tracking (cursors, selections)
//! - Per-user undo/redo
//! - Protocol for join/broadcast/merge operations
//! - Integration with remarkable-server (local sync)
//!
//! # Example
//!
//! ```no_run
//! use remarkable_collab::ws::{CollabServer, ServerConfig};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let server = Arc::new(CollabServer::new(ServerConfig::default()));
//!     // Start accepting WebSocket connections...
//! }
//! ```

pub mod crdt;
pub mod integration;
pub mod presence;
pub mod protocol;
pub mod ws;

// Re-exports for convenience
pub use crdt::{CrdtDocument, CrdtOp, LamportClock, LamportId};
pub use integration::{RemarkableServerClient, ServerIntegrationConfig};
pub use presence::{CursorPosition, PresenceTracker, UserPresence};
pub use protocol::{ClientMessage, ServerMessage};
pub use ws::{CollabServer, ServerConfig};
