//! reMarkable .rm file format parser (Version 6)
//!
//! Parses stroke data from reMarkable tablet files using CRDT structures.

mod types;
mod parser;
mod svg;
mod markdown;

pub use types::*;
pub use parser::*;
pub use svg::*;
pub use markdown::*;
