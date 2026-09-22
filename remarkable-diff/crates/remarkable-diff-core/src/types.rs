//! Core types for CRDT diff/merge visualization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A document version containing CRDT state and stroke data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    /// Unique identifier
    pub id: Uuid,
    /// Document UUID
    pub document_id: Uuid,
    /// Version name/label
    pub name: String,
    /// Timestamp when version was created
    pub timestamp: DateTime<Utc>,
    /// Author device ID
    pub author_id: u64,
    /// Pages in this version
    pub pages: Vec<PageVersion>,
    /// Raw CRDT operations (for timeline)
    pub operations: Vec<CrdtOperationLog>,
}

/// A page version with stroke data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageVersion {
    /// Page UUID
    pub id: Uuid,
    /// Page index (fractional, e.g., "ba", "bb")
    pub index: String,
    /// Layers in this page
    pub layers: Vec<LayerData>,
    /// Template name
    pub template: Option<String>,
}

/// Layer data with strokes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerData {
    /// Layer identifier
    pub id: u32,
    /// Layer name
    pub name: Option<String>,
    /// Is layer visible
    pub visible: bool,
    /// Strokes in this layer
    pub strokes: Vec<StrokeData>,
}

/// Stroke data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeData {
    /// Unique stroke ID
    pub id: String,
    /// CRDT timestamp
    pub timestamp: CrdtTimestamp,
    /// Author ID
    pub author_id: u64,
    /// Pen type
    pub pen_type: PenType,
    /// Color
    pub color: StrokeColor,
    /// Points (x, y, pressure, width, speed)
    pub points: Vec<StrokePoint>,
    /// Is this stroke deleted (tombstone)
    pub deleted: bool,
}

/// CRDT Lamport timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CrdtTimestamp {
    pub replica: u64,
    pub counter: u64,
}

impl CrdtTimestamp {
    pub fn new(replica: u64, counter: u64) -> Self {
        Self { replica, counter }
    }
    
    pub fn parse(s: &str) -> Option<Self> {
        let mut parts = s.split(':');
        let replica = parts.next()?.parse().ok()?;
        let counter = parts.next()?.parse().ok()?;
        Some(Self { replica, counter })
    }
}

impl std::fmt::Display for CrdtTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.replica, self.counter)
    }
}

/// Stroke point
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrokePoint {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    pub width: f32,
    pub speed: f32,
}

/// Pen types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PenType {
    BallPoint,
    Marker,
    Fineliner,
    SharpPencil,
    TiltPencil,
    Brush,
    Highlighter,
    Eraser,
    EraseArea,
    EraseAll,
    Calligraphy,
    Pen,
    SelectionBrush,
    Unknown(u32),
}

/// Stroke colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    Unknown(u32),
}

/// Logged CRDT operation for timeline visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtOperationLog {
    /// Operation type
    pub operation: CrdtOperationType,
    /// CRDT timestamp
    pub timestamp: CrdtTimestamp,
    /// Author ID
    pub author_id: u64,
    /// Target item ID(s)
    pub target_ids: Vec<String>,
    /// Additional data (varies by operation)
    #[serde(default)]
    pub data: serde_json::Value,
    /// Human-readable timestamp
    pub wall_time: Option<DateTime<Utc>>,
}

/// CRDT operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrdtOperationType {
    AddItem,
    AddItems,
    DeleteItems,
    DeleteAppendItems,
    InsertItemsAfter,
    SwapItems,
    CreateGroup,
    DeleteGroup,
    AddLayer,
    DeleteLayer,
    MoveLayer,
    MergeLayerDown,
    SetLayerName,
    SetLayerVisible,
    TextInsert,
    TextRemove,
}

impl CrdtOperationType {
    /// Category for grouping in timeline
    pub fn category(&self) -> &'static str {
        match self {
            Self::AddItem | Self::AddItems | Self::DeleteItems | 
            Self::DeleteAppendItems | Self::InsertItemsAfter | Self::SwapItems => "item",
            Self::CreateGroup | Self::DeleteGroup => "group",
            Self::AddLayer | Self::DeleteLayer | Self::MoveLayer | 
            Self::MergeLayerDown | Self::SetLayerName | Self::SetLayerVisible => "layer",
            Self::TextInsert | Self::TextRemove => "text",
        }
    }
    
    /// Is this an addition operation?
    pub fn is_addition(&self) -> bool {
        matches!(self, 
            Self::AddItem | Self::AddItems | Self::InsertItemsAfter |
            Self::CreateGroup | Self::AddLayer | Self::TextInsert
        )
    }
    
    /// Is this a deletion operation?
    pub fn is_deletion(&self) -> bool {
        matches!(self,
            Self::DeleteItems | Self::DeleteAppendItems |
            Self::DeleteGroup | Self::DeleteLayer | Self::TextRemove
        )
    }
}
