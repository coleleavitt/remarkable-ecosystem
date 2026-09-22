//! Diff computation between document versions
//!
//! Computes stroke-level and operation-level differences
//! for visualization in the web UI.

use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Result of diffing two document versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDiff {
    /// Version A (base/left)
    pub version_a: VersionInfo,
    /// Version B (changed/right)
    pub version_b: VersionInfo,
    /// Page-level diffs
    pub page_diffs: Vec<PageDiff>,
    /// Summary statistics
    pub stats: DiffStats,
}

/// Version info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: Uuid,
    pub name: String,
    pub author_id: u64,
}

/// Diff statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    /// Total strokes added
    pub strokes_added: usize,
    /// Total strokes removed
    pub strokes_removed: usize,
    /// Total strokes modified
    pub strokes_modified: usize,
    /// Total strokes unchanged
    pub strokes_unchanged: usize,
    /// Pages added
    pub pages_added: usize,
    /// Pages removed
    pub pages_removed: usize,
    /// Operations by type
    pub operations_by_type: HashMap<String, usize>,
}

/// Page-level diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDiff {
    /// Page UUID (may be from A or B or both)
    pub page_id: Uuid,
    /// Status of this page
    pub status: PageDiffStatus,
    /// Layer diffs
    pub layer_diffs: Vec<LayerDiff>,
    /// Combined stroke diffs (all layers)
    pub stroke_diffs: Vec<StrokeDiff>,
}

/// Page diff status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageDiffStatus {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// Layer-level diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDiff {
    /// Layer ID
    pub layer_id: u32,
    /// Layer name in A
    pub name_a: Option<String>,
    /// Layer name in B
    pub name_b: Option<String>,
    /// Status
    pub status: LayerDiffStatus,
}

/// Layer diff status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerDiffStatus {
    Added,
    Removed,
    Renamed,
    Unchanged,
}

/// Individual stroke diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeDiff {
    /// Stroke ID
    pub stroke_id: String,
    /// Layer containing this stroke
    pub layer_id: u32,
    /// Diff status
    pub status: StrokeDiffStatus,
    /// Stroke from version A (if exists)
    pub stroke_a: Option<StrokeData>,
    /// Stroke from version B (if exists)
    pub stroke_b: Option<StrokeData>,
    /// Author of the change (if any)
    pub change_author: Option<u64>,
    /// Timestamp of change
    pub change_timestamp: Option<CrdtTimestamp>,
}

/// Stroke diff status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrokeDiffStatus {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// Compute diff between two document versions
pub fn compute_diff(version_a: &DocumentVersion, version_b: &DocumentVersion) -> DocumentDiff {
    let mut stats = DiffStats::default();
    let mut page_diffs = Vec::new();
    
    // Index pages by ID
    let pages_a: HashMap<Uuid, &PageVersion> = version_a.pages.iter()
        .map(|p| (p.id, p))
        .collect();
    let pages_b: HashMap<Uuid, &PageVersion> = version_b.pages.iter()
        .map(|p| (p.id, p))
        .collect();
    
    let all_page_ids: HashSet<Uuid> = pages_a.keys()
        .chain(pages_b.keys())
        .copied()
        .collect();
    
    for page_id in all_page_ids {
        let page_a = pages_a.get(&page_id);
        let page_b = pages_b.get(&page_id);
        
        let page_diff = match (page_a, page_b) {
            (None, Some(pb)) => {
                stats.pages_added += 1;
                compute_page_diff_added(pb, &mut stats)
            }
            (Some(pa), None) => {
                stats.pages_removed += 1;
                compute_page_diff_removed(pa, &mut stats)
            }
            (Some(pa), Some(pb)) => {
                compute_page_diff(pa, pb, &mut stats)
            }
            (None, None) => unreachable!(),
        };
        
        page_diffs.push(page_diff);
    }
    
    // Count operations by type
    for op in &version_b.operations {
        let key = format!("{:?}", op.operation).to_lowercase();
        *stats.operations_by_type.entry(key).or_insert(0) += 1;
    }
    
    DocumentDiff {
        version_a: VersionInfo {
            id: version_a.id,
            name: version_a.name.clone(),
            author_id: version_a.author_id,
        },
        version_b: VersionInfo {
            id: version_b.id,
            name: version_b.name.clone(),
            author_id: version_b.author_id,
        },
        page_diffs,
        stats,
    }
}

fn compute_page_diff_added(page: &PageVersion, stats: &mut DiffStats) -> PageDiff {
    let mut stroke_diffs = Vec::new();
    
    for layer in &page.layers {
        for stroke in &layer.strokes {
            stats.strokes_added += 1;
            stroke_diffs.push(StrokeDiff {
                stroke_id: stroke.id.clone(),
                layer_id: layer.id,
                status: StrokeDiffStatus::Added,
                stroke_a: None,
                stroke_b: Some(stroke.clone()),
                change_author: Some(stroke.author_id),
                change_timestamp: Some(stroke.timestamp),
            });
        }
    }
    
    PageDiff {
        page_id: page.id,
        status: PageDiffStatus::Added,
        layer_diffs: page.layers.iter().map(|l| LayerDiff {
            layer_id: l.id,
            name_a: None,
            name_b: l.name.clone(),
            status: LayerDiffStatus::Added,
        }).collect(),
        stroke_diffs,
    }
}

fn compute_page_diff_removed(page: &PageVersion, stats: &mut DiffStats) -> PageDiff {
    let mut stroke_diffs = Vec::new();
    
    for layer in &page.layers {
        for stroke in &layer.strokes {
            stats.strokes_removed += 1;
            stroke_diffs.push(StrokeDiff {
                stroke_id: stroke.id.clone(),
                layer_id: layer.id,
                status: StrokeDiffStatus::Removed,
                stroke_a: Some(stroke.clone()),
                stroke_b: None,
                change_author: None,
                change_timestamp: None,
            });
        }
    }
    
    PageDiff {
        page_id: page.id,
        status: PageDiffStatus::Removed,
        layer_diffs: page.layers.iter().map(|l| LayerDiff {
            layer_id: l.id,
            name_a: l.name.clone(),
            name_b: None,
            status: LayerDiffStatus::Removed,
        }).collect(),
        stroke_diffs,
    }
}

fn compute_page_diff(page_a: &PageVersion, page_b: &PageVersion, stats: &mut DiffStats) -> PageDiff {
    // Index strokes by ID from both versions
    let mut strokes_a: HashMap<&str, (&LayerData, &StrokeData)> = HashMap::new();
    let mut strokes_b: HashMap<&str, (&LayerData, &StrokeData)> = HashMap::new();
    
    for layer in &page_a.layers {
        for stroke in &layer.strokes {
            strokes_a.insert(&stroke.id, (layer, stroke));
        }
    }
    
    for layer in &page_b.layers {
        for stroke in &layer.strokes {
            strokes_b.insert(&stroke.id, (layer, stroke));
        }
    }
    
    let all_stroke_ids: HashSet<&str> = strokes_a.keys()
        .chain(strokes_b.keys())
        .copied()
        .collect();
    
    let mut stroke_diffs = Vec::new();
    let mut has_changes = false;
    
    for stroke_id in all_stroke_ids {
        let a = strokes_a.get(stroke_id);
        let b = strokes_b.get(stroke_id);
        
        let diff = match (a, b) {
            (None, Some((layer, stroke))) => {
                has_changes = true;
                stats.strokes_added += 1;
                StrokeDiff {
                    stroke_id: stroke_id.to_string(),
                    layer_id: layer.id,
                    status: StrokeDiffStatus::Added,
                    stroke_a: None,
                    stroke_b: Some((*stroke).clone()),
                    change_author: Some(stroke.author_id),
                    change_timestamp: Some(stroke.timestamp),
                }
            }
            (Some((layer, stroke)), None) => {
                has_changes = true;
                stats.strokes_removed += 1;
                StrokeDiff {
                    stroke_id: stroke_id.to_string(),
                    layer_id: layer.id,
                    status: StrokeDiffStatus::Removed,
                    stroke_a: Some((*stroke).clone()),
                    stroke_b: None,
                    change_author: None,
                    change_timestamp: None,
                }
            }
            (Some((layer_a, stroke_a)), Some((layer_b, stroke_b))) => {
                // Compare stroke content
                let modified = stroke_a.points.len() != stroke_b.points.len()
                    || stroke_a.pen_type != stroke_b.pen_type
                    || stroke_a.color != stroke_b.color
                    || stroke_a.deleted != stroke_b.deleted;
                
                if modified {
                    has_changes = true;
                    stats.strokes_modified += 1;
                    StrokeDiff {
                        stroke_id: stroke_id.to_string(),
                        layer_id: layer_b.id,
                        status: StrokeDiffStatus::Modified,
                        stroke_a: Some((*stroke_a).clone()),
                        stroke_b: Some((*stroke_b).clone()),
                        change_author: if stroke_b.timestamp > stroke_a.timestamp {
                            Some(stroke_b.author_id)
                        } else {
                            None
                        },
                        change_timestamp: Some(stroke_b.timestamp),
                    }
                } else {
                    stats.strokes_unchanged += 1;
                    StrokeDiff {
                        stroke_id: stroke_id.to_string(),
                        layer_id: layer_a.id,
                        status: StrokeDiffStatus::Unchanged,
                        stroke_a: Some((*stroke_a).clone()),
                        stroke_b: Some((*stroke_b).clone()),
                        change_author: None,
                        change_timestamp: None,
                    }
                }
            }
            (None, None) => unreachable!(),
        };
        
        stroke_diffs.push(diff);
    }
    
    // Compute layer diffs
    let layers_a: HashMap<u32, &LayerData> = page_a.layers.iter()
        .map(|l| (l.id, l))
        .collect();
    let layers_b: HashMap<u32, &LayerData> = page_b.layers.iter()
        .map(|l| (l.id, l))
        .collect();
    
    let all_layer_ids: HashSet<u32> = layers_a.keys()
        .chain(layers_b.keys())
        .copied()
        .collect();
    
    let layer_diffs: Vec<LayerDiff> = all_layer_ids.iter().map(|&lid| {
        match (layers_a.get(&lid), layers_b.get(&lid)) {
            (None, Some(lb)) => LayerDiff {
                layer_id: lid,
                name_a: None,
                name_b: lb.name.clone(),
                status: LayerDiffStatus::Added,
            },
            (Some(la), None) => LayerDiff {
                layer_id: lid,
                name_a: la.name.clone(),
                name_b: None,
                status: LayerDiffStatus::Removed,
            },
            (Some(la), Some(lb)) => {
                let status = if la.name != lb.name {
                    LayerDiffStatus::Renamed
                } else {
                    LayerDiffStatus::Unchanged
                };
                LayerDiff {
                    layer_id: lid,
                    name_a: la.name.clone(),
                    name_b: lb.name.clone(),
                    status,
                }
            }
            (None, None) => unreachable!(),
        }
    }).collect();
    
    PageDiff {
        page_id: page_a.id,
        status: if has_changes { PageDiffStatus::Modified } else { PageDiffStatus::Unchanged },
        layer_diffs,
        stroke_diffs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_diff() {
        let version = DocumentVersion {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            name: "Test".to_string(),
            timestamp: chrono::Utc::now(),
            author_id: 1,
            pages: vec![],
            operations: vec![],
        };
        
        let diff = compute_diff(&version, &version);
        assert_eq!(diff.stats.strokes_added, 0);
        assert_eq!(diff.stats.strokes_removed, 0);
    }
}
