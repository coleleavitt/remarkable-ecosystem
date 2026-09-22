//! CRDT document state management.
//!
//! Handles applying operations, conflict resolution, and state queries.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::id::{LamportClock, LamportId};
use super::ops::{CrdtOp, Group, Layer, Stroke, UndoableOp};

/// Tombstone marker for deleted items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tombstone {
    pub deleted_at: LamportId,
    pub deleted_by: Uuid,
}

/// A page in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    /// Layers in order (bottom to top).
    pub layers: Vec<Layer>,
    /// Items by ID (strokes, text, etc.).
    pub items: HashMap<Uuid, PageItem>,
    /// Tombstones for deleted items.
    pub tombstones: HashMap<Uuid, Tombstone>,
    /// Groups of items.
    pub groups: HashMap<Uuid, Group>,
    /// Fractional index for page ordering.
    pub index: String,
}

/// An item on a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageItem {
    Stroke(Stroke),
    Text {
        id: Uuid,
        content: String,
        position: (f32, f32),
    },
}

impl PageItem {
    pub fn id(&self) -> Uuid {
        match self {
            Self::Stroke(s) => s.id,
            Self::Text { id, .. } => *id,
        }
    }
}

impl Page {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            layers: vec![Layer {
                id: Uuid::new_v4(),
                name: "Layer 1".to_string(),
                visible: true,
            }],
            items: HashMap::new(),
            tombstones: HashMap::new(),
            groups: HashMap::new(),
            index: "ba".to_string(),
        }
    }

    /// Check if an item has been deleted (is a tombstone).
    pub fn is_deleted(&self, item_id: &Uuid) -> bool {
        self.tombstones.contains_key(item_id)
    }

    /// Get all visible (non-tombstoned) items.
    pub fn visible_items(&self) -> impl Iterator<Item = &PageItem> {
        self.items
            .iter()
            .filter(|(id, _)| !self.tombstones.contains_key(id))
            .map(|(_, item)| item)
    }
}

/// The complete CRDT document state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtDocument {
    /// Document ID.
    pub id: Uuid,
    /// Document name.
    pub name: String,
    /// Pages in the document.
    pub pages: BTreeMap<String, Page>,
    /// Operation log (all operations ever applied).
    pub operations: Vec<CrdtOp>,
    /// Current generation (incremented on sync).
    pub generation: u64,
    /// Hash of current state.
    #[serde(skip)]
    state_hash: Option<String>,
}

impl CrdtDocument {
    pub fn new(id: Uuid, name: String) -> Self {
        let page = Page::new(Uuid::new_v4());
        let mut pages = BTreeMap::new();
        pages.insert(page.index.clone(), page);

        Self {
            id,
            name,
            pages,
            operations: Vec::new(),
            generation: 0,
            state_hash: None,
        }
    }

    /// Apply a CRDT operation.
    ///
    /// Returns true if the operation was applied, false if it was a duplicate
    /// or conflicted with existing state in a way that was rejected.
    pub fn apply(&mut self, op: CrdtOp) -> bool {
        // Check for duplicate operations
        if self
            .operations
            .iter()
            .any(|existing| existing.lamport_id() == op.lamport_id())
        {
            return false;
        }

        let result = self.apply_internal(&op);

        if result {
            self.operations.push(op);
            self.state_hash = None; // Invalidate cached hash
            self.generation += 1;
        }

        result
    }

    /// Internal apply without logging.
    fn apply_internal(&mut self, op: &CrdtOp) -> bool {
        match op {
            CrdtOp::AddItem {
                layer_id, stroke, ..
            } => {
                // Find the page containing this layer
                for page in self.pages.values_mut() {
                    if page.layers.iter().any(|l| l.id == *layer_id) {
                        page.items.insert(stroke.id, PageItem::Stroke(stroke.clone()));
                        return true;
                    }
                }
                false
            }

            CrdtOp::AddItems {
                layer_id, strokes, ..
            } => {
                for page in self.pages.values_mut() {
                    if page.layers.iter().any(|l| l.id == *layer_id) {
                        for stroke in strokes {
                            page.items.insert(stroke.id, PageItem::Stroke(stroke.clone()));
                        }
                        return true;
                    }
                }
                false
            }

            CrdtOp::DeleteItems { id, item_ids, .. } => {
                let mut deleted_any = false;
                for page in self.pages.values_mut() {
                    for item_id in item_ids {
                        if page.items.contains_key(item_id) && !page.tombstones.contains_key(item_id) {
                            page.tombstones.insert(
                                *item_id,
                                Tombstone {
                                    deleted_at: *id,
                                    deleted_by: Uuid::nil(), // Would be user ID
                                },
                            );
                            deleted_any = true;
                        }
                    }
                }
                deleted_any
            }

            CrdtOp::AddLayer {
                layer, index, ..
            } => {
                // Add to first page for now
                if let Some(page) = self.pages.values_mut().next() {
                    let idx = (*index).min(page.layers.len());
                    page.layers.insert(idx, layer.clone());
                    return true;
                }
                false
            }

            CrdtOp::DeleteLayer { layer_id, .. } => {
                for page in self.pages.values_mut() {
                    if let Some(pos) = page.layers.iter().position(|l| l.id == *layer_id) {
                        // Don't delete the last layer
                        if page.layers.len() > 1 {
                            page.layers.remove(pos);
                            return true;
                        }
                    }
                }
                false
            }

            CrdtOp::MoveLayer {
                layer_id,
                new_index,
                ..
            } => {
                for page in self.pages.values_mut() {
                    if let Some(pos) = page.layers.iter().position(|l| l.id == *layer_id) {
                        let layer = page.layers.remove(pos);
                        let new_pos = (*new_index).min(page.layers.len());
                        page.layers.insert(new_pos, layer);
                        return true;
                    }
                }
                false
            }

            CrdtOp::SetLayerName {
                layer_id, name, ..
            } => {
                for page in self.pages.values_mut() {
                    if let Some(layer) = page.layers.iter_mut().find(|l| l.id == *layer_id) {
                        layer.name = name.clone();
                        return true;
                    }
                }
                false
            }

            CrdtOp::SetLayerVisible {
                layer_id, visible, ..
            } => {
                for page in self.pages.values_mut() {
                    if let Some(layer) = page.layers.iter_mut().find(|l| l.id == *layer_id) {
                        layer.visible = *visible;
                        return true;
                    }
                }
                false
            }

            CrdtOp::MergeLayerDown { layer_id, .. } => {
                for page in self.pages.values_mut() {
                    if let Some(pos) = page.layers.iter().position(|l| l.id == *layer_id) {
                        if pos > 0 {
                            // Merge items from this layer to the one below
                            // For now, just remove the layer
                            page.layers.remove(pos);
                            return true;
                        }
                    }
                }
                false
            }

            CrdtOp::CreateGroup { group, .. } => {
                // Find page containing the items
                for page in self.pages.values_mut() {
                    if group.item_ids.iter().any(|id| page.items.contains_key(id)) {
                        page.groups.insert(group.id, group.clone());
                        return true;
                    }
                }
                false
            }

            CrdtOp::DeleteGroup { group_id, .. } => {
                for page in self.pages.values_mut() {
                    if page.groups.remove(group_id).is_some() {
                        return true;
                    }
                }
                false
            }

            CrdtOp::EraseWithLine {
                id,
                affected_items,
                ..
            } => {
                let mut erased_any = false;
                for page in self.pages.values_mut() {
                    for item_id in affected_items {
                        if page.items.contains_key(item_id) && !page.tombstones.contains_key(item_id) {
                            page.tombstones.insert(
                                *item_id,
                                Tombstone {
                                    deleted_at: *id,
                                    deleted_by: Uuid::nil(),
                                },
                            );
                            erased_any = true;
                        }
                    }
                }
                erased_any
            }

            CrdtOp::TextInsert { .. } | CrdtOp::TextRemove { .. } => {
                // Text operations - simplified for now
                true
            }

            CrdtOp::CursorMove { .. } | CrdtOp::Noop { .. } => {
                // These don't modify document state
                true
            }

            CrdtOp::SwapItems {
                item_a, item_b, ..
            } => {
                // Swap would require z-ordering; simplified for now
                for page in self.pages.values_mut() {
                    if page.items.contains_key(item_a) && page.items.contains_key(item_b) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Merge operations from another document state.
    ///
    /// Uses Last-Writer-Wins (LWW) conflict resolution.
    pub fn merge(&mut self, other: &CrdtDocument) {
        // Collect all operations and sort by Lamport ID
        let mut all_ops: Vec<CrdtOp> = self.operations.clone();

        for op in &other.operations {
            if !all_ops.iter().any(|existing| existing.lamport_id() == op.lamport_id()) {
                all_ops.push(op.clone());
            }
        }

        // Sort by Lamport ID (LWW: later timestamp wins)
        all_ops.sort_by_key(|op| op.lamport_id());

        // Rebuild state from scratch
        let id = self.id;
        let name = self.name.clone();
        *self = Self::new(id, name);

        // Re-apply all operations
        for op in all_ops {
            self.apply(op);
        }
    }

    /// Get operations since a given generation.
    pub fn ops_since(&self, generation: u64) -> Vec<&CrdtOp> {
        self.operations
            .iter()
            .filter(|op| op.lamport_id().sequence >= generation)
            .collect()
    }

    /// Compute a hash of the current state (for delta sync).
    pub fn compute_hash(&mut self) -> String {
        if let Some(ref hash) = self.state_hash {
            return hash.clone();
        }

        // Simple hash based on operation log
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for op in &self.operations {
            op.lamport_id().sequence.hash(&mut hasher);
            op.lamport_id().author.hash(&mut hasher);
        }

        let hash = format!("{:016x}", hasher.finish());
        self.state_hash = Some(hash.clone());
        hash
    }
}

/// Per-user undo/redo stack.
#[derive(Debug, Default)]
pub struct UndoStack {
    /// Operations that can be undone (most recent first).
    undo_stack: Vec<UndoableOp>,
    /// Operations that can be redone (most recent first).
    redo_stack: Vec<UndoableOp>,
    /// Maximum stack size.
    max_size: usize,
}

impl UndoStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Push an operation onto the undo stack.
    pub fn push(&mut self, op: UndoableOp) {
        self.undo_stack.push(op);
        // Clear redo stack on new operation
        self.redo_stack.clear();

        // Trim if over max size
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// Pop and return the operation to undo.
    pub fn undo(&mut self) -> Option<UndoableOp> {
        if let Some(op) = self.undo_stack.pop() {
            self.redo_stack.push(op.clone());
            Some(op)
        } else {
            None
        }
    }

    /// Pop and return the operation to redo.
    pub fn redo(&mut self) -> Option<UndoableOp> {
        if let Some(op) = self.redo_stack.pop() {
            self.undo_stack.push(op.clone());
            Some(op)
        } else {
            None
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::ops::{PenType, StrokeColor, StrokePoint};

    fn make_stroke(id: Uuid) -> Stroke {
        Stroke {
            id,
            pen_type: PenType::Fineliner,
            color: StrokeColor::Black,
            base_width: 2.0,
            points: vec![
                StrokePoint::new(0.0, 0.0),
                StrokePoint::new(100.0, 100.0),
            ],
        }
    }

    #[test]
    fn test_add_and_delete_stroke() {
        let mut doc = CrdtDocument::new(Uuid::new_v4(), "Test".to_string());
        let layer_id = doc.pages.values().next().unwrap().layers[0].id;
        let stroke_id = Uuid::new_v4();
        let stroke = make_stroke(stroke_id);

        // Add stroke
        let op = CrdtOp::AddItem {
            id: LamportId::new(1, 0),
            layer_id,
            stroke,
        };
        assert!(doc.apply(op));
        assert!(doc.pages.values().next().unwrap().items.contains_key(&stroke_id));

        // Delete stroke
        let del_op = CrdtOp::DeleteItems {
            id: LamportId::new(1, 1),
            item_ids: vec![stroke_id],
        };
        assert!(doc.apply(del_op));
        assert!(doc.pages.values().next().unwrap().is_deleted(&stroke_id));
    }


}
