//! CRDT identifiers using Lamport timestamps.
//!
//! reMarkable uses `author:sequence` pairs for CRDT ordering.
//! This module implements the ID system for conflict-free operations.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// A Lamport timestamp ID for CRDT operations.
///
/// Format: `author:sequence` where:
/// - `author` is the device/user ID (integer)
/// - `sequence` is a monotonically increasing counter per author
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LamportId {
    /// The author (device/user) identifier.
    pub author: u32,
    /// The sequence number for this author.
    pub sequence: u64,
}

impl LamportId {
    /// Create a new Lamport ID.
    pub const fn new(author: u32, sequence: u64) -> Self {
        Self { author, sequence }
    }

    /// Create the initial ID for an author (sequence 0).
    pub const fn initial(author: u32) -> Self {
        Self::new(author, 0)
    }

    /// Increment the sequence and return the new ID.
    pub fn increment(&self) -> Self {
        Self {
            author: self.author,
            sequence: self.sequence.saturating_add(1),
        }
    }

    /// Parse from the reMarkable format "author:sequence".
    pub fn parse(s: &str) -> Option<Self> {
        let (author_str, seq_str) = s.split_once(':')?;
        let author = author_str.parse().ok()?;
        let sequence = seq_str.parse().ok()?;
        Some(Self { author, sequence })
    }
}

impl fmt::Display for LamportId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.author, self.sequence)
    }
}

impl Ord for LamportId {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare by sequence first (causality), then by author (tiebreaker)
        match self.sequence.cmp(&other.sequence) {
            Ordering::Equal => self.author.cmp(&other.author),
            ord => ord,
        }
    }
}

impl PartialOrd for LamportId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A clock that generates Lamport IDs for an author.
#[derive(Debug, Clone)]
pub struct LamportClock {
    author: u32,
    counter: u64,
}

impl LamportClock {
    /// Create a new clock for the given author.
    pub fn new(author: u32) -> Self {
        Self {
            author,
            counter: 0,
        }
    }

    /// Get the current author ID.
    pub fn author(&self) -> u32 {
        self.author
    }

    /// Get the current sequence number.
    pub fn sequence(&self) -> u64 {
        self.counter
    }

    /// Generate the next ID.
    pub fn tick(&mut self) -> LamportId {
        let id = LamportId::new(self.author, self.counter);
        self.counter = self.counter.saturating_add(1);
        id
    }

    /// Update the clock based on a received ID (ensure we stay ahead).
    pub fn update(&mut self, received: LamportId) {
        if received.sequence >= self.counter {
            self.counter = received.sequence.saturating_add(1);
        }
    }

    /// Peek at the next ID without incrementing.
    pub fn peek(&self) -> LamportId {
        LamportId::new(self.author, self.counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamport_id_ordering() {
        let a = LamportId::new(1, 5);
        let b = LamportId::new(2, 5);
        let c = LamportId::new(1, 6);

        // Same sequence, lower author wins
        assert!(a < b);
        // Higher sequence always wins
        assert!(a < c);
        assert!(b < c);
    }

    #[test]
    fn test_lamport_id_parse() {
        let id = LamportId::parse("2:15").unwrap();
        assert_eq!(id.author, 2);
        assert_eq!(id.sequence, 15);
        assert_eq!(id.to_string(), "2:15");
    }

    #[test]
    fn test_lamport_clock() {
        let mut clock = LamportClock::new(1);
        
        let id1 = clock.tick();
        assert_eq!(id1, LamportId::new(1, 0));
        
        let id2 = clock.tick();
        assert_eq!(id2, LamportId::new(1, 1));
        
        // Update from external event
        clock.update(LamportId::new(2, 100));
        let id3 = clock.tick();
        assert_eq!(id3, LamportId::new(1, 101));
    }
}
