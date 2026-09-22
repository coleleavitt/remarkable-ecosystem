//! Timeline visualization of CRDT operations
//!
//! Provides a chronological view of all CRDT operations
//! for understanding document evolution.

use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timeline of CRDT operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Ordered list of events
    pub events: Vec<TimelineEvent>,
    /// Authors present in this timeline
    pub authors: Vec<AuthorInfo>,
    /// Time range
    pub time_range: TimeRange,
    /// Statistics by operation type
    pub stats: TimelineStats,
}

/// A single timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// Event index (sequential)
    pub index: usize,
    /// CRDT operation
    pub operation: CrdtOperationLog,
    /// Visual category for styling
    pub category: String,
    /// Is this an addition or deletion
    pub change_type: ChangeType,
    /// Affected items summary
    pub summary: String,
}

/// Change type for visual styling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Addition,
    Deletion,
    Modification,
    Move,
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// Author ID
    pub id: u64,
    /// Color for visualization
    pub color: String,
    /// Operation count
    pub operation_count: usize,
    /// First operation timestamp
    pub first_seen: CrdtTimestamp,
    /// Last operation timestamp
    pub last_seen: CrdtTimestamp,
}

/// Time range
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: CrdtTimestamp,
    pub end: CrdtTimestamp,
}

/// Timeline statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimelineStats {
    pub total_operations: usize,
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub by_category: HashMap<String, usize>,
    pub by_author: HashMap<u64, usize>,
}

/// Build timeline from document operations
pub fn build_timeline(operations: &[CrdtOperationLog]) -> Timeline {
    let mut events = Vec::with_capacity(operations.len());
    let mut author_stats: HashMap<u64, (CrdtTimestamp, CrdtTimestamp, usize)> = HashMap::new();
    let mut stats = TimelineStats::default();
    
    // Sort operations by timestamp
    let mut sorted_ops: Vec<_> = operations.iter().collect();
    sorted_ops.sort_by_key(|op| op.timestamp);
    
    for (index, op) in sorted_ops.iter().enumerate() {
        let change_type = if op.operation.is_addition() {
            stats.additions += 1;
            ChangeType::Addition
        } else if op.operation.is_deletion() {
            stats.deletions += 1;
            ChangeType::Deletion
        } else {
            stats.modifications += 1;
            ChangeType::Modification
        };
        
        let category = op.operation.category().to_string();
        *stats.by_category.entry(category.clone()).or_insert(0) += 1;
        *stats.by_author.entry(op.author_id).or_insert(0) += 1;
        
        // Track author stats
        author_stats
            .entry(op.author_id)
            .and_modify(|(first, last, count)| {
                if op.timestamp < *first { *first = op.timestamp; }
                if op.timestamp > *last { *last = op.timestamp; }
                *count += 1;
            })
            .or_insert((op.timestamp, op.timestamp, 1));
        
        events.push(TimelineEvent {
            index,
            operation: (*op).clone(),
            category,
            change_type,
            summary: summarize_operation(op),
        });
    }
    
    stats.total_operations = events.len();
    
    // Build author list with colors
    let colors = ["#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6", "#ec4899"];
    let authors: Vec<AuthorInfo> = author_stats.into_iter()
        .enumerate()
        .map(|(i, (id, (first, last, count)))| {
            AuthorInfo {
                id,
                color: colors[i % colors.len()].to_string(),
                operation_count: count,
                first_seen: first,
                last_seen: last,
            }
        })
        .collect();
    
    let time_range = if let (Some(first), Some(last)) = (events.first(), events.last()) {
        TimeRange {
            start: first.operation.timestamp,
            end: last.operation.timestamp,
        }
    } else {
        TimeRange {
            start: CrdtTimestamp::new(0, 0),
            end: CrdtTimestamp::new(0, 0),
        }
    };
    
    Timeline {
        events,
        authors,
        time_range,
        stats,
    }
}

fn summarize_operation(op: &CrdtOperationLog) -> String {
    let items = if op.target_ids.is_empty() {
        "".to_string()
    } else if op.target_ids.len() == 1 {
        format!(" ({})", &op.target_ids[0])
    } else {
        format!(" ({} items)", op.target_ids.len())
    };
    
    match op.operation {
        CrdtOperationType::AddItem => format!("Add item{}", items),
        CrdtOperationType::AddItems => format!("Add {} items", op.target_ids.len()),
        CrdtOperationType::DeleteItems => format!("Delete{}", items),
        CrdtOperationType::DeleteAppendItems => format!("Delete and append{}", items),
        CrdtOperationType::InsertItemsAfter => format!("Insert after{}", items),
        CrdtOperationType::SwapItems => "Swap items".to_string(),
        CrdtOperationType::CreateGroup => format!("Create group{}", items),
        CrdtOperationType::DeleteGroup => format!("Delete group{}", items),
        CrdtOperationType::AddLayer => "Add layer".to_string(),
        CrdtOperationType::DeleteLayer => "Delete layer".to_string(),
        CrdtOperationType::MoveLayer => "Move layer".to_string(),
        CrdtOperationType::MergeLayerDown => "Merge layer down".to_string(),
        CrdtOperationType::SetLayerName => "Rename layer".to_string(),
        CrdtOperationType::SetLayerVisible => "Toggle layer visibility".to_string(),
        CrdtOperationType::TextInsert => format!("Insert text{}", items),
        CrdtOperationType::TextRemove => format!("Remove text{}", items),
    }
}

/// Filter timeline by author
pub fn filter_by_author(timeline: &Timeline, author_id: u64) -> Timeline {
    let events: Vec<_> = timeline.events.iter()
        .filter(|e| e.operation.author_id == author_id)
        .cloned()
        .collect();
    
    let mut stats = TimelineStats::default();
    for e in &events {
        stats.total_operations += 1;
        match e.change_type {
            ChangeType::Addition => stats.additions += 1,
            ChangeType::Deletion => stats.deletions += 1,
            ChangeType::Modification | ChangeType::Move => stats.modifications += 1,
        }
    }
    
    Timeline {
        events,
        authors: timeline.authors.iter()
            .filter(|a| a.id == author_id)
            .cloned()
            .collect(),
        time_range: timeline.time_range,
        stats,
    }
}

/// Filter timeline by time range
pub fn filter_by_range(timeline: &Timeline, start: CrdtTimestamp, end: CrdtTimestamp) -> Timeline {
    let events: Vec<_> = timeline.events.iter()
        .filter(|e| e.operation.timestamp >= start && e.operation.timestamp <= end)
        .cloned()
        .collect();
    
    Timeline {
        time_range: TimeRange { start, end },
        events,
        authors: timeline.authors.clone(),
        stats: timeline.stats.clone(),
    }
}

/// Filter timeline by category
pub fn filter_by_category(timeline: &Timeline, category: &str) -> Timeline {
    let events: Vec<_> = timeline.events.iter()
        .filter(|e| e.category == category)
        .cloned()
        .collect();
    
    Timeline {
        events,
        authors: timeline.authors.clone(),
        time_range: timeline.time_range,
        stats: timeline.stats.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_timeline() {
        let timeline = build_timeline(&[]);
        assert_eq!(timeline.events.len(), 0);
        assert_eq!(timeline.stats.total_operations, 0);
    }
}
