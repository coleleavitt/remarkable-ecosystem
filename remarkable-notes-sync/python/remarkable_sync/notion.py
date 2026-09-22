"""
Notion API integration
"""

import json
import base64
from typing import Optional, List, Dict, Any
from dataclasses import dataclass

import httpx

from .core import RmPage, export_svg


NOTION_API_URL = "https://api.notion.com/v1"
NOTION_VERSION = "2022-06-28"


class NotionClient:
    """Notion API client"""
    
    def __init__(self, token: str):
        self.token = token
        self.client = httpx.Client(
            headers={
                "Authorization": f"Bearer {token}",
                "Notion-Version": NOTION_VERSION,
                "Content-Type": "application/json",
            },
            timeout=30.0,
        )
    
    def get_page(self, page_id: str) -> Dict[str, Any]:
        """Get a page by ID"""
        response = self.client.get(f"{NOTION_API_URL}/pages/{page_id}")
        response.raise_for_status()
        return response.json()
    
    def create_page(
        self,
        parent_id: str,
        title: str,
        blocks: List[Dict[str, Any]] = None,
        is_database: bool = False,
    ) -> Dict[str, Any]:
        """Create a new page"""
        if is_database:
            parent = {"database_id": parent_id}
        else:
            parent = {"page_id": parent_id}
        
        data = {
            "parent": parent,
            "properties": {
                "title": {
                    "title": [{"type": "text", "text": {"content": title}}]
                }
            },
            "icon": {"emoji": "📝"},
        }
        
        if blocks:
            data["children"] = blocks
        
        response = self.client.post(f"{NOTION_API_URL}/pages", json=data)
        response.raise_for_status()
        return response.json()
    
    def update_page(self, page_id: str, properties: Dict[str, Any]) -> Dict[str, Any]:
        """Update page properties"""
        response = self.client.patch(
            f"{NOTION_API_URL}/pages/{page_id}",
            json={"properties": properties}
        )
        response.raise_for_status()
        return response.json()
    
    def archive_page(self, page_id: str) -> Dict[str, Any]:
        """Archive (delete) a page"""
        response = self.client.patch(
            f"{NOTION_API_URL}/pages/{page_id}",
            json={"archived": True}
        )
        response.raise_for_status()
        return response.json()
    
    def get_blocks(self, page_id: str) -> List[Dict[str, Any]]:
        """Get blocks from a page"""
        response = self.client.get(f"{NOTION_API_URL}/blocks/{page_id}/children")
        response.raise_for_status()
        return response.json().get("results", [])
    
    def append_blocks(self, page_id: str, blocks: List[Dict[str, Any]]) -> None:
        """Append blocks to a page"""
        response = self.client.patch(
            f"{NOTION_API_URL}/blocks/{page_id}/children",
            json={"children": blocks}
        )
        response.raise_for_status()
    
    def delete_block(self, block_id: str) -> None:
        """Delete a block"""
        response = self.client.delete(f"{NOTION_API_URL}/blocks/{block_id}")
        response.raise_for_status()
    
    def clear_page(self, page_id: str) -> None:
        """Clear all blocks from a page"""
        blocks = self.get_blocks(page_id)
        for block in blocks:
            self.delete_block(block["id"])
    
    def query_database(
        self,
        database_id: str,
        filter: Optional[Dict[str, Any]] = None,
        sorts: Optional[List[Dict[str, Any]]] = None,
    ) -> List[Dict[str, Any]]:
        """Query a database"""
        data = {}
        if filter:
            data["filter"] = filter
        if sorts:
            data["sorts"] = sorts
        
        response = self.client.post(
            f"{NOTION_API_URL}/databases/{database_id}/query",
            json=data
        )
        response.raise_for_status()
        return response.json().get("results", [])


class NotionSync:
    """Sync manager for Notion"""
    
    def __init__(self, client: NotionClient, state_path: Optional[str] = None):
        self.client = client
        self.state_path = state_path
        
        # Load state
        self.page_mappings: Dict[str, str] = {}  # remarkable_id -> notion_page_id
        self.last_synced: Dict[str, str] = {}  # remarkable_id -> timestamp
        
        if state_path:
            self._load_state()
    
    def _load_state(self) -> None:
        """Load sync state from disk"""
        try:
            with open(self.state_path) as f:
                state = json.load(f)
                self.page_mappings = state.get("page_mappings", {})
                self.last_synced = state.get("last_synced", {})
        except (FileNotFoundError, json.JSONDecodeError):
            pass
    
    def _save_state(self) -> None:
        """Save sync state to disk"""
        if not self.state_path:
            return
        
        state = {
            "page_mappings": self.page_mappings,
            "last_synced": self.last_synced,
        }
        with open(self.state_path, 'w') as f:
            json.dump(state, f, indent=2)
    
    def push_document(
        self,
        pages: List[RmPage],
        doc: Any,  # RemarkableDocument
        parent_id: str,
    ) -> str:
        """Push a document to Notion"""
        remarkable_id = doc.id
        title = getattr(doc, 'visible_name', '') or f"Untitled-{doc.id[:8]}"
        
        # Build blocks
        blocks = self._build_blocks(pages, doc)
        
        # Check if page exists
        if remarkable_id in self.page_mappings:
            notion_id = self.page_mappings[remarkable_id]
            self.client.clear_page(notion_id)
            self.client.append_blocks(notion_id, blocks)
        else:
            page = self.client.create_page(parent_id, title, blocks)
            notion_id = page["id"]
            self.page_mappings[remarkable_id] = notion_id
        
        from datetime import datetime
        self.last_synced[remarkable_id] = datetime.utcnow().isoformat()
        self._save_state()
        
        return notion_id
    
    def _build_blocks(self, pages: List[RmPage], doc: Any) -> List[Dict[str, Any]]:
        """Build Notion blocks from pages"""
        blocks = []
        
        # Header
        title = getattr(doc, 'visible_name', '') or f"Untitled-{doc.id[:8]}"
        blocks.append({
            "type": "heading_2",
            "heading_2": {
                "rich_text": [{"type": "text", "text": {"content": title}}]
            }
        })
        
        # Metadata
        blocks.append({
            "type": "paragraph",
            "paragraph": {
                "rich_text": [{"type": "text", "text": {"content": f"Source: reMarkable | ID: {doc.id[:8]}"}}]
            }
        })
        
        blocks.append({"type": "divider", "divider": {}})
        
        # Pages
        for i, page in enumerate(pages):
            if len(pages) > 1:
                blocks.append({
                    "type": "heading_3",
                    "heading_3": {
                        "rich_text": [{"type": "text", "text": {"content": f"Page {i+1}"}}]
                    }
                })
            
            # Add SVG as image (data URL)
            if page.strokes:
                svg = export_svg(page)
                svg_b64 = base64.b64encode(svg.encode()).decode()
                data_url = f"data:image/svg+xml;base64,{svg_b64}"
                
                blocks.append({
                    "type": "image",
                    "image": {
                        "type": "external",
                        "external": {"url": data_url}
                    }
                })
            
            # Text content
            for text in page.text_items:
                blocks.append({
                    "type": "paragraph",
                    "paragraph": {
                        "rich_text": [{"type": "text", "text": {"content": text.text}}]
                    }
                })
        
        # Footer
        blocks.append({"type": "divider", "divider": {}})
        blocks.append({
            "type": "paragraph",
            "paragraph": {
                "rich_text": [{"type": "text", "text": {"content": "Synced from reMarkable"}}]
            }
        })
        
        return blocks
    
    def delete_document(self, remarkable_id: str) -> bool:
        """Delete a document from Notion"""
        if remarkable_id not in self.page_mappings:
            return False
        
        notion_id = self.page_mappings.pop(remarkable_id)
        self.last_synced.pop(remarkable_id, None)
        
        self.client.archive_page(notion_id)
        self._save_state()
        
        return True
    
    def get_notion_id(self, remarkable_id: str) -> Optional[str]:
        """Get Notion page ID for a reMarkable document"""
        return self.page_mappings.get(remarkable_id)
