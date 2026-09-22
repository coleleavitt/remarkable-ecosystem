//! Collaboration protocol messages.
//!
//! Defines all messages exchanged between clients and server for:
//! - Session management (join, leave)
//! - Operation broadcast
//! - Presence updates
//! - Undo/redo

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crdt::{CrdtOp, LamportId};
use crate::presence::{CursorPosition, UserPresence};

/// Client-to-server messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Join a collaboration session.
    JoinSession {
        document_id: Uuid,
        user_name: String,
        /// Optional session password.
        password: Option<String>,
    },

    /// Leave the current session.
    LeaveSession,

    /// Apply a CRDT operation.
    Operation {
        op: CrdtOp,
    },

    /// Batch of operations (for efficiency).
    OperationBatch {
        ops: Vec<CrdtOp>,
    },

    /// Update cursor position.
    CursorUpdate {
        position: CursorPosition,
    },

    /// Update selection.
    SelectionUpdate {
        selected_items: Vec<Uuid>,
    },

    /// Start drawing (prevents others from erasing nearby).
    StartDrawing,

    /// Stop drawing.
    StopDrawing,

    /// Request undo.
    Undo,

    /// Request redo.
    Redo,

    /// Sync request (get operations since generation).
    SyncRequest {
        since_generation: u64,
    },

    /// Ping (keepalive).
    Ping {
        timestamp: u64,
    },

    /// Request full document state.
    RequestSnapshot,

    /// Set active tool.
    SetTool {
        tool: String,
    },
}

/// Server-to-client messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Session joined successfully.
    SessionJoined {
        session_id: Uuid,
        user_id: Uuid,
        author_id: u32,
        document_id: Uuid,
        generation: u64,
        /// Current users in the session.
        users: Vec<UserPresence>,
    },

    /// Session join failed.
    SessionError {
        error: String,
    },

    /// User joined the session.
    UserJoined {
        user: UserPresence,
    },

    /// User left the session.
    UserLeft {
        user_id: Uuid,
    },

    /// Operation from another user.
    Operation {
        op: CrdtOp,
        from_user: Uuid,
    },

    /// Batch of operations.
    OperationBatch {
        ops: Vec<CrdtOp>,
        from_user: Uuid,
    },

    /// Operation acknowledged (for the sending user).
    OperationAck {
        op_id: LamportId,
    },

    /// Presence update.
    PresenceUpdate {
        user_id: Uuid,
        cursor: Option<CursorPosition>,
        selection: Vec<Uuid>,
        is_drawing: bool,
        tool: Option<String>,
    },

    /// Undo result.
    UndoResult {
        success: bool,
        /// The operation that was undone (if any).
        undone_op: Option<CrdtOp>,
    },

    /// Redo result.
    RedoResult {
        success: bool,
        /// The operation that was redone (if any).
        redone_op: Option<CrdtOp>,
    },

    /// Sync response.
    SyncResponse {
        operations: Vec<CrdtOp>,
        current_generation: u64,
    },

    /// Full document snapshot.
    Snapshot {
        document_json: String,
        generation: u64,
    },

    /// Pong response.
    Pong {
        client_timestamp: u64,
        server_timestamp: u64,
    },

    /// Server error.
    Error {
        code: ErrorCode,
        message: String,
    },

    /// Server notification.
    Notification {
        level: NotificationLevel,
        message: String,
    },
}

/// Error codes for server errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Session not found.
    SessionNotFound,
    /// Not authorized.
    Unauthorized,
    /// Invalid operation.
    InvalidOperation,
    /// Document not found.
    DocumentNotFound,
    /// Rate limited.
    RateLimited,
    /// Internal server error.
    InternalError,
    /// Protocol error.
    ProtocolError,
}

/// Notification levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

impl ClientMessage {
    /// Parse from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl ServerMessage {
    /// Parse from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::{LamportId, PenType, Stroke, StrokeColor, StrokePoint};

    #[test]
    fn test_client_message_serialization() {
        let msg = ClientMessage::JoinSession {
            document_id: Uuid::new_v4(),
            user_name: "Alice".to_string(),
            password: None,
        };

        let json = msg.to_json();
        let parsed = ClientMessage::from_json(&json).unwrap();
        
        match (msg, parsed) {
            (
                ClientMessage::JoinSession { user_name: n1, .. },
                ClientMessage::JoinSession { user_name: n2, .. },
            ) => assert_eq!(n1, n2),
            _ => panic!("Mismatched message types"),
        }
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::Error {
            code: ErrorCode::SessionNotFound,
            message: "Session not found".to_string(),
        };

        let json = msg.to_json();
        let parsed = ServerMessage::from_json(&json).unwrap();
        
        match (msg, parsed) {
            (
                ServerMessage::Error { code: c1, .. },
                ServerMessage::Error { code: c2, .. },
            ) => assert_eq!(c1, c2),
            _ => panic!("Mismatched message types"),
        }
    }
}
