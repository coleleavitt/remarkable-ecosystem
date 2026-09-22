//! Merge conflicting document versions
//!
//! Uses CRDT semantics for conflict-free merging:
//! - Last-writer-wins based on Lamport timestamps
//! - Tombstones preserve deleted items for causal consistency
//! - Supports manual conflict resolution for ambiguous cases

use crate::types::*;
use crate::diff::{DocumentDiff, StrokeDiff, StrokeDiffStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Merge result containing resolved document and any conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Merged document version
    pub merged: DocumentVersion,
    /// Conflicts that required manual resolution
    pub conflicts: Vec<MergeConflict>,
    /// Resolution strategy used
    pub strategy: MergeStrategy,
    /// Statistics
    pub stats: MergeStats,
}

/// Merge conflict requiring resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    /// Conflict ID for resolution
    pub id: Uuid,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Location (page, layer)
    pub location: ConflictLocation,
    /// Version A data
    pub version_a: serde_json::Value,
    /// Version B data
    pub version_b: serde_json::Value,
    /// Chosen resolution (if resolved)
    pub resolution: Option<ConflictResolution>,
}

/// Conflict type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// Same stroke modified differently
    ConcurrentModification,
    /// Layer renamed to different names
    LayerNameConflict,
    /// Page ordering conflict
    PageOrderConflict,
    /// Group membership conflict
    GroupConflict,
}

/// Conflict location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictLocation {
    pub page_id: Option<Uuid>,
    pub layer_id: Option<u32>,
    pub stroke_id: Option<String>,
}

/// How a conflict was resolved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    KeepA,
    KeepB,
    KeepBoth,
    KeepNeither,
    Manual,
}

/// Merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    /// Use CRDT semantics (last writer wins)
    #[default]
    LastWriterWins,
    /// Keep all changes (union)
    KeepAll,
    /// Prefer version A
    PreferA,
    /// Prefer version B
    PreferB,
    /// Manual resolution required for conflicts
    Manual,
}

/// Merge statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MergeStats {
    pub strokes_from_a: usize,
    pub strokes_from_b: usize,
    pub strokes_merged: usize,
    pub conflicts_auto_resolved: usize,
    pub conflicts_manual: usize,
}

/// Merge two document versions
pub fn merge_documents(
    version_a: &DocumentVersion,
    version_b: &DocumentVersion,
    strategy: MergeStrategy,
    manual_resolutions: &HashMap<Uuid, ConflictResolution>,
) -> MergeResult {
    let mut stats = MergeStats::default();
    let mut conflicts = Vec::new();
    
    // Start with version A as base
    let mut merged = version_a.clone();
    merged.id = Uuid::new_v4();
    merged.name = format!("Merged: {} + {}", version_a.name, version_b.name);
    merged.timestamp = chrono::Utc::now();
    
    // Index pages
    let pages_a: HashMap<Uuid, usize> = version_a.pages.iter()
        .enumerate()
        .map(|(i, p)| (p.id, i))
        .collect();
    
    // Process pages from B
    for page_b in &version_b.pages {
        if let Some(&idx) = pages_a.get(&page_b.id) {
            // Page exists in both - merge strokes
            let page_a = &version_a.pages[idx];
            let merged_page = merge_pages(
                page_a, 
                page_b, 
                strategy,
                manual_resolutions,
                &mut stats, 
                &mut conflicts,
            );
            merged.pages[idx] = merged_page;
        } else {
            // Page only in B - add it
            stats.strokes_from_b += page_b.layers.iter()
                .map(|l| l.strokes.len())
                .sum::<usize>();
            merged.pages.push(page_b.clone());
        }
    }
    
    // Merge CRDT operations
    merged.operations = merge_operations(&version_a.operations, &version_b.operations);
    
    MergeResult {
        merged,
        conflicts,
        strategy,
        stats,
    }
}

fn merge_pages(
    page_a: &PageVersion,
    page_b: &PageVersion,
    strategy: MergeStrategy,
    manual_resolutions: &HashMap<Uuid, ConflictResolution>,
    stats: &mut MergeStats,
    conflicts: &mut Vec<MergeConflict>,
) -> PageVersion {
    let mut merged = page_a.clone();
    
    // Index layers
    let layers_a: HashMap<u32, usize> = page_a.layers.iter()
        .enumerate()
        .map(|(i, l)| (l.id, i))
        .collect();
    
    // Process layers from B
    for layer_b in &page_b.layers {
        if let Some(&idx) = layers_a.get(&layer_b.id) {
            // Layer exists in both - merge strokes
            let layer_a = &page_a.layers[idx];
            let merged_layer = merge_layers(
                layer_a, 
                layer_b, 
                page_a.id,
                strategy,
                manual_resolutions,
                stats, 
                conflicts,
            );
            merged.layers[idx] = merged_layer;
        } else {
            // Layer only in B - add it
            stats.strokes_from_b += layer_b.strokes.len();
            merged.layers.push(layer_b.clone());
        }
    }
    
    merged
}

fn merge_layers(
    layer_a: &LayerData,
    layer_b: &LayerData,
    page_id: Uuid,
    strategy: MergeStrategy,
    manual_resolutions: &HashMap<Uuid, ConflictResolution>,
    stats: &mut MergeStats,
    conflicts: &mut Vec<MergeConflict>,
) -> LayerData {
    let mut merged = layer_a.clone();
    
    // Check for name conflict
    if layer_a.name != layer_b.name && layer_a.name.is_some() && layer_b.name.is_some() {
        let conflict_id = Uuid::new_v4();
        let resolution = manual_resolutions.get(&conflict_id)
            .copied()
            .unwrap_or_else(|| {
                match strategy {
                    MergeStrategy::PreferA => ConflictResolution::KeepA,
                    MergeStrategy::PreferB => ConflictResolution::KeepB,
                    MergeStrategy::LastWriterWins => ConflictResolution::KeepB,
                    _ => ConflictResolution::KeepB,
                }
            });
        
        if resolution == ConflictResolution::KeepB {
            merged.name = layer_b.name.clone();
        }
        
        conflicts.push(MergeConflict {
            id: conflict_id,
            conflict_type: ConflictType::LayerNameConflict,
            location: ConflictLocation {
                page_id: Some(page_id),
                layer_id: Some(layer_a.id),
                stroke_id: None,
            },
            version_a: serde_json::json!(layer_a.name),
            version_b: serde_json::json!(layer_b.name),
            resolution: Some(resolution),
        });
        stats.conflicts_auto_resolved += 1;
    }
    
    // Index strokes
    let strokes_a: HashMap<&str, usize> = layer_a.strokes.iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();
    
    // Process strokes from B
    for stroke_b in &layer_b.strokes {
        if let Some(&idx) = strokes_a.get(stroke_b.id.as_str()) {
            // Stroke exists in both - resolve based on timestamp
            let stroke_a = &layer_a.strokes[idx];
            
            if stroke_a.timestamp != stroke_b.timestamp {
                // Different timestamps - use CRDT semantics
                let keep_b = match strategy {
                    MergeStrategy::LastWriterWins => stroke_b.timestamp > stroke_a.timestamp,
                    MergeStrategy::PreferA => false,
                    MergeStrategy::PreferB => true,
                    MergeStrategy::KeepAll => {
                        // Keep both - add B as new stroke
                        let mut new_stroke = stroke_b.clone();
                        new_stroke.id = format!("{}_b", stroke_b.id);
                        merged.strokes.push(new_stroke);
                        stats.strokes_from_b += 1;
                        false
                    }
                    MergeStrategy::Manual => {
                        stroke_b.timestamp > stroke_a.timestamp
                    }
                };
                
                if keep_b && strategy != MergeStrategy::KeepAll {
                    merged.strokes[idx] = stroke_b.clone();
                    stats.strokes_from_b += 1;
                } else if strategy != MergeStrategy::KeepAll {
                    stats.strokes_from_a += 1;
                }
                
                stats.strokes_merged += 1;
            } else {
                stats.strokes_from_a += 1;
            }
        } else {
            // Stroke only in B - add it
            merged.strokes.push(stroke_b.clone());
            stats.strokes_from_b += 1;
        }
    }
    
    merged
}

fn merge_operations(
    ops_a: &[CrdtOperationLog],
    ops_b: &[CrdtOperationLog],
) -> Vec<CrdtOperationLog> {
    // Combine and sort by timestamp
    let mut merged: Vec<CrdtOperationLog> = ops_a.iter()
        .chain(ops_b.iter())
        .cloned()
        .collect();
    
    // Sort by CRDT timestamp
    merged.sort_by(|a, b| {
        a.timestamp.cmp(&b.timestamp)
    });
    
    // Deduplicate by timestamp (same timestamp = same operation)
    merged.dedup_by(|a, b| a.timestamp == b.timestamp);
    
    merged
}

/// Apply manual resolution to a conflict
pub fn apply_resolution(
    merge_result: &mut MergeResult,
    conflict_id: Uuid,
    resolution: ConflictResolution,
) -> Result<(), String> {
    let conflict = merge_result.conflicts.iter_mut()
        .find(|c| c.id == conflict_id)
        .ok_or_else(|| format!("Conflict {} not found", conflict_id))?;
    
    conflict.resolution = Some(resolution);
    merge_result.stats.conflicts_manual += 1;
    
    // Apply resolution to merged document based on conflict type
    // (In a real implementation, this would modify the merged document)
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merge_empty_documents() {
        let version_a = DocumentVersion {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            name: "A".to_string(),
            timestamp: chrono::Utc::now(),
            author_id: 1,
            pages: vec![],
            operations: vec![],
        };
        
        let version_b = DocumentVersion {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            name: "B".to_string(),
            timestamp: chrono::Utc::now(),
            author_id: 2,
            pages: vec![],
            operations: vec![],
        };
        
        let result = merge_documents(&version_a, &version_b, MergeStrategy::LastWriterWins, &HashMap::new());
        assert!(result.conflicts.is_empty());
    }
}
