"""
remarkable_sync - Python bindings for reMarkable sync

This package provides Python bindings for the remarkable-notes-sync Rust library,
enabling bidirectional sync between reMarkable, Obsidian, and Notion.

Example usage:
    from remarkable_sync import RemarkableClient, ObsidianVault
    
    # Connect to reMarkable cloud
    client = RemarkableClient.from_device_token("...")
    documents = client.list_documents()
    
    # Sync to Obsidian
    vault = ObsidianVault("~/Obsidian/Notes")
    vault.sync_document(documents[0])
"""

from .core import (
    parse_rm_file,
    export_svg,
    export_markdown,
    RmPage,
    Stroke,
    Point,
)

from .sync import (
    RemarkableClient,
    sync_to_obsidian,
    sync_to_notion,
)

from .obsidian import (
    ObsidianVault,
    watch_vault,
)

from .notion import (
    NotionClient,
    NotionSync,
)

__version__ = "0.1.0"
__all__ = [
    # Core
    "parse_rm_file",
    "export_svg", 
    "export_markdown",
    "RmPage",
    "Stroke",
    "Point",
    # Sync
    "RemarkableClient",
    "sync_to_obsidian",
    "sync_to_notion",
    # Obsidian
    "ObsidianVault",
    "watch_vault",
    # Notion
    "NotionClient",
    "NotionSync",
]
