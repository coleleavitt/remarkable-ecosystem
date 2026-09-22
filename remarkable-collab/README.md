# remarkable-collab

Real-time collaboration server for reMarkable documents using CRDT-based conflict resolution.

## Features

- **WebSocket Real-time Sync**: Low-latency bidirectional communication
- **CRDT Conflict Resolution**: Lamport timestamps with Last-Writer-Wins (LWW) semantics
- **Multi-user Editing**: Multiple users can edit the same document simultaneously
- **Presence Indicators**: See other users' cursors and selections
- **Per-user Undo/Redo**: Each user has their own undo/redo stack
- **Web Client**: Built-in browser-based collaboration interface
- **remarkable-server Integration**: Works with local sync server

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     remarkable-collab                        │
├─────────────────────────────────────────────────────────────┤
│  WebSocket Server (Axum)                                     │
│  ├── Session Management (per document)                       │
│  ├── User Authentication                                     │
│  └── Message Routing                                         │
├─────────────────────────────────────────────────────────────┤
│  CRDT Engine                                                 │
│  ├── Lamport Clock (author:sequence IDs)                     │
│  ├── Operation Types (14 scene actions)                      │
│  ├── Document State (pages, layers, strokes)                 │
│  └── Conflict Resolution (LWW merge)                         │
├─────────────────────────────────────────────────────────────┤
│  Presence Tracking                                           │
│  ├── Cursor Positions                                        │
│  ├── Selection State                                         │
│  ├── Drawing State                                           │
│  └── User Colors                                             │
├─────────────────────────────────────────────────────────────┤
│  Protocol                                                    │
│  ├── ClientMessage (join, ops, cursor, undo/redo)            │
│  └── ServerMessage (ack, broadcast, presence, sync)          │
└─────────────────────────────────────────────────────────────┘
```

## CRDT Operations

Based on reverse engineering of the reMarkable xochitl binary, supports 14+ operations:

| Operation | Description |
|-----------|-------------|
| `AddItem` | Add a single stroke |
| `AddItems` | Add multiple strokes |
| `DeleteItems` | Delete strokes (tombstone) |
| `SwapItems` | Swap item z-order |
| `AddLayer` | Create new layer |
| `MoveLayer` | Reorder layer |
| `DeleteLayer` | Remove layer |
| `SetLayerName` | Rename layer |
| `SetLayerVisible` | Toggle visibility |
| `MergeLayerDown` | Merge with layer below |
| `CreateGroup` | Group items |
| `DeleteGroup` | Ungroup items |
| `EraseWithLine` | Eraser stroke |
| `TextInsert/Remove` | Text editing |

## Protocol

### Join Session
```json
{
  "type": "join_session",
  "document_id": "uuid",
  "user_name": "Alice"
}
```

### Send Operation
```json
{
  "type": "operation",
  "op": {
    "type": "add_item",
    "id": {"author": 1, "sequence": 42},
    "layer_id": "uuid",
    "stroke": { ... }
  }
}
```

### Receive Broadcast
```json
{
  "type": "operation",
  "op": { ... },
  "from_user": "uuid"
}
```

## Usage

### Start Server

```bash
cargo run -- --port 8090
```

### CLI Options

```
remarkable-collab [OPTIONS]

Options:
  -H, --host <HOST>      Host to bind [default: 0.0.0.0]
  -p, --port <PORT>      Port to bind [default: 8090]
      --web-dir <PATH>   Path to web client files
  -d, --debug            Enable debug logging
  -h, --help             Print help
  -V, --version          Print version
```

### Web Client

Open `http://localhost:8090` in your browser to access the built-in collaboration interface.

### Programmatic Usage

```rust
use remarkable_collab::{CollabServer, ServerConfig, ClientMessage};
use std::sync::Arc;

// Create server
let config = ServerConfig::default();
let server = Arc::new(CollabServer::new(config));

// Get/create session for a document
let session = server.get_or_create_session(document_id).await;

// Sessions handle CRDT operations automatically
```

## Integration with remarkable-server

To integrate with the local sync server:

1. Start remarkable-collab on port 8090
2. Configure remarkable-server to notify collab on document changes
3. Enable WebSocket proxy in remarkable-server for `/collab` endpoint

## Development

```bash
# Build
cargo build

# Test
cargo test

# Run with debug logging
cargo run -- --debug

# Format
cargo fmt

# Lint
cargo clippy
```

## License

MIT
