//! remarkable-collab: Real-time collaboration server for reMarkable documents.
//!
//! Features:
//! - WebSocket-based real-time sync
//! - CRDT conflict resolution (Lamport timestamps, LWW)
//! - Multiple users editing same document
//! - Cursor/presence indicators
//! - Per-user undo/redo
//! - Web client for browser participation
//! - Integration with remarkable-server

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{WebSocketUpgrade, WebSocket},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use clap::Parser;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod crdt;
mod integration;
mod presence;
mod protocol;
mod ws;

use integration::{RemarkableServerClient, ServerIntegrationConfig};
use ws::{CollabServer, ServerConfig};

/// Command-line arguments.
#[derive(Parser, Debug)]
#[command(name = "remarkable-collab")]
#[command(about = "Real-time collaboration server for reMarkable documents")]
#[command(version)]
struct Args {
    /// Host to bind to.
    #[arg(short = 'H', long, default_value = "0.0.0.0", env = "COLLAB_HOST")]
    host: String,

    /// Port to bind to.
    #[arg(short, long, default_value_t = 8090, env = "COLLAB_PORT")]
    port: u16,

    /// Path to web client files (optional).
    #[arg(long, env = "COLLAB_WEB_DIR")]
    web_dir: Option<String>,

    /// Enable debug logging.
    #[arg(short, long)]
    debug: bool,

    /// remarkable-server URL (for integration).
    #[arg(long, env = "REMARKABLE_SERVER_URL")]
    remarkable_server: Option<String>,

    /// Path to reMarkable data directory.
    #[arg(long, env = "REMARKABLE_DATA_DIR")]
    data_dir: Option<PathBuf>,
}

/// Application state shared across handlers.
struct AppState {
    server: Arc<CollabServer>,
    #[allow(dead_code)]
    remarkable_client: Option<RemarkableServerClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.debug { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Create collaboration server
    let config = ServerConfig::default();
    let server = Arc::new(CollabServer::new(config));

    // Create remarkable-server client if configured
    let remarkable_client = if args.remarkable_server.is_some() || args.data_dir.is_some() {
        let integration_config = ServerIntegrationConfig {
            server_url: args.remarkable_server.unwrap_or_else(|| "http://localhost:8080".to_string()),
            data_dir: args.data_dir.unwrap_or_else(|| PathBuf::from("/home/root/.local/share/remarkable/xochitl")),
            mqtt_url: None,
            auth_token: None,
        };
        Some(RemarkableServerClient::new(integration_config))
    } else {
        None
    };

    let state = Arc::new(AppState {
        server,
        remarkable_client,
    });

    // Build router
    let mut app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .route("/api/documents", get(list_documents_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Optionally serve web client files
    if let Some(ref web_dir) = args.web_dir {
        app = app.nest_service("/static", ServeDir::new(web_dir));
    }

    // Start server
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    info!("Starting remarkable-collab server on {}", addr);
    info!("WebSocket endpoint: ws://{}/ws", addr);
    info!("Web client: http://{}/", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Index page handler.
async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// WebSocket upgrade handler.
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    state.server.clone().handle_connection(socket).await;
}

/// Health check handler.
async fn health_handler() -> &'static str {
    "OK"
}

/// List documents from remarkable-server.
async fn list_documents_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(ref client) = state.remarkable_client {
        match client.list_documents().await {
            Ok(docs) => axum::Json(docs).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error: {}", e),
            ).into_response(),
        }
    } else {
        axum::Json::<Vec<integration::DocumentInfo>>(vec![]).into_response()
    }
}

/// Embedded index page HTML.
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>remarkable-collab</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
        }
        header {
            background: #16213e;
            padding: 1rem 2rem;
            border-bottom: 1px solid #0f3460;
        }
        header h1 {
            font-size: 1.5rem;
            font-weight: 600;
        }
        main {
            flex: 1;
            display: flex;
            flex-direction: column;
            padding: 2rem;
            max-width: 1200px;
            margin: 0 auto;
            width: 100%;
        }
        .status {
            background: #16213e;
            border-radius: 8px;
            padding: 1.5rem;
            margin-bottom: 2rem;
        }
        .status h2 {
            font-size: 1rem;
            color: #888;
            margin-bottom: 1rem;
        }
        .status-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
        }
        .status-item {
            background: #1a1a2e;
            padding: 1rem;
            border-radius: 6px;
        }
        .status-item .label {
            font-size: 0.85rem;
            color: #888;
            margin-bottom: 0.5rem;
        }
        .status-item .value {
            font-size: 1.5rem;
            font-weight: 600;
        }
        .status-item .value.connected {
            color: #4ade80;
        }
        .status-item .value.disconnected {
            color: #f87171;
        }
        .join-form {
            background: #16213e;
            border-radius: 8px;
            padding: 1.5rem;
            margin-bottom: 2rem;
        }
        .join-form h2 {
            margin-bottom: 1rem;
        }
        .form-row {
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
        }
        .form-row input {
            flex: 1;
            min-width: 200px;
            padding: 0.75rem 1rem;
            border: 1px solid #0f3460;
            border-radius: 6px;
            background: #1a1a2e;
            color: #eee;
            font-size: 1rem;
        }
        .form-row input:focus {
            outline: none;
            border-color: #3b82f6;
        }
        .form-row button {
            padding: 0.75rem 2rem;
            background: #3b82f6;
            color: white;
            border: none;
            border-radius: 6px;
            font-size: 1rem;
            cursor: pointer;
            transition: background 0.2s;
        }
        .form-row button:hover {
            background: #2563eb;
        }
        .form-row button:disabled {
            background: #4b5563;
            cursor: not-allowed;
        }
        .canvas-container {
            flex: 1;
            background: #16213e;
            border-radius: 8px;
            position: relative;
            min-height: 400px;
            overflow: hidden;
        }
        #canvas {
            width: 100%;
            height: 100%;
            cursor: crosshair;
        }
        .users-list {
            position: absolute;
            top: 1rem;
            right: 1rem;
            background: rgba(26, 26, 46, 0.9);
            padding: 0.75rem;
            border-radius: 6px;
            font-size: 0.85rem;
        }
        .user-badge {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.25rem 0;
        }
        .user-color {
            width: 12px;
            height: 12px;
            border-radius: 50%;
        }
        .log {
            background: #16213e;
            border-radius: 8px;
            padding: 1rem;
            margin-top: 1rem;
            max-height: 200px;
            overflow-y: auto;
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 0.85rem;
        }
        .log-entry {
            padding: 0.25rem 0;
            border-bottom: 1px solid #1a1a2e;
        }
        .log-entry.info { color: #60a5fa; }
        .log-entry.success { color: #4ade80; }
        .log-entry.error { color: #f87171; }
        .log-entry.op { color: #fbbf24; }
    </style>
</head>
<body>
    <header>
        <h1>🖊️ remarkable-collab</h1>
    </header>
    <main>
        <div class="status">
            <h2>Server Status</h2>
            <div class="status-grid">
                <div class="status-item">
                    <div class="label">Connection</div>
                    <div class="value" id="conn-status">Disconnected</div>
                </div>
                <div class="status-item">
                    <div class="label">Session</div>
                    <div class="value" id="session-status">-</div>
                </div>
                <div class="status-item">
                    <div class="label">Users Online</div>
                    <div class="value" id="user-count">0</div>
                </div>
                <div class="status-item">
                    <div class="label">Operations</div>
                    <div class="value" id="op-count">0</div>
                </div>
            </div>
        </div>

        <div class="join-form">
            <h2>Join Session</h2>
            <div class="form-row">
                <input type="text" id="doc-id" placeholder="Document ID (UUID or any string)">
                <input type="text" id="user-name" placeholder="Your Name">
                <button id="join-btn" onclick="joinSession()">Join</button>
            </div>
        </div>

        <div class="canvas-container">
            <canvas id="canvas"></canvas>
            <div class="users-list" id="users-list"></div>
        </div>

        <div class="log" id="log"></div>
    </main>

    <script>
        let ws = null;
        let userId = null;
        let authorId = null;
        let isDrawing = false;
        let currentStroke = [];
        let strokes = [];
        let cursors = {};
        let users = {};
        let opCount = 0;

        const canvas = document.getElementById('canvas');
        const ctx = canvas.getContext('2d');

        function resizeCanvas() {
            const container = canvas.parentElement;
            canvas.width = container.clientWidth;
            canvas.height = container.clientHeight;
            redraw();
        }
        window.addEventListener('resize', resizeCanvas);
        resizeCanvas();

        function log(msg, type = 'info') {
            const logEl = document.getElementById('log');
            const entry = document.createElement('div');
            entry.className = `log-entry ${type}`;
            entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
            logEl.insertBefore(entry, logEl.firstChild);
            if (logEl.children.length > 100) {
                logEl.removeChild(logEl.lastChild);
            }
        }

        function updateStatus(connected) {
            const el = document.getElementById('conn-status');
            el.textContent = connected ? 'Connected' : 'Disconnected';
            el.className = `value ${connected ? 'connected' : 'disconnected'}`;
        }

        function updateUserList() {
            const el = document.getElementById('users-list');
            el.innerHTML = Object.values(users).map(u => `
                <div class="user-badge">
                    <div class="user-color" style="background: ${u.color}"></div>
                    <span>${u.name}${u.id === userId ? ' (you)' : ''}</span>
                </div>
            `).join('');
            document.getElementById('user-count').textContent = Object.keys(users).length;
        }

        function joinSession() {
            const docId = document.getElementById('doc-id').value || crypto.randomUUID();
            const userName = document.getElementById('user-name').value || 'Anonymous';

            document.getElementById('join-btn').disabled = true;

            const wsUrl = `ws://${location.host}/ws`;
            ws = new WebSocket(wsUrl);

            ws.onopen = () => {
                log('WebSocket connected', 'success');
                updateStatus(true);
                
                ws.send(JSON.stringify({
                    type: 'join_session',
                    document_id: docId,
                    user_name: userName
                }));
            };

            ws.onmessage = (e) => {
                const msg = JSON.parse(e.data);
                handleMessage(msg);
            };

            ws.onclose = () => {
                log('WebSocket disconnected', 'error');
                updateStatus(false);
                document.getElementById('join-btn').disabled = false;
            };

            ws.onerror = (e) => {
                log('WebSocket error', 'error');
            };
        }

        function handleMessage(msg) {
            switch (msg.type) {
                case 'session_joined':
                    userId = msg.user_id;
                    authorId = msg.author_id;
                    log(`Joined session as author ${authorId}`, 'success');
                    document.getElementById('session-status').textContent = msg.document_id.slice(0, 8);
                    msg.users.forEach(u => {
                        users[u.user_id] = { id: u.user_id, name: u.display_name, color: u.color ? `rgb(${u.color.r},${u.color.g},${u.color.b})` : '#3b82f6' };
                    });
                    updateUserList();
                    break;

                case 'user_joined':
                    users[msg.user.user_id] = { 
                        id: msg.user.user_id, 
                        name: msg.user.display_name, 
                        color: msg.user.color ? `rgb(${msg.user.color.r},${msg.user.color.g},${msg.user.color.b})` : '#3b82f6'
                    };
                    log(`${msg.user.display_name} joined`, 'info');
                    updateUserList();
                    break;

                case 'user_left':
                    const leftUser = users[msg.user_id];
                    if (leftUser) {
                        log(`${leftUser.name} left`, 'info');
                        delete users[msg.user_id];
                    }
                    updateUserList();
                    break;

                case 'operation':
                case 'operation_batch':
                    const ops = msg.ops || [msg.op];
                    ops.forEach(op => {
                        if (op.type === 'add_item' && op.stroke) {
                            strokes.push({
                                ...op.stroke,
                                color: users[msg.from_user]?.color || '#fff'
                            });
                            opCount++;
                            redraw();
                        }
                    });
                    document.getElementById('op-count').textContent = opCount;
                    break;

                case 'operation_ack':
                    log(`Op ${msg.op_id.author}:${msg.op_id.sequence} acknowledged`, 'op');
                    break;

                case 'presence_update':
                    if (msg.user_id !== userId) {
                        cursors[msg.user_id] = msg.cursor;
                        redraw();
                    }
                    break;

                case 'error':
                    log(`Error: ${msg.message}`, 'error');
                    break;
            }
        }

        function redraw() {
            ctx.fillStyle = '#fff';
            ctx.fillRect(0, 0, canvas.width, canvas.height);

            // Draw existing strokes
            strokes.forEach(stroke => {
                if (!stroke.points || stroke.points.length < 2) return;
                ctx.beginPath();
                ctx.strokeStyle = stroke.color || '#000';
                ctx.lineWidth = stroke.base_width || 2;
                ctx.lineCap = 'round';
                ctx.lineJoin = 'round';
                ctx.moveTo(stroke.points[0].x, stroke.points[0].y);
                stroke.points.slice(1).forEach(p => ctx.lineTo(p.x, p.y));
                ctx.stroke();
            });

            // Draw current stroke
            if (currentStroke.length > 1) {
                ctx.beginPath();
                ctx.strokeStyle = '#3b82f6';
                ctx.lineWidth = 2;
                ctx.moveTo(currentStroke[0].x, currentStroke[0].y);
                currentStroke.slice(1).forEach(p => ctx.lineTo(p.x, p.y));
                ctx.stroke();
            }

            // Draw other users' cursors
            Object.entries(cursors).forEach(([uid, cursor]) => {
                if (!cursor || uid === userId) return;
                const user = users[uid];
                ctx.beginPath();
                ctx.fillStyle = user?.color || '#888';
                ctx.arc(cursor.x, cursor.y, 5, 0, Math.PI * 2);
                ctx.fill();
            });
        }

        canvas.addEventListener('mousedown', (e) => {
            if (!ws || ws.readyState !== WebSocket.OPEN) return;
            isDrawing = true;
            currentStroke = [{ x: e.offsetX, y: e.offsetY, pressure: 1, width: 2, speed: 0, direction: 0 }];
            ws.send(JSON.stringify({ type: 'start_drawing' }));
        });

        canvas.addEventListener('mousemove', (e) => {
            if (!ws || ws.readyState !== WebSocket.OPEN) return;

            // Send cursor update
            ws.send(JSON.stringify({
                type: 'cursor_update',
                position: { x: e.offsetX, y: e.offsetY, page_id: '00000000-0000-0000-0000-000000000000' }
            }));

            if (isDrawing) {
                currentStroke.push({ x: e.offsetX, y: e.offsetY, pressure: 1, width: 2, speed: 0, direction: 0 });
                redraw();
            }
        });

        canvas.addEventListener('mouseup', (e) => {
            if (!ws || ws.readyState !== WebSocket.OPEN || !isDrawing) return;
            isDrawing = false;

            if (currentStroke.length > 1) {
                const stroke = {
                    id: crypto.randomUUID(),
                    pen_type: 'fineliner',
                    color: 'black',
                    base_width: 2,
                    points: currentStroke
                };

                const op = {
                    type: 'add_item',
                    id: { author: authorId, sequence: opCount++ },
                    layer_id: '00000000-0000-0000-0000-000000000000',
                    stroke: stroke
                };

                ws.send(JSON.stringify({ type: 'operation', op }));
                strokes.push({ ...stroke, color: '#3b82f6' });
                document.getElementById('op-count').textContent = opCount;
            }

            currentStroke = [];
            ws.send(JSON.stringify({ type: 'stop_drawing' }));
            redraw();
        });

        canvas.addEventListener('mouseleave', (e) => {
            if (isDrawing) {
                canvas.dispatchEvent(new MouseEvent('mouseup', e));
            }
        });

        // Generate a default doc ID
        document.getElementById('doc-id').value = crypto.randomUUID();
    </script>
</body>
</html>
"#;
