//! Parse reMarkable files into our diff/merge types

use remarkable_diff_core::{
    DocumentVersion, PageVersion, LayerData,
    CrdtTimestamp, CrdtOperationLog, CrdtOperationType, PenType, StrokeColor,
};
use crate::error::{AppError, Result};
use uuid::Uuid;

/// Parse a .rm file (lines format) into stroke data
pub fn parse_rm_file(data: &[u8], page_id: Uuid) -> Result<PageVersion> {
    // Check header
    let header = b"reMarkable .lines file, version=";
    if !data.starts_with(header) {
        return Err(AppError::InvalidFormat("Not a valid .rm file".into()));
    }
    
    // Parse version
    let header_end = data.iter().position(|&b| b == b'\n')
        .ok_or_else(|| AppError::InvalidFormat("Missing header newline".into()))?;
    
    let version_str = std::str::from_utf8(&data[header.len()..header_end])
        .map_err(|e| AppError::ParseError(e.to_string()))?;
    
    let version: u32 = version_str.trim()
        .parse()
        .map_err(|_| AppError::InvalidFormat("Invalid version number".into()))?;
    
    if version != 6 && version != 5 && version != 3 {
        return Err(AppError::InvalidFormat(format!("Unsupported version: {}", version)));
    }
    
    // For now, return a placeholder - actual binary parsing would go here
    // In production, this would use remarkable-lines crate
    Ok(PageVersion {
        id: page_id,
        index: "ba".to_string(),
        layers: vec![LayerData {
            id: 0,
            name: Some("Layer 1".to_string()),
            visible: true,
            strokes: vec![],
        }],
        template: Some("Blank".to_string()),
    })
}

/// Parse .content JSON file for CRDT data
pub fn parse_content_file(data: &[u8]) -> Result<Vec<CrdtOperationLog>> {
    let content: serde_json::Value = serde_json::from_slice(data)
        .map_err(|e| AppError::ParseError(e.to_string()))?;
    
    let mut operations = Vec::new();
    
    // Extract CRDT timestamps from content structure
    if let Some(pages) = content.get("cPages").and_then(|cp| cp.get("pages")).and_then(|p| p.as_array()) {
        for (idx, page) in pages.iter().enumerate() {
            // Each page has CRDT-wrapped values
            if let Some(ts) = page.get("idx").and_then(|i| i.get("timestamp")).and_then(|t| t.as_str()) {
                if let Some(timestamp) = CrdtTimestamp::parse(ts) {
                    operations.push(CrdtOperationLog {
                        operation: CrdtOperationType::AddItem,
                        timestamp,
                        author_id: timestamp.replica,
                        target_ids: vec![page.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string()],
                        data: serde_json::json!({"page_index": idx}),
                        wall_time: None,
                    });
                }
            }
        }
    }
    
    Ok(operations)
}

/// Convert pen type from raw u32
pub fn parse_pen_type(raw: u32) -> PenType {
    match raw {
        0 => PenType::BallPoint,
        1 => PenType::Marker,
        2 => PenType::Fineliner,
        3 => PenType::SharpPencil,
        4 => PenType::TiltPencil,
        5 => PenType::Brush,
        6 => PenType::Highlighter,
        7 => PenType::Eraser,
        8 => PenType::EraseArea,
        9 => PenType::EraseAll,
        10 => PenType::Calligraphy,
        11 => PenType::Pen,
        12 => PenType::SelectionBrush,
        _ => PenType::Unknown(raw),
    }
}

/// Convert color from raw u32
pub fn parse_color(raw: u32) -> StrokeColor {
    match raw {
        0 => StrokeColor::Black,
        1 => StrokeColor::Gray,
        2 => StrokeColor::White,
        3 => StrokeColor::Yellow,
        4 => StrokeColor::Green,
        5 => StrokeColor::Pink,
        6 => StrokeColor::Blue,
        7 => StrokeColor::Red,
        8 => StrokeColor::GrayOverlap,
        _ => StrokeColor::Unknown(raw),
    }
}

/// Create a document version from uploaded files
pub fn create_version_from_upload(
    name: String,
    rm_files: Vec<(Uuid, Vec<u8>)>,
    content_data: Option<Vec<u8>>,
    author_id: u64,
) -> Result<DocumentVersion> {
    let document_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();
    
    let mut pages = Vec::with_capacity(rm_files.len());
    for (page_id, data) in rm_files {
        let page = parse_rm_file(&data, page_id)?;
        pages.push(page);
    }
    
    let operations = if let Some(data) = content_data {
        parse_content_file(&data)?
    } else {
        vec![]
    };
    
    Ok(DocumentVersion {
        id: version_id,
        document_id,
        name,
        timestamp: chrono::Utc::now(),
        author_id,
        pages,
        operations,
    })
}
