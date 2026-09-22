"""
Sync functionality for reMarkable cloud
"""

import os
import json
import httpx
from dataclasses import dataclass
from typing import Optional, List, Dict, Any
from datetime import datetime
from pathlib import Path


@dataclass
class RemarkableDocument:
    """Document metadata from reMarkable"""
    id: str
    hash: str
    doc_type: str
    visible_name: str
    parent: str
    version: int
    modified: Optional[datetime] = None


class RemarkableClient:
    """Client for reMarkable cloud API"""
    
    DISCOVERY_URL = "https://internal.cloud.remarkable.com/discovery/v1/endpoints"
    AUTH_URL = "https://webapp-prod.cloud.remarkable.engineering/token/json/2"
    
    def __init__(self, device_token: str):
        self.device_token = device_token
        self.user_token: Optional[str] = None
        self.tectonic_url: Optional[str] = None
        self.client = httpx.Client(timeout=30.0)
    
    @classmethod
    def from_device_token(cls, token: str) -> "RemarkableClient":
        """Create client from device token"""
        return cls(token)
    
    @classmethod
    def from_config(cls, config_path: str = "~/.config/remarkable-sync/tokens.json") -> "RemarkableClient":
        """Load client from config file"""
        path = Path(config_path).expanduser()
        if not path.exists():
            raise FileNotFoundError(f"Config not found: {path}")
        
        with open(path) as f:
            config = json.load(f)
        
        return cls(config["device_token"])
    
    def refresh_token(self) -> None:
        """Refresh user token"""
        url = f"{self.AUTH_URL}/user/new"
        
        response = self.client.post(
            url,
            headers={"Authorization": f"Bearer {self.device_token}"}
        )
        response.raise_for_status()
        
        self.user_token = response.text
        
        # Extract tectonic region from JWT
        import base64
        parts = self.user_token.split('.')
        if len(parts) == 3:
            payload = base64.urlsafe_b64decode(parts[1] + '==')
            data = json.loads(payload)
            region = data.get('tectonic', data.get('region', 'eu'))
            self.tectonic_url = f"https://{region}.tectonic.remarkable.com"
    
    def _ensure_auth(self) -> None:
        """Ensure we have a valid user token"""
        if not self.user_token:
            self.refresh_token()
    
    def get_root(self) -> Dict[str, Any]:
        """Get sync root state"""
        self._ensure_auth()
        
        url = f"{self.tectonic_url}/sync/v3/root"
        response = self.client.get(
            url,
            headers={"Authorization": f"Bearer {self.user_token}"}
        )
        response.raise_for_status()
        
        return response.json()
    
    def download_file(self, hash: str, filename: str) -> bytes:
        """Download a file by hash"""
        self._ensure_auth()
        
        url = f"{self.tectonic_url}/sync/v3/files/{hash}"
        response = self.client.get(
            url,
            headers={
                "Authorization": f"Bearer {self.user_token}",
                "rm-filename": filename,
            }
        )
        response.raise_for_status()
        
        return response.content
    
    def list_documents(self) -> List[RemarkableDocument]:
        """List all documents"""
        root = self.get_root()
        root_blob = self.download_file(root["hash"], "root")
        
        documents = []
        for line in root_blob.decode().split('\n')[1:]:
            parts = line.split(':')
            if len(parts) >= 5:
                documents.append(RemarkableDocument(
                    id=parts[2],
                    hash=parts[0],
                    doc_type=parts[1],
                    visible_name="",
                    parent="",
                    version=int(parts[3]) if parts[3].isdigit() else 0,
                ))
        
        return documents


def sync_to_obsidian(
    client: RemarkableClient,
    vault_path: str,
    document_id: Optional[str] = None,
) -> int:
    """
    Sync documents to Obsidian vault.
    
    Args:
        client: RemarkableClient instance
        vault_path: Path to Obsidian vault
        document_id: Optional specific document to sync
        
    Returns:
        Number of documents synced
    """
    from .core import parse_rm_file, export_markdown
    from .obsidian import ObsidianVault
    
    vault = ObsidianVault(vault_path)
    documents = client.list_documents()
    
    if document_id:
        documents = [d for d in documents if d.id == document_id]
    
    synced = 0
    for doc in documents:
        try:
            # Download and parse pages
            pages_data = _download_document_pages(client, doc)
            
            for i, page_data in enumerate(pages_data):
                # Write to temp file and parse
                import tempfile
                with tempfile.NamedTemporaryFile(suffix='.rm', delete=False) as f:
                    f.write(page_data)
                    temp_path = f.name
                
                try:
                    page = parse_rm_file(temp_path)
                    vault.sync_page(page, doc, page_number=i)
                finally:
                    os.unlink(temp_path)
            
            synced += 1
        except Exception as e:
            print(f"Failed to sync {doc.id}: {e}")
    
    return synced


def sync_to_notion(
    client: RemarkableClient,
    notion_token: str,
    parent_id: str,
    document_id: Optional[str] = None,
) -> int:
    """
    Sync documents to Notion.
    
    Args:
        client: RemarkableClient instance
        notion_token: Notion API token
        parent_id: Parent page or database ID
        document_id: Optional specific document to sync
        
    Returns:
        Number of documents synced
    """
    from .notion import NotionClient, NotionSync
    from .core import parse_rm_file
    
    notion = NotionClient(notion_token)
    sync = NotionSync(notion)
    
    documents = client.list_documents()
    
    if document_id:
        documents = [d for d in documents if d.id == document_id]
    
    synced = 0
    for doc in documents:
        try:
            pages_data = _download_document_pages(client, doc)
            pages = []
            
            for page_data in pages_data:
                import tempfile
                with tempfile.NamedTemporaryFile(suffix='.rm', delete=False) as f:
                    f.write(page_data)
                    temp_path = f.name
                
                try:
                    page = parse_rm_file(temp_path)
                    pages.append(page)
                finally:
                    os.unlink(temp_path)
            
            if pages:
                sync.push_document(pages, doc, parent_id)
                synced += 1
                
        except Exception as e:
            print(f"Failed to sync {doc.id}: {e}")
    
    return synced


def _download_document_pages(client: RemarkableClient, doc: RemarkableDocument) -> List[bytes]:
    """Download .rm page files for a document"""
    doc_blob = client.download_file(doc.hash, doc.id)
    
    pages = []
    for line in doc_blob.decode().split('\n')[1:]:
        parts = line.split(':')
        if len(parts) >= 2 and parts[1].endswith('.rm'):
            page_data = client.download_file(parts[0], parts[1])
            pages.append(page_data)
    
    return pages
