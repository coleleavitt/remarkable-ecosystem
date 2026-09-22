//! RFB (Remote Framebuffer) Protocol Implementation
//!
//! Based on RFB 3.8 specification. reMarkable uses RFB over WebRTC DataChannel.

use bytes::{Buf, BufMut, BytesMut};
use std::io::{self, Cursor};
use tracing::{debug, trace, warn};

use crate::constants::{rfb_encoding, rfb_server_msg, FB_BPP, FB_HEIGHT, FB_WIDTH};
use crate::error::{Error, Result};

/// Pixel format descriptor
#[derive(Debug, Clone, Copy)]
pub struct PixelFormat {
    pub bits_per_pixel: u8,
    pub depth: u8,
    pub big_endian: bool,
    pub true_color: bool,
    pub red_max: u16,
    pub green_max: u16,
    pub blue_max: u16,
    pub red_shift: u8,
    pub green_shift: u8,
    pub blue_shift: u8,
}

impl Default for PixelFormat {
    /// Default 8-bit grayscale format (reMarkable standard)
    fn default() -> Self {
        Self {
            bits_per_pixel: 8,
            depth: 8,
            big_endian: false,
            true_color: true,
            red_max: 255,
            green_max: 255,
            blue_max: 255,
            red_shift: 0,
            green_shift: 0,
            blue_shift: 0,
        }
    }
}

impl PixelFormat {
    /// Create 8-bit grayscale format
    pub fn grayscale_8bit() -> Self {
        Self::default()
    }
    
    /// Create 16-bit RGB565 format
    pub fn rgb565() -> Self {
        Self {
            bits_per_pixel: 16,
            depth: 16,
            big_endian: false,
            true_color: true,
            red_max: 31,
            green_max: 63,
            blue_max: 31,
            red_shift: 11,
            green_shift: 5,
            blue_shift: 0,
        }
    }
    
    /// Bytes per pixel
    pub fn bytes_per_pixel(&self) -> usize {
        (self.bits_per_pixel / 8) as usize
    }
    
    /// Serialize to wire format (16 bytes)
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0] = self.bits_per_pixel;
        buf[1] = self.depth;
        buf[2] = if self.big_endian { 1 } else { 0 };
        buf[3] = if self.true_color { 1 } else { 0 };
        buf[4..6].copy_from_slice(&self.red_max.to_be_bytes());
        buf[6..8].copy_from_slice(&self.green_max.to_be_bytes());
        buf[8..10].copy_from_slice(&self.blue_max.to_be_bytes());
        buf[10] = self.red_shift;
        buf[11] = self.green_shift;
        buf[12] = self.blue_shift;
        // bytes 13-15 are padding
        buf
    }
    
    /// Parse from wire format
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(Error::RfbProtocol("Pixel format too short".into()));
        }
        Ok(Self {
            bits_per_pixel: data[0],
            depth: data[1],
            big_endian: data[2] != 0,
            true_color: data[3] != 0,
            red_max: u16::from_be_bytes([data[4], data[5]]),
            green_max: u16::from_be_bytes([data[6], data[7]]),
            blue_max: u16::from_be_bytes([data[8], data[9]]),
            red_shift: data[10],
            green_shift: data[11],
            blue_shift: data[12],
        })
    }
}

/// Framebuffer rectangle
#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub encoding: i32,
    pub data: Vec<u8>,
}

impl Rectangle {
    /// Number of pixels in this rectangle
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

/// Framebuffer update message
#[derive(Debug, Clone)]
pub struct FramebufferUpdate {
    pub rectangles: Vec<Rectangle>,
}

impl FramebufferUpdate {
    /// Check if this is a full-screen update
    pub fn is_full_screen(&self, width: u16, height: u16) -> bool {
        if self.rectangles.len() != 1 {
            return false;
        }
        let r = &self.rectangles[0];
        r.x == 0 && r.y == 0 && r.width == width && r.height == height
    }
}

/// RFB protocol decoder
pub struct RfbDecoder {
    width: u16,
    height: u16,
    pixel_format: PixelFormat,
    buffer: BytesMut,
    /// Current framebuffer (accumulated from incremental updates)
    framebuffer: Vec<u8>,
}

impl RfbDecoder {
    /// Create new decoder with default reMarkable 2 dimensions
    pub fn new() -> Self {
        Self::with_dimensions(FB_WIDTH as u16, FB_HEIGHT as u16)
    }
    
    /// Create decoder with custom dimensions
    pub fn with_dimensions(width: u16, height: u16) -> Self {
        let pixel_format = PixelFormat::default();
        let fb_size = width as usize * height as usize * pixel_format.bytes_per_pixel();
        Self {
            width,
            height,
            pixel_format,
            buffer: BytesMut::with_capacity(fb_size),
            framebuffer: vec![0u8; fb_size],
        }
    }
    
    /// Get current framebuffer dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
    
    /// Set pixel format
    pub fn set_pixel_format(&mut self, format: PixelFormat) {
        let old_bpp = self.pixel_format.bytes_per_pixel();
        let new_bpp = format.bytes_per_pixel();
        self.pixel_format = format;
        
        if old_bpp != new_bpp {
            let fb_size = self.width as usize * self.height as usize * new_bpp;
            self.framebuffer = vec![0u8; fb_size];
        }
    }
    
    /// Get current pixel format
    pub fn pixel_format(&self) -> &PixelFormat {
        &self.pixel_format
    }
    
    /// Feed incoming data and parse framebuffer updates
    pub fn feed(&mut self, data: &[u8]) -> Vec<FramebufferUpdate> {
        self.buffer.extend_from_slice(data);
        let mut updates = Vec::new();
        
        while let Some(update) = self.try_parse_update() {
            // Apply update to framebuffer
            self.apply_update(&update);
            updates.push(update);
        }
        
        updates
    }
    
    /// Get current framebuffer as bytes
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }
    
    /// Get current framebuffer as mutable
    pub fn framebuffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.framebuffer
    }
    
    /// Try to parse a framebuffer update from the buffer
    fn try_parse_update(&mut self) -> Option<FramebufferUpdate> {
        if self.buffer.is_empty() {
            return None;
        }
        
        // Need at least 4 bytes for header
        if self.buffer.len() < 4 {
            return None;
        }
        
        let msg_type = self.buffer[0];
        
        match msg_type {
            rfb_server_msg::FB_UPDATE => self.parse_fb_update(),
            rfb_server_msg::SET_COLOR_MAP => {
                debug!("Received SetColorMap message (ignoring)");
                self.skip_color_map()
            }
            rfb_server_msg::BELL => {
                debug!("Received Bell message");
                self.buffer.advance(1);
                None
            }
            rfb_server_msg::SERVER_CUT_TEXT => {
                debug!("Received ServerCutText message (ignoring)");
                self.skip_server_cut_text()
            }
            _ => {
                warn!("Unknown message type: {}", msg_type);
                None
            }
        }
    }
    
    /// Parse framebuffer update message
    fn parse_fb_update(&mut self) -> Option<FramebufferUpdate> {
        // Header: type (1) + padding (1) + num_rects (2) = 4 bytes
        if self.buffer.len() < 4 {
            return None;
        }
        
        let num_rects = u16::from_be_bytes([self.buffer[2], self.buffer[3]]) as usize;
        trace!("Parsing FB update with {} rectangles", num_rects);
        
        // Calculate total size needed
        let mut cursor = Cursor::new(&self.buffer[4..]);
        let mut rectangles = Vec::with_capacity(num_rects);
        let bpp = self.pixel_format.bytes_per_pixel();
        
        for i in 0..num_rects {
            // Each rectangle header: x(2) + y(2) + w(2) + h(2) + encoding(4) = 12 bytes
            if cursor.remaining() < 12 {
                trace!("Not enough data for rectangle {} header", i);
                return None;
            }
            
            let x = cursor.get_u16();
            let y = cursor.get_u16();
            let width = cursor.get_u16();
            let height = cursor.get_u16();
            let encoding = cursor.get_i32();
            
            let data_size = match encoding {
                rfb_encoding::RAW => width as usize * height as usize * bpp,
                rfb_encoding::COPYRECT => 4, // src_x (2) + src_y (2)
                _ => {
                    warn!("Unsupported encoding: {}", encoding);
                    return None;
                }
            };
            
            if cursor.remaining() < data_size {
                trace!("Not enough data for rectangle {} data ({} needed, {} available)", 
                       i, data_size, cursor.remaining());
                return None;
            }
            
            let mut data = vec![0u8; data_size];
            cursor.copy_to_slice(&mut data);
            
            rectangles.push(Rectangle {
                x, y, width, height, encoding, data
            });
        }
        
        // Calculate how much we consumed
        let consumed = 4 + cursor.position() as usize;
        self.buffer.advance(consumed);
        
        Some(FramebufferUpdate { rectangles })
    }
    
    /// Skip SetColorMap message
    fn skip_color_map(&mut self) -> Option<FramebufferUpdate> {
        // Header: type(1) + padding(1) + first_color(2) + num_colors(2) = 6 bytes
        if self.buffer.len() < 6 {
            return None;
        }
        let num_colors = u16::from_be_bytes([self.buffer[4], self.buffer[5]]) as usize;
        let total = 6 + num_colors * 6; // Each color is R(2) + G(2) + B(2)
        
        if self.buffer.len() < total {
            return None;
        }
        self.buffer.advance(total);
        None
    }
    
    /// Skip ServerCutText message
    fn skip_server_cut_text(&mut self) -> Option<FramebufferUpdate> {
        // Header: type(1) + padding(3) + length(4) = 8 bytes
        if self.buffer.len() < 8 {
            return None;
        }
        let length = u32::from_be_bytes([
            self.buffer[4], self.buffer[5], self.buffer[6], self.buffer[7]
        ]) as usize;
        let total = 8 + length;
        
        if self.buffer.len() < total {
            return None;
        }
        self.buffer.advance(total);
        None
    }
    
    /// Apply framebuffer update to internal buffer
    fn apply_update(&mut self, update: &FramebufferUpdate) {
        let bpp = self.pixel_format.bytes_per_pixel();
        let stride = self.width as usize * bpp;
        
        for rect in &update.rectangles {
            match rect.encoding {
                rfb_encoding::RAW => {
                    // Copy raw pixel data
                    for row in 0..rect.height as usize {
                        let fb_y = rect.y as usize + row;
                        let fb_offset = fb_y * stride + rect.x as usize * bpp;
                        let src_offset = row * rect.width as usize * bpp;
                        let row_size = rect.width as usize * bpp;
                        
                        if fb_offset + row_size <= self.framebuffer.len() 
                           && src_offset + row_size <= rect.data.len() {
                            self.framebuffer[fb_offset..fb_offset + row_size]
                                .copy_from_slice(&rect.data[src_offset..src_offset + row_size]);
                        }
                    }
                }
                rfb_encoding::COPYRECT => {
                    // Copy from another region
                    if rect.data.len() >= 4 {
                        let src_x = u16::from_be_bytes([rect.data[0], rect.data[1]]) as usize;
                        let src_y = u16::from_be_bytes([rect.data[2], rect.data[3]]) as usize;
                        
                        // Copy row by row (handle overlapping regions)
                        let mut temp = vec![0u8; rect.width as usize * rect.height as usize * bpp];
                        for row in 0..rect.height as usize {
                            let src_offset = (src_y + row) * stride + src_x * bpp;
                            let temp_offset = row * rect.width as usize * bpp;
                            let row_size = rect.width as usize * bpp;
                            
                            if src_offset + row_size <= self.framebuffer.len() {
                                temp[temp_offset..temp_offset + row_size]
                                    .copy_from_slice(&self.framebuffer[src_offset..src_offset + row_size]);
                            }
                        }
                        for row in 0..rect.height as usize {
                            let dst_y = rect.y as usize + row;
                            let dst_offset = dst_y * stride + rect.x as usize * bpp;
                            let temp_offset = row * rect.width as usize * bpp;
                            let row_size = rect.width as usize * bpp;
                            
                            if dst_offset + row_size <= self.framebuffer.len() {
                                self.framebuffer[dst_offset..dst_offset + row_size]
                                    .copy_from_slice(&temp[temp_offset..temp_offset + row_size]);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for RfbDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// RFB client message encoder
pub struct RfbEncoder;

impl RfbEncoder {
    /// Encode SetPixelFormat message
    pub fn set_pixel_format(format: &PixelFormat) -> Vec<u8> {
        let mut buf = vec![0u8; 20];
        buf[0] = 0; // message type
        // bytes 1-3 are padding
        buf[4..20].copy_from_slice(&format.to_bytes());
        buf
    }
    
    /// Encode SetEncodings message
    pub fn set_encodings(encodings: &[i32]) -> Vec<u8> {
        let mut buf = vec![0u8; 4 + encodings.len() * 4];
        buf[0] = 2; // message type
        // byte 1 is padding
        buf[2..4].copy_from_slice(&(encodings.len() as u16).to_be_bytes());
        for (i, &enc) in encodings.iter().enumerate() {
            let offset = 4 + i * 4;
            buf[offset..offset + 4].copy_from_slice(&enc.to_be_bytes());
        }
        buf
    }
    
    /// Encode FramebufferUpdateRequest message
    pub fn fb_update_request(incremental: bool, x: u16, y: u16, width: u16, height: u16) -> Vec<u8> {
        let mut buf = vec![0u8; 10];
        buf[0] = 3; // message type
        buf[1] = if incremental { 1 } else { 0 };
        buf[2..4].copy_from_slice(&x.to_be_bytes());
        buf[4..6].copy_from_slice(&y.to_be_bytes());
        buf[6..8].copy_from_slice(&width.to_be_bytes());
        buf[8..10].copy_from_slice(&height.to_be_bytes());
        buf
    }
    
    /// Encode KeyEvent message
    pub fn key_event(down: bool, key: u32) -> Vec<u8> {
        let mut buf = vec![0u8; 8];
        buf[0] = 4; // message type
        buf[1] = if down { 1 } else { 0 };
        // bytes 2-3 are padding
        buf[4..8].copy_from_slice(&key.to_be_bytes());
        buf
    }
    
    /// Encode PointerEvent message
    pub fn pointer_event(button_mask: u8, x: u16, y: u16) -> Vec<u8> {
        let mut buf = vec![0u8; 6];
        buf[0] = 5; // message type
        buf[1] = button_mask;
        buf[2..4].copy_from_slice(&x.to_be_bytes());
        buf[4..6].copy_from_slice(&y.to_be_bytes());
        buf
    }
}
