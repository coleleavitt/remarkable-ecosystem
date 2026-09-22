//! CRDT operations for reMarkable documents.
//!
//! Based on the 14 CRDT operations identified in the xochitl binary:
//! - Scene item operations (add, delete, swap)
//! - Layer operations (add, move, delete, merge, visibility, name)
//! - Group operations (create, delete)
//! - Erase operations
//! - Text operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::id::LamportId;

/// A 2D point with pressure/width data for strokes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    pub width: f32,
    pub speed: f32,
    pub direction: f32,
}

impl StrokePoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            pressure: 1.0,
            width: 2.0,
            speed: 0.0,
            direction: 0.0,
        }
    }
}

/// Stroke data for a line item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// Unique ID for this stroke.
    pub id: Uuid,
    /// Pen type used (ballpoint, fineliner, marker, etc.).
    pub pen_type: PenType,
    /// Stroke color.
    pub color: StrokeColor,
    /// Base stroke width.
    pub base_width: f32,
    /// Points making up the stroke.
    pub points: Vec<StrokePoint>,
}

/// Pen types available on reMarkable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PenType {
    Ballpoint,
    BallpointV2,
    Fineliner,
    FinelinerV2,
    Marker,
    MarkerV2,
    Pencil,
    PencilV2,
    SharpPencil,
    SharpPencilV2,
    Highlighter,
    HighlighterV2,
    Eraser,
    EraserArea,
    Calligraphy,
    CalligraphyV2,
    Paintbrush,
    PaintbrushV2,
}

/// Stroke colors available on reMarkable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrokeColor {
    Black,
    Gray,
    White,
    Yellow,
    Green,
    Pink,
    Blue,
    Red,
    GrayOverlap,
}

impl StrokeColor {
    /// Convert to RGB values.
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Black => (0, 0, 0),
            Self::Gray => (125, 125, 125),
            Self::White => (255, 255, 255),
            Self::Yellow => (251, 247, 25),
            Self::Green => (0, 255, 0),
            Self::Pink => (255, 192, 203),
            Self::Blue => (0, 0, 255),
            Self::Red => (255, 0, 0),
            Self::GrayOverlap => (125, 125, 125),
        }
    }
}

/// A layer in the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
}

/// A group of items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub item_ids: Vec<Uuid>,
}

/// Text formatting state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextFormat {
    pub bold: bool,
    pub italic: bool,
    pub font_size: f32,
}

impl Default for TextFormat {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            font_size: 12.0,
        }
    }
}

/// CRDT operations for scene manipulation.
///
/// These mirror the operations found in the xochitl binary:
/// - SceneAddItemActionCrdt
/// - SceneDeleteItemsActionCrdt
/// - SceneAddLayerActionCrdt
/// - etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CrdtOp {
    /// Add a single stroke item.
    AddItem {
        id: LamportId,
        layer_id: Uuid,
        stroke: Stroke,
    },

    /// Add multiple stroke items at once.
    AddItems {
        id: LamportId,
        layer_id: Uuid,
        strokes: Vec<Stroke>,
    },

    /// Delete items (creates tombstones).
    DeleteItems {
        id: LamportId,
        item_ids: Vec<Uuid>,
    },

    /// Swap item positions.
    SwapItems {
        id: LamportId,
        item_a: Uuid,
        item_b: Uuid,
    },

    /// Add a new layer.
    AddLayer {
        id: LamportId,
        layer: Layer,
        index: usize,
    },

    /// Move a layer to a new position.
    MoveLayer {
        id: LamportId,
        layer_id: Uuid,
        new_index: usize,
    },

    /// Delete a layer.
    DeleteLayer {
        id: LamportId,
        layer_id: Uuid,
    },

    /// Set layer name.
    SetLayerName {
        id: LamportId,
        layer_id: Uuid,
        name: String,
    },

    /// Set layer visibility.
    SetLayerVisible {
        id: LamportId,
        layer_id: Uuid,
        visible: bool,
    },

    /// Merge layer down into the one below.
    MergeLayerDown {
        id: LamportId,
        layer_id: Uuid,
    },

    /// Create a group of items.
    CreateGroup {
        id: LamportId,
        group: Group,
    },

    /// Delete a group (ungroups items, doesn't delete them).
    DeleteGroup {
        id: LamportId,
        group_id: Uuid,
    },

    /// Erase with a line (eraser stroke).
    EraseWithLine {
        id: LamportId,
        eraser_stroke: Stroke,
        affected_items: Vec<Uuid>,
    },

    /// Insert text.
    TextInsert {
        id: LamportId,
        text_id: Uuid,
        position: usize,
        content: String,
        format: TextFormat,
    },

    /// Remove text.
    TextRemove {
        id: LamportId,
        text_id: Uuid,
        position: usize,
        length: usize,
    },

    /// Cursor position update (for presence).
    CursorMove {
        id: LamportId,
        user_id: Uuid,
        x: f32,
        y: f32,
        page: Uuid,
    },

    /// No-op (used for presence keepalive).
    Noop {
        id: LamportId,
    },
}

impl CrdtOp {
    /// Get the Lamport ID for this operation.
    pub fn lamport_id(&self) -> LamportId {
        match self {
            Self::AddItem { id, .. } => *id,
            Self::AddItems { id, .. } => *id,
            Self::DeleteItems { id, .. } => *id,
            Self::SwapItems { id, .. } => *id,
            Self::AddLayer { id, .. } => *id,
            Self::MoveLayer { id, .. } => *id,
            Self::DeleteLayer { id, .. } => *id,
            Self::SetLayerName { id, .. } => *id,
            Self::SetLayerVisible { id, .. } => *id,
            Self::MergeLayerDown { id, .. } => *id,
            Self::CreateGroup { id, .. } => *id,
            Self::DeleteGroup { id, .. } => *id,
            Self::EraseWithLine { id, .. } => *id,
            Self::TextInsert { id, .. } => *id,
            Self::TextRemove { id, .. } => *id,
            Self::CursorMove { id, .. } => *id,
            Self::Noop { id } => *id,
        }
    }

    /// Check if this operation is a tombstone-creating operation.
    pub fn is_delete(&self) -> bool {
        matches!(
            self,
            Self::DeleteItems { .. }
                | Self::DeleteLayer { .. }
                | Self::DeleteGroup { .. }
                | Self::EraseWithLine { .. }
                | Self::TextRemove { .. }
        )
    }

    /// Check if this operation affects the given item.
    pub fn affects_item(&self, item_id: Uuid) -> bool {
        match self {
            Self::AddItem { stroke, .. } => stroke.id == item_id,
            Self::AddItems { strokes, .. } => strokes.iter().any(|s| s.id == item_id),
            Self::DeleteItems { item_ids, .. } => item_ids.contains(&item_id),
            Self::SwapItems { item_a, item_b, .. } => *item_a == item_id || *item_b == item_id,
            Self::CreateGroup { group, .. } => group.item_ids.contains(&item_id),
            Self::EraseWithLine {
                affected_items, ..
            } => affected_items.contains(&item_id),
            _ => false,
        }
    }
}

/// An operation with metadata for undo/redo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoableOp {
    /// The forward operation.
    pub op: CrdtOp,
    /// The reverse operation (for undo).
    pub inverse: Option<CrdtOp>,
    /// User who performed this operation.
    pub user_id: Uuid,
    /// Timestamp when the operation was created.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl UndoableOp {
    pub fn new(op: CrdtOp, user_id: Uuid) -> Self {
        Self {
            op,
            inverse: None,
            user_id,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_inverse(mut self, inverse: CrdtOp) -> Self {
        self.inverse = Some(inverse);
        self
    }
}
