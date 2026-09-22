//! Core types for the .rm file format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Screen dimensions for reMarkable 2
pub const SCREEN_WIDTH: f64 = 1404.0;
pub const SCREEN_HEIGHT: f64 = 1872.0;

/// File header for version 6
pub const HEADER_V6: &[u8] = b"reMarkable .lines file, version=6          ";

/// CRDT identifier for sync conflict resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrdtId {
    /// Author/device index
    pub part1: u8,
    /// Sequence number
    pub part2: u64,
}

impl CrdtId {
    pub fn new(part1: u8, part2: u64) -> Self {
        Self { part1, part2 }
    }

    pub fn is_null(&self) -> bool {
        self.part1 == 0 && self.part2 == 0
    }
}

impl Default for CrdtId {
    fn default() -> Self {
        Self { part1: 0, part2: 0 }
    }
}

/// Pen/tool types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum PenType {
    Paintbrush1 = 0,
    Pencil1 = 1,
    Ballpoint1 = 2,
    Marker1 = 3,
    Fineliner1 = 4,
    Highlighter1 = 5,
    Eraser = 6,
    MechanicalPencil1 = 7,
    EraserArea = 8,
    Paintbrush2 = 12,
    MechanicalPencil2 = 13,
    Pencil2 = 14,
    Ballpoint2 = 15,
    Marker2 = 16,
    Fineliner2 = 17,
    Highlighter2 = 18,
    Calligraphy = 21,
    Shader = 23,
    Unknown(u32),
}

impl From<u32> for PenType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Paintbrush1,
            1 => Self::Pencil1,
            2 => Self::Ballpoint1,
            3 => Self::Marker1,
            4 => Self::Fineliner1,
            5 => Self::Highlighter1,
            6 => Self::Eraser,
            7 => Self::MechanicalPencil1,
            8 => Self::EraserArea,
            12 => Self::Paintbrush2,
            13 => Self::MechanicalPencil2,
            14 => Self::Pencil2,
            15 => Self::Ballpoint2,
            16 => Self::Marker2,
            17 => Self::Fineliner2,
            18 => Self::Highlighter2,
            21 => Self::Calligraphy,
            23 => Self::Shader,
            v => Self::Unknown(v),
        }
    }
}

impl PenType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Paintbrush1 | Self::Paintbrush2 => "Paintbrush",
            Self::Pencil1 | Self::Pencil2 => "Pencil",
            Self::Ballpoint1 | Self::Ballpoint2 => "Ballpoint",
            Self::Marker1 | Self::Marker2 => "Marker",
            Self::Fineliner1 | Self::Fineliner2 => "Fineliner",
            Self::Highlighter1 | Self::Highlighter2 => "Highlighter",
            Self::Eraser => "Eraser",
            Self::MechanicalPencil1 | Self::MechanicalPencil2 => "Mechanical Pencil",
            Self::EraserArea => "Area Eraser",
            Self::Calligraphy => "Calligraphy",
            Self::Shader => "Shader",
            Self::Unknown(_) => "Unknown",
        }
    }

    pub fn is_highlighter(&self) -> bool {
        matches!(self, Self::Highlighter1 | Self::Highlighter2)
    }
}

/// Pen colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum PenColor {
    Black = 0,
    Gray = 1,
    White = 2,
    Yellow = 3,
    Green = 4,
    Pink = 5,
    Blue = 6,
    Red = 7,
    GrayOverlap = 8,
    Highlight = 9,
    Green2 = 10,
    Cyan = 11,
    Magenta = 12,
    Yellow2 = 13,
    Unknown(u32),
}

impl From<u32> for PenColor {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Black,
            1 => Self::Gray,
            2 => Self::White,
            3 => Self::Yellow,
            4 => Self::Green,
            5 => Self::Pink,
            6 => Self::Blue,
            7 => Self::Red,
            8 => Self::GrayOverlap,
            9 => Self::Highlight,
            10 => Self::Green2,
            11 => Self::Cyan,
            12 => Self::Magenta,
            13 => Self::Yellow2,
            v => Self::Unknown(v),
        }
    }
}

impl PenColor {
    /// Get RGB hex color code
    pub fn to_rgb(&self) -> &'static str {
        match self {
            Self::Black => "#000000",
            Self::Gray => "#888888",
            Self::White => "#ffffff",
            Self::Yellow | Self::Yellow2 => "#f0c020",
            Self::Green | Self::Green2 => "#00a000",
            Self::Pink => "#ff80c0",
            Self::Blue => "#0060ff",
            Self::Red => "#ff0000",
            Self::GrayOverlap => "#666666",
            Self::Highlight => "#ffff00",
            Self::Cyan => "#00ffff",
            Self::Magenta => "#ff00ff",
            Self::Unknown(_) => "#000000",
        }
    }

    /// Get RGBA with alpha for highlighters
    pub fn to_rgba(&self, tool: PenType) -> (u8, u8, u8, u8) {
        let alpha = if tool.is_highlighter() { 128 } else { 255 };
        let rgb = self.to_rgb();
        let r = u8::from_str_radix(&rgb[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&rgb[3..5], 16).unwrap_or(0);
        let b = u8::from_str_radix(&rgb[5..7], 16).unwrap_or(0);
        (r, g, b, alpha)
    }
}

/// A single point in a stroke
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub speed: u16,
    pub direction: u16,
    pub width: u16,
    pub pressure: u16,
}

impl Point {
    /// Calculate display width based on pressure and base width
    pub fn display_width(&self, base_width: f64) -> f64 {
        base_width * (self.pressure as f64 / 2048.0).max(0.1)
    }
}

/// A stroke/line in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    /// CRDT identifier for sync
    pub id: CrdtId,
    /// Parent item ID
    pub parent_id: CrdtId,
    /// Pen/tool type
    pub tool: PenType,
    /// Stroke color
    pub color: PenColor,
    /// Thickness scale factor
    pub thickness_scale: f64,
    /// Starting length offset
    pub starting_length: f32,
    /// Points in the stroke
    pub points: Vec<Point>,
    /// Optional custom RGBA color
    pub rgba: Option<(u8, u8, u8, u8)>,
}

impl Stroke {
    /// Calculate bounding box
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        if self.points.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for p in &self.points {
            min_x = min_x.min(p.x as f64);
            max_x = max_x.max(p.x as f64);
            min_y = min_y.min(p.y as f64);
            max_y = max_y.max(p.y as f64);
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Average pressure (normalized 0-1)
    pub fn avg_pressure(&self) -> f64 {
        if self.points.is_empty() {
            return 0.5;
        }
        let sum: u64 = self.points.iter().map(|p| p.pressure as u64).sum();
        (sum as f64) / (self.points.len() as f64 * 2048.0)
    }
}

/// A layer containing strokes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: CrdtId,
    pub name: String,
    pub visible: bool,
    pub strokes: Vec<Stroke>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            id: CrdtId::default(),
            name: String::from("Layer 1"),
            visible: true,
            strokes: Vec::new(),
        }
    }
}

/// Text item in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextItem {
    pub id: CrdtId,
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub font_size: f32,
}

/// A parsed reMarkable document page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmPage {
    /// File format version
    pub version: u8,
    /// Layers with strokes
    pub layers: Vec<Layer>,
    /// Text items (typed text)
    pub text_items: Vec<TextItem>,
    /// Author ID mappings for sync
    pub author_ids: HashMap<u8, uuid::Uuid>,
    /// Page info
    pub page_id: Option<String>,
}

impl RmPage {
    pub fn new() -> Self {
        Self {
            version: 6,
            layers: vec![Layer::default()],
            text_items: Vec::new(),
            author_ids: HashMap::new(),
            page_id: None,
        }
    }

    /// Get all strokes across all layers
    pub fn all_strokes(&self) -> impl Iterator<Item = &Stroke> {
        self.layers.iter().flat_map(|l| &l.strokes)
    }

    /// Calculate document bounds
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for stroke in self.all_strokes() {
            let (sx, sy, ex, ey) = stroke.bounds();
            min_x = min_x.min(sx);
            max_x = max_x.max(ex);
            min_y = min_y.min(sy);
            max_y = max_y.max(ey);
        }

        if min_x == f64::MAX {
            (0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT)
        } else {
            (min_x, min_y, max_x, max_y)
        }
    }

    /// Stroke count
    pub fn stroke_count(&self) -> usize {
        self.layers.iter().map(|l| l.strokes.len()).sum()
    }

    /// Point count
    pub fn point_count(&self) -> usize {
        self.all_strokes().map(|s| s.points.len()).sum()
    }
}

impl Default for RmPage {
    fn default() -> Self {
        Self::new()
    }
}

/// Block types in the .rm file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockType {
    MigrationInfo = 0x00,
    SceneTree = 0x01,
    TreeNode = 0x02,
    SceneGlyphItem = 0x03,
    SceneGroupItem = 0x04,
    SceneLineItem = 0x05,
    SceneTextItem = 0x06,
    RootText = 0x07,
    SceneTombstone = 0x08,
    AuthorIds = 0x09,
    PageInfo = 0x0A,
    SceneInfo = 0x0D,
    Unknown(u8),
}

impl From<u8> for BlockType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::MigrationInfo,
            0x01 => Self::SceneTree,
            0x02 => Self::TreeNode,
            0x03 => Self::SceneGlyphItem,
            0x04 => Self::SceneGroupItem,
            0x05 => Self::SceneLineItem,
            0x06 => Self::SceneTextItem,
            0x07 => Self::RootText,
            0x08 => Self::SceneTombstone,
            0x09 => Self::AuthorIds,
            0x0A => Self::PageInfo,
            0x0D => Self::SceneInfo,
            v => Self::Unknown(v),
        }
    }
}

/// Tag types for value encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TagType {
    Byte1 = 0x1,
    Byte4 = 0x4,
    Byte8 = 0x8,
    Length4 = 0xC,
    Id = 0xF,
}

impl TryFrom<u8> for TagType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0xF {
            0x1 => Ok(Self::Byte1),
            0x4 => Ok(Self::Byte4),
            0x8 => Ok(Self::Byte8),
            0xC => Ok(Self::Length4),
            0xF => Ok(Self::Id),
            _ => Err(value),
        }
    }
}
