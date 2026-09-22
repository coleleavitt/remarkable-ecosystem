//! CRDT (Conflict-free Replicated Data Type) implementation for reMarkable documents.
//!
//! This module provides the core data structures for real-time collaboration:
//!
//! - `LamportId` / `LamportClock`: Ordering of operations across users/devices
//! - `CrdtOp`: All operations that can be performed on a document
//! - `CrdtDocument`: The complete document state with operation log
//! - `UndoStack`: Per-user undo/redo support

pub mod id;
pub mod ops;
pub mod document;

pub use id::{LamportId, LamportClock};
pub use ops::{
    CrdtOp, Group, Layer, PenType, Stroke, StrokeColor, StrokePoint,
    TextFormat, UndoableOp,
};
pub use document::{CrdtDocument, Page, PageItem, Tombstone, UndoStack};
