# remarkable-notes-sync

Bidirectional sync between reMarkable tablet, Obsidian vaults, and Notion workspaces.

## Features

- **Obsidian Sync**
  - Export .rm → Markdown + embedded/linked SVG
  - Import Markdown → .rm (text items)
  - Watch vault folder for changes
  - Preserve wikilinks

- **Notion Sync**
  - Export .rm → Notion page (via API)
  - Handwriting as inline images
  - Tag mapping: rm folders → Notion databases

- **Conflict Resolution**
  - Timestamp-based
  - Configurable strategies

## Installation

### From PyPI

```bash
pip install remarkable-sync
```

### From Source

```bash
git clone https://github.com/coleleavitt/remarkable-notes-sync.git
cd remarkable-notes-sync
pip install -e ".[dev]"
```

## CLI Usage

```bash
# Authenticate
remarkable-sync auth login --code XXXXXXXX

# Push to Obsidian
remarkable-sync obsidian push --vault ~/Obsidian/Notes

# Watch Obsidian vault
remarkable-sync obsidian watch --vault ~/Obsidian/Notes

# Push to Notion
remarkable-sync notion push --token $NOTION_TOKEN --parent PAGE_ID

# List documents
remarkable-sync list
```

## Python API

```python
from remarkable_sync import RemarkableClient, ObsidianVault, sync_to_obsidian

# Connect to reMarkable cloud
client = RemarkableClient.from_config()

# Sync to Obsidian
synced = sync_to_obsidian(client, "~/Obsidian/Notes")
print(f"Synced {synced} documents")

# Or use the vault directly
vault = ObsidianVault("~/Obsidian/Notes")
documents = client.list_documents()
for doc in documents:
    vault.sync_document(doc)
```

## Configuration

Create `~/.config/remarkable-sync/config.toml`:

```toml
[obsidian]
vault = "~/Obsidian/Notes"
remarkable_folder = "reMarkable"
embed_svg = false

[notion]
token = "secret_..."
parent_id = "..."

[sync]
strategy = "last_write_wins"  # or "remarkable_first", "local_first", "manual"
```

## Architecture

```
remarkable-notes-sync/
├── src/                    # Rust core
│   ├── rm_format/          # .rm file parser
│   ├── sync/               # reMarkable cloud client
│   ├── obsidian/           # Obsidian vault operations
│   ├── notion/             # Notion API client
│   └── conflict/           # Conflict resolution
└── python/                 # Python bindings
    └── remarkable_sync/
```

## License

MIT
