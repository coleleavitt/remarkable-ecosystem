# remarkable-diff

Visual CRDT merge/diff viewer for reMarkable documents.

## Features

- **Side-by-side stroke diff**: Compare two document versions with color-coded additions, deletions, and modifications
- **Timeline view**: Visualize all CRDT operations chronologically, filtered by author or category
- **Conflict resolution**: Manually resolve merge conflicts when automatic CRDT merging is ambiguous
- **Export merged**: Export the merged document for use with reMarkable

## Architecture

```
remarkable-diff/
├── crates/
│   ├── remarkable-diff-core/     # Core diff/merge engine
│   │   ├── types.rs              # Document, stroke, CRDT types
│   │   ├── diff.rs               # Diff computation
│   │   ├── merge.rs              # CRDT-based merging
│   │   └── timeline.rs           # Operation timeline
│   └── remarkable-diff-server/   # Axum web server
│       ├── main.rs               # Entry point
│       ├── handlers.rs           # API endpoints
│       ├── parsers.rs            # .rm file parsing
│       ├── state.rs              # App state
│       └── error.rs              # Error types
└── web/                          # React frontend
    ├── src/
    │   ├── App.tsx               # Main app component
    │   ├── lib/
    │   │   ├── api.ts            # API client
    │   │   └── types.ts          # TypeScript types
    │   └── components/
    │       ├── StrokeDiffView.tsx  # Side-by-side diff
    │       ├── TimelineView.tsx    # CRDT operation timeline
    │       ├── MergePanel.tsx      # Conflict resolution UI
    │       ├── DiffStats.tsx       # Diff statistics
    │       └── UploadPanel.tsx     # Document upload
    └── package.json
```

## CRDT Operations Visualized

| Operation | Category | Description |
|-----------|----------|-------------|
| AddItem | item | Add a single stroke/text |
| AddItems | item | Add multiple items |
| DeleteItems | item | Delete items (tombstone) |
| InsertItemsAfter | item | Insert after specific item |
| SwapItems | item | Swap item positions |
| CreateGroup | group | Create item group |
| DeleteGroup | group | Delete item group |
| AddLayer | layer | Add new layer |
| DeleteLayer | layer | Delete layer |
| MoveLayer | layer | Reorder layer |
| MergeLayerDown | layer | Merge with layer below |
| SetLayerName | layer | Rename layer |
| SetLayerVisible | layer | Toggle visibility |
| TextInsert | text | Insert text |
| TextRemove | text | Remove text |

## Usage

### Start the server

```bash
cd crates/remarkable-diff-server
cargo run -- --port 3001
```

### Start the frontend (development)

```bash
cd web
npm install
npm run dev
```

### Build for production

```bash
# Build frontend
cd web
npm run build

# Build server
cargo build --release -p remarkable-diff-server
```

## API Endpoints

### Documents

- `GET /api/documents` - List all documents
- `POST /api/documents` - Upload document (multipart)
- `GET /api/documents/:id` - Get document info
- `GET /api/documents/:id/versions` - List versions
- `GET /api/documents/:id/versions/:vid` - Get version

### Diff

- `POST /api/diff` - Compute diff between versions
- `GET /api/diff/:id` - Get cached diff

### Merge

- `POST /api/merge` - Merge two versions
- `POST /api/merge/:id/resolve` - Resolve conflict
- `GET /api/merge/:id/export` - Export merged document

### Timeline

- `GET /api/timeline/:doc_id` - Get operation timeline
- `POST /api/timeline/:doc_id/filter` - Filter timeline

## Merge Strategies

| Strategy | Description |
|----------|-------------|
| `last_writer_wins` | Use CRDT semantics (default) |
| `keep_all` | Keep all changes from both versions |
| `prefer_a` | Prefer version A on conflicts |
| `prefer_b` | Prefer version B on conflicts |
| `manual` | Require manual resolution |

## Development

Uses:
- Rust + Axum for the backend
- React 19 + TanStack Query for the frontend
- remarkable-core for .rm file parsing
- Tailwind CSS for styling

## License

MIT
