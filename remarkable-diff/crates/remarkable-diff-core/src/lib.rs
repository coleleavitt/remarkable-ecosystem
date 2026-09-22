//! CRDT Diff & Merge Engine for reMarkable documents
//!
//! Provides visual diffing and conflict-free merging of reMarkable documents
//! by analyzing CRDT operations and stroke data.

pub mod diff;
pub mod merge;
pub mod timeline;
pub mod types;

pub use diff::*;
pub use merge::*;
pub use timeline::*;
pub use types::*;
