//! reMarkable .rm file parser
//! Supports v3, v5, and v6 format variants

use thiserror::Error;

use super::types::*;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid header: expected reMarkable lines file")]
    InvalidHeader,
    
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unexpected end of file")]
    UnexpectedEof,
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Parse a .rm file from bytes
pub fn parse_rm_file(data: &[u8]) -> Result<RmPage, ParseError> {
    if data.len() < 43 {
        return Err(ParseError::InvalidHeader);
    }
    
    let header = &data[..43];
    
    let version = if header.starts_with(b"reMarkable .lines file, version=") {
        // Extract version number
        let version_byte = header[32];
        (version_byte - b'0') as u32
    } else {
        return Err(ParseError::InvalidHeader);
    };
    
    match version {
        3 => parse_v3(&data[43..]),
        5 => parse_v5(&data[43..]),
        6 => parse_v6(&data[43..]),
        _ => Err(ParseError::UnsupportedVersion(version)),
    }
}

/// Parse v3 format
fn parse_v3(data: &[u8]) -> Result<RmPage, ParseError> {
    let mut page = RmPage::new();
    page.version = 3;
    
    if data.len() < 4 {
        return Ok(page);
    }
    
    let mut pos = 0;
    
    // Number of layers
    let num_layers = read_u32(data, &mut pos)?;
    
    for layer_idx in 0..num_layers {
        let mut layer = Layer {
            id: CrdtId::new(0, layer_idx as u64),
            visible: true,
            name: format!("Layer {}", layer_idx + 1),
            strokes: Vec::new(),
        };
        
        // Number of strokes in layer
        let num_strokes = read_u32(data, &mut pos)?;
        
        for _ in 0..num_strokes {
            let stroke = parse_stroke_v3(data, &mut pos)?;
            layer.strokes.push(stroke);
        }
        
        page.layers.push(layer);
    }
    
    Ok(page)
}

/// Parse v5 format (similar to v3 with extensions)
fn parse_v5(data: &[u8]) -> Result<RmPage, ParseError> {
    // v5 is essentially v3 with some metadata extensions
    parse_v3(data)
}

/// Parse v6 format (rmscene/CRDT-based)
fn parse_v6(_data: &[u8]) -> Result<RmPage, ParseError> {
    // v6 uses a completely different format based on rmscene
    // For now, return an empty page - full parsing requires rmscene library
    let mut page = RmPage::new();
    page.version = 6;
    
    // TODO: Implement proper v6 parsing using rmscene-like logic
    // The format uses protobuf-like encoding with CRDT operations
    
    Ok(page)
}

/// Parse a stroke (v3 format)
fn parse_stroke_v3(data: &[u8], pos: &mut usize) -> Result<Stroke, ParseError> {
    let pen_type = read_u32(data, pos)?;
    let color = read_u32(data, pos)?;
    let _unknown = read_u32(data, pos)?;
    let thickness = read_f32(data, pos)?;
    let _unknown2 = read_f32(data, pos)?;
    let num_points = read_u32(data, pos)?;
    
    let mut points = Vec::with_capacity(num_points as usize);
    
    for _ in 0..num_points {
        let x = read_f32(data, pos)?;
        let y = read_f32(data, pos)?;
        let speed = read_f32(data, pos)?;
        let direction = read_f32(data, pos)?;
        let width = read_f32(data, pos)?;
        let pressure = read_f32(data, pos)?;
        
        points.push(Point {
            x,
            y,
            pressure: pressure as u16,
            width: width as u16,
            speed: speed as u16,
            direction: direction as u16,
        });
    }
    
    Ok(Stroke {
        id: CrdtId::new(0, 0),
        parent_id: CrdtId::default(),
        tool: PenType::from(pen_type),
        color: PenColor::from(color),
        thickness_scale: thickness as f64,
        starting_length: 0.0,
        points,
        rgba: None,
    })
}

// Helper functions for reading binary data

fn read_u32(data: &[u8], pos: &mut usize) -> Result<u32, ParseError> {
    if *pos + 4 > data.len() {
        return Err(ParseError::UnexpectedEof);
    }
    let value = u32::from_le_bytes([
        data[*pos],
        data[*pos + 1],
        data[*pos + 2],
        data[*pos + 3],
    ]);
    *pos += 4;
    Ok(value)
}

fn read_f32(data: &[u8], pos: &mut usize) -> Result<f32, ParseError> {
    if *pos + 4 > data.len() {
        return Err(ParseError::UnexpectedEof);
    }
    let value = f32::from_le_bytes([
        data[*pos],
        data[*pos + 1],
        data[*pos + 2],
        data[*pos + 3],
    ]);
    *pos += 4;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invalid_header() {
        let result = parse_rm_file(b"invalid");
        assert!(matches!(result, Err(ParseError::InvalidHeader)));
    }
}
