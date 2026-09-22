//! WebSocket server for real-time collaboration.
//!
//! Handles WebSocket connections, session management, and message routing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::crdt::{CrdtDocument, CrdtOp, LamportClock, UndoStack, UndoableOp};
use crate::presence::{PresenceTracker, UserPresence};
use crate::protocol::{ClientMessage, ErrorCode, ServerMessage};

/// Configuration for the collaboration server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Presence timeout (how long until a user is considered offline).
    pub presence_timeout: Duration,
    /// Heartbeat interval.
    pub heartbeat_interval: Duration,
    /// Maximum operations per second per user.
    pub rate_limit: u32,
    /// Maximum undo stack size per user.
    pub max_undo_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            presence_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(15),
            rate_limit: 100,
            max_undo_size: 100,
        }
    }
}

/// A connected user in a session.
struct ConnectedUser {
    user_id: Uuid,
    author_id: u32,
    presence: UserPresence,
    clock: LamportClock,
    undo_stack: UndoStack,
    tx: mpsc::Sender<ServerMessage>,
}

/// A collaboration session for a document.
pub struct Session {
    id: Uuid,
    document_id: Uuid,
    document: CrdtDocument,
    users: HashMap<Uuid, ConnectedUser>,
    next_author_id: u32,
    presence: PresenceTracker,
    /// Broadcast channel for operations.
    op_broadcast: broadcast::Sender<ServerMessage>,
}

impl Session {
    pub fn new(document_id: Uuid, document: CrdtDocument) -> Self {
        let (op_broadcast, _) = broadcast::channel(1024);
        
        Self {
            id: Uuid::new_v4(),
            document_id,
            document,
            users: HashMap::new(),
            next_author_id: 1,
            presence: PresenceTracker::new(Duration::from_secs(30)),
            op_broadcast,
        }
    }

    /// Add a user to the session.
    pub fn add_user(&mut self, user_name: String, tx: mpsc::Sender<ServerMessage>) -> (Uuid, u32) {
        let user_id = Uuid::new_v4();
        let author_id = self.next_author_id;
        self.next_author_id += 1;

        let presence = UserPresence::new(user_id, user_name);
        self.presence.upsert(presence.clone());

        let user = ConnectedUser {
            user_id,
            author_id,
            presence,
            clock: LamportClock::new(author_id),
            undo_stack: UndoStack::new(100),
            tx,
        };

        self.users.insert(user_id, user);
        (user_id, author_id)
    }

    /// Remove a user from the session.
    pub fn remove_user(&mut self, user_id: &Uuid) {
        self.users.remove(user_id);
        self.presence.remove(user_id);
    }

    /// Get the current users.
    pub fn get_users(&self) -> Vec<UserPresence> {
        self.presence.get_presence_snapshot()
    }

    /// Apply an operation from a user.
    pub fn apply_operation(&mut self, user_id: &Uuid, op: CrdtOp) -> bool {
        if let Some(user) = self.users.get_mut(user_id) {
            // Update user's clock
            user.clock.update(op.lamport_id());

            // Create undoable operation
            let undoable = UndoableOp::new(op.clone(), *user_id);
            user.undo_stack.push(undoable);

            // Apply to document
            self.document.apply(op)
        } else {
            false
        }
    }

    /// Broadcast an operation to all users except the sender.
    pub async fn broadcast_operation(&self, op: CrdtOp, from_user: Uuid) {
        let msg = ServerMessage::Operation { op, from_user };
        let _ = self.op_broadcast.send(msg);
    }

    /// Get user's next Lamport ID.
    pub fn next_lamport_id(&mut self, user_id: &Uuid) -> Option<crate::crdt::LamportId> {
        self.users.get_mut(user_id).map(|u| u.clock.tick())
    }

    /// Undo for a user.
    pub fn undo(&mut self, user_id: &Uuid) -> Option<CrdtOp> {
        if let Some(user) = self.users.get_mut(user_id) {
            if let Some(undoable) = user.undo_stack.undo() {
                if let Some(inverse) = undoable.inverse {
                    self.document.apply(inverse.clone());
                    return Some(inverse);
                }
            }
        }
        None
    }

    /// Redo for a user.
    pub fn redo(&mut self, user_id: &Uuid) -> Option<CrdtOp> {
        if let Some(user) = self.users.get_mut(user_id) {
            if let Some(undoable) = user.undo_stack.redo() {
                self.document.apply(undoable.op.clone());
                return Some(undoable.op);
            }
        }
        None
    }

    /// Get broadcast receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
        self.op_broadcast.subscribe()
    }
}

/// The collaboration server managing all sessions.
pub struct CollabServer {
    sessions: RwLock<HashMap<Uuid, Arc<RwLock<Session>>>>,
    config: ServerConfig,
}

impl CollabServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Create or get a session for a document.
    pub async fn get_or_create_session(&self, document_id: Uuid) -> Arc<RwLock<Session>> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(&document_id) {
            return session.clone();
        }
        drop(sessions);

        // Create new session
        let document = CrdtDocument::new(document_id, "Untitled".to_string());
        let session = Session::new(document_id, document);
        let session = Arc::new(RwLock::new(session));

        let mut sessions = self.sessions.write().await;
        sessions.insert(document_id, session.clone());
        session
    }

    /// Remove a session.
    pub async fn remove_session(&self, document_id: &Uuid) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(document_id);
    }

    /// Handle a WebSocket connection.
    pub async fn handle_connection(self: Arc<Self>, ws: WebSocket) {
        let (mut ws_tx, mut ws_rx) = ws.split();

        // Channel for sending messages to the WebSocket
        let (tx, mut rx) = mpsc::channel::<ServerMessage>(256);

        // Spawn task to forward messages to WebSocket
        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let json = msg.to_json();
                if ws_tx.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        });

        let mut session: Option<Arc<RwLock<Session>>> = None;
        let mut user_id: Option<Uuid> = None;
        let mut broadcast_rx: Option<broadcast::Receiver<ServerMessage>> = None;

        // Main message loop
        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                Some(msg_result) = ws_rx.next() => {
                    match msg_result {
                        Ok(Message::Text(text)) => {
                            match ClientMessage::from_json(&text) {
                                Ok(client_msg) => {
                                    self.handle_message(
                                        client_msg,
                                        &mut session,
                                        &mut user_id,
                                        &mut broadcast_rx,
                                        &tx,
                                    ).await;
                                }
                                Err(e) => {
                                    let _ = tx.send(ServerMessage::Error {
                                        code: ErrorCode::ProtocolError,
                                        message: format!("Invalid message: {}", e),
                                    }).await;
                                }
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }

                // Handle broadcast messages
                Some(broadcast_msg) = async {
                    if let Some(ref mut rx) = broadcast_rx {
                        rx.recv().await.ok()
                    } else {
                        None
                    }
                } => {
                    // Forward broadcast to this user (if not from themselves)
                    if let ServerMessage::Operation { from_user, .. } = &broadcast_msg {
                        if Some(*from_user) != user_id {
                            let _ = tx.send(broadcast_msg).await;
                        }
                    } else {
                        let _ = tx.send(broadcast_msg).await;
                    }
                }

                else => break,
            }
        }

        // Cleanup on disconnect
        if let (Some(session), Some(uid)) = (session, user_id) {
            let mut session = session.write().await;
            session.remove_user(&uid);

            // Notify other users
            let msg = ServerMessage::UserLeft { user_id: uid };
            let _ = session.op_broadcast.send(msg);

            info!("User {} disconnected from session {}", uid, session.id);
        }

        send_task.abort();
    }

    /// Handle a client message.
    async fn handle_message(
        &self,
        msg: ClientMessage,
        session: &mut Option<Arc<RwLock<Session>>>,
        user_id: &mut Option<Uuid>,
        broadcast_rx: &mut Option<broadcast::Receiver<ServerMessage>>,
        tx: &mpsc::Sender<ServerMessage>,
    ) {
        match msg {
            ClientMessage::JoinSession {
                document_id,
                user_name,
                password: _,
            } => {
                let sess = self.get_or_create_session(document_id).await;
                let mut sess_guard = sess.write().await;

                let (uid, author_id) = sess_guard.add_user(user_name, tx.clone());
                *user_id = Some(uid);
                *broadcast_rx = Some(sess_guard.subscribe());

                let users = sess_guard.get_users();
                let generation = sess_guard.document.generation;

                // Notify existing users
                let join_msg = ServerMessage::UserJoined {
                    user: sess_guard.presence.get(&uid).unwrap().clone(),
                };
                let _ = sess_guard.op_broadcast.send(join_msg);

                drop(sess_guard);
                *session = Some(sess);

                // Send join confirmation
                let _ = tx
                    .send(ServerMessage::SessionJoined {
                        session_id: Uuid::new_v4(),
                        user_id: uid,
                        author_id,
                        document_id,
                        generation,
                        users,
                    })
                    .await;

                info!("User {} joined document {}", uid, document_id);
            }

            ClientMessage::LeaveSession => {
                if let (Some(sess), Some(uid)) = (session.take(), user_id.take()) {
                    let mut sess_guard = sess.write().await;
                    sess_guard.remove_user(&uid);

                    let msg = ServerMessage::UserLeft { user_id: uid };
                    let _ = sess_guard.op_broadcast.send(msg);
                }
            }

            ClientMessage::Operation { op } => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;
                    
                    if sess_guard.apply_operation(uid, op.clone()) {
                        // Acknowledge to sender
                        let _ = tx
                            .send(ServerMessage::OperationAck {
                                op_id: op.lamport_id(),
                            })
                            .await;

                        // Broadcast to others
                        let msg = ServerMessage::Operation {
                            op,
                            from_user: *uid,
                        };
                        let _ = sess_guard.op_broadcast.send(msg);
                    }
                }
            }

            ClientMessage::OperationBatch { ops } => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;

                    for op in &ops {
                        sess_guard.apply_operation(uid, op.clone());
                    }

                    // Broadcast batch
                    let msg = ServerMessage::OperationBatch {
                        ops,
                        from_user: *uid,
                    };
                    let _ = sess_guard.op_broadcast.send(msg);
                }
            }

            ClientMessage::CursorUpdate { position } => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;
                    sess_guard.presence.update_cursor(*uid, position);

                    let presence = sess_guard.presence.get(uid).cloned();
                    if let Some(p) = presence {
                        let msg = ServerMessage::PresenceUpdate {
                            user_id: *uid,
                            cursor: p.cursor,
                            selection: p.selection,
                            is_drawing: p.is_drawing,
                            tool: p.active_tool,
                        };
                        let _ = sess_guard.op_broadcast.send(msg);
                    }
                }
            }

            ClientMessage::SelectionUpdate { selected_items } => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;
                    sess_guard.presence.update_selection(*uid, selected_items);
                }
            }

            ClientMessage::StartDrawing => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;
                    sess_guard.presence.set_drawing(*uid, true);
                }
            }

            ClientMessage::StopDrawing => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;
                    sess_guard.presence.set_drawing(*uid, false);
                }
            }

            ClientMessage::Undo => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;
                    let undone = sess_guard.undo(uid);

                    let _ = tx
                        .send(ServerMessage::UndoResult {
                            success: undone.is_some(),
                            undone_op: undone.clone(),
                        })
                        .await;

                    // Broadcast the undo operation
                    if let Some(op) = undone {
                        let msg = ServerMessage::Operation {
                            op,
                            from_user: *uid,
                        };
                        let _ = sess_guard.op_broadcast.send(msg);
                    }
                }
            }

            ClientMessage::Redo => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;
                    let redone = sess_guard.redo(uid);

                    let _ = tx
                        .send(ServerMessage::RedoResult {
                            success: redone.is_some(),
                            redone_op: redone.clone(),
                        })
                        .await;

                    if let Some(op) = redone {
                        let msg = ServerMessage::Operation {
                            op,
                            from_user: *uid,
                        };
                        let _ = sess_guard.op_broadcast.send(msg);
                    }
                }
            }

            ClientMessage::SyncRequest { since_generation } => {
                if let Some(sess) = session.as_ref() {
                    let sess_guard = sess.read().await;
                    let ops: Vec<CrdtOp> = sess_guard
                        .document
                        .ops_since(since_generation)
                        .into_iter()
                        .cloned()
                        .collect();

                    let _ = tx
                        .send(ServerMessage::SyncResponse {
                            operations: ops,
                            current_generation: sess_guard.document.generation,
                        })
                        .await;
                }
            }

            ClientMessage::RequestSnapshot => {
                if let Some(sess) = session.as_ref() {
                    let sess_guard = sess.read().await;
                    let json = serde_json::to_string(&sess_guard.document).unwrap_or_default();

                    let _ = tx
                        .send(ServerMessage::Snapshot {
                            document_json: json,
                            generation: sess_guard.document.generation,
                        })
                        .await;
                }
            }

            ClientMessage::Ping { timestamp } => {
                let _ = tx
                    .send(ServerMessage::Pong {
                        client_timestamp: timestamp,
                        server_timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    })
                    .await;
            }

            ClientMessage::SetTool { tool } => {
                if let (Some(sess), Some(uid)) = (session.as_ref(), user_id.as_ref()) {
                    let mut sess_guard = sess.write().await;
                    if let Some(presence) = sess_guard.presence.get(uid).cloned() {
                        let mut updated = presence;
                        updated.active_tool = Some(tool);
                        sess_guard.presence.upsert(updated);
                    }
                }
            }
        }
    }
}
