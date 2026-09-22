"""
Sync operations for reMarkable cloud.

Handles document tree traversal, file download, and backup operations
using the sync v3 API (tectonic).
"""

from __future__ import annotations

import asyncio
import base64
import json
import logging
import struct
from pathlib import Path
from typing import TYPE_CHECKING, Any
from uuid import UUID

from remarkable.models import (
    Document,
    DocumentSchema,
    DocumentType,
    FileEntry,
    SyncRoot,
    compute_file_hash,
)

if TYPE_CHECKING:
    from remarkable.client import RemarkableClient

logger = logging.getLogger(__name__)


class SyncClient:
    """
    Client for sync v3 API operations.
    
    Handles document tree traversal, file download/upload, and backups.
    """
    
    def __init__(self, client: RemarkableClient) -> None:
        self._client = client
        self._cache: dict[str, bytes] = {}
        self._document_cache: dict[str, DocumentSchema] = {}
    
    @property
    def base_url(self) -> str:
        return self._client.tectonic_url
    
    # === Root operations ===
    
    async def get_root(self) -> SyncRoot:
        """
        Get the current sync root hash and generation.
        
        Returns:
            SyncRoot with hash and generation number
        """
        url = f"{self.base_url}/sync/v3/root"
        resp = await self._client._get(url)
        resp.raise_for_status()
        return SyncRoot.from_response(resp.json())
    
    # === File operations ===
    
    async def download_file(
        self, file_hash: str, filename: str
    ) -> bytes:
        """
        Download a file by hash.
        
        The rm-filename header is REQUIRED by the API and must contain
        just the filename (not a path).
        
        Args:
            file_hash: SHA256 hash of the file
            filename: Filename for rm-filename header (e.g., "root.docSchema")
            
        Returns:
            File contents as bytes
        """
        if file_hash in self._cache:
            return self._cache[file_hash]
        
        url = f"{self.base_url}/sync/v3/files/{file_hash}"
        
        # rm-filename header must be just the filename, not a path
        clean_filename = Path(filename).name
        headers = {"rm-filename": clean_filename}
        
        resp = await self._client._get(url, headers=headers)
        resp.raise_for_status()
        
        data = resp.content
        
        # Verify hash
        actual_hash = compute_file_hash(data)
        if actual_hash != file_hash:
            logger.warning(
                f"Hash mismatch for {filename}: expected {file_hash}, got {actual_hash}"
            )
        
        # Verify CRC32C if provided
        if "x-goog-hash" in resp.headers:
            expected_crc = self._parse_crc32c_header(resp.headers["x-goog-hash"])
            if expected_crc:
                actual_crc = self._compute_crc32c(data)
                if actual_crc != expected_crc:
                    logger.warning(f"CRC32C mismatch for {filename}")
        
        self._cache[file_hash] = data
        return data
    
    def _parse_crc32c_header(self, header: str) -> bytes | None:
        """Parse x-goog-hash header for CRC32C value."""
        for part in header.split(","):
            part = part.strip()
            if part.startswith("crc32c="):
                try:
                    return base64.b64decode(part[7:])
                except Exception:
                    return None
        return None
    
    def _compute_crc32c(self, data: bytes) -> bytes:
        """Compute CRC32C checksum."""
        try:
            import crc32c  # type: ignore
            crc = crc32c.crc32c(data)
            return struct.pack(">I", crc)
        except ImportError:
            # Fallback: no verification
            return b""
    
    # === Document tree ===
    
    async def get_root_index(self) -> dict[str, Any]:
        """
        Download and parse the root document index.
        
        Returns:
            Parsed root.docSchema JSON
        """
        root = await self.get_root()
        data = await self.download_file(root.hash, "root.docSchema")
        return json.loads(data)
    
    async def list_documents(self) -> list[Document]:
        """
        List all documents in the library.
        
        Traverses the document tree starting from root.
        
        Returns:
            List of Document objects
        """
        root_index = await self.get_root_index()
        documents: list[Document] = []
        
        # Parse file entries from root index
        file_entries = root_index.get("files", [])
        
        # Download each document schema
        tasks = []
        for entry in file_entries:
            doc_hash = entry.get("hash", "")
            doc_id = entry.get("documentId", "")
            filename = entry.get("filename", "")
            
            if filename.endswith(".docSchema"):
                tasks.append(self._load_document(doc_id, doc_hash))
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        for result in results:
            if isinstance(result, Document):
                documents.append(result)
            elif isinstance(result, Exception):
                logger.warning(f"Failed to load document: {result}")
        
        return documents
    
    async def _load_document(self, doc_id: str, doc_hash: str) -> Document:
        """Load a document schema and convert to Document."""
        filename = f"{doc_id}.docSchema"
        data = await self.download_file(doc_hash, filename)
        schema_data = json.loads(data)
        
        schema = DocumentSchema.from_schema_data(doc_id, schema_data)
        self._document_cache[doc_id] = schema
        
        # Skip collections
        if schema.type == DocumentType.COLLECTION:
            raise ValueError(f"Skipping collection {doc_id}")
        
        return Document(
            id=schema.id,
            name=schema.visible_name,
            parent=schema.parent,
            pinned=schema.pinned,
            last_modified=schema.last_modified,
            pages=schema.pages,
            file_type=schema.file_type,
        )
    
    async def get_document(self, doc_id: str) -> DocumentSchema | None:
        """
        Get a specific document schema by ID.
        
        Args:
            doc_id: Document UUID string
            
        Returns:
            DocumentSchema or None if not found
        """
        if doc_id in self._document_cache:
            return self._document_cache[doc_id]
        
        # Need to search the root index
        root_index = await self.get_root_index()
        
        for entry in root_index.get("files", []):
            if entry.get("documentId") == doc_id:
                filename = f"{doc_id}.docSchema"
                data = await self.download_file(entry["hash"], filename)
                schema_data = json.loads(data)
                schema = DocumentSchema.from_schema_data(doc_id, schema_data)
                self._document_cache[doc_id] = schema
                return schema
        
        return None
    
    # === Download operations ===
    
    async def download_document(
        self, doc_id: str, output_dir: Path
    ) -> Path:
        """
        Download a complete document with all pages.
        
        Args:
            doc_id: Document UUID
            output_dir: Directory to save files
            
        Returns:
            Path to the document directory
        """
        doc_dir = output_dir / doc_id
        doc_dir.mkdir(parents=True, exist_ok=True)
        
        # Get document schema
        schema = await self.get_document(doc_id)
        if not schema:
            raise ValueError(f"Document not found: {doc_id}")
        
        # Save schema
        schema_path = doc_dir / f"{doc_id}.docSchema"
        schema_path.write_text(json.dumps(schema.model_dump(), indent=2, default=str))
        
        # Download all files for this document
        root_index = await self.get_root_index()
        
        tasks = []
        for entry in root_index.get("files", []):
            entry_doc_id = entry.get("documentId", "")
            if entry_doc_id != doc_id:
                continue
            
            filename = entry.get("filename", "")
            file_hash = entry.get("hash", "")
            
            # Skip the schema we already saved
            if filename.endswith(".docSchema"):
                continue
            
            tasks.append(
                self._download_file_to_disk(
                    file_hash, filename, doc_dir, entry_doc_id
                )
            )
        
        await asyncio.gather(*tasks, return_exceptions=True)
        
        logger.info(f"Downloaded document {doc_id} to {doc_dir}")
        return doc_dir
    
    async def _download_file_to_disk(
        self,
        file_hash: str,
        filename: str,
        output_dir: Path,
        doc_id: str,
    ) -> Path:
        """Download a file and save to disk."""
        # Parse filename path (e.g., "doc-id/page-uuid.rm")
        parts = filename.split("/")
        if len(parts) > 1:
            # Create subdirectory structure
            file_path = output_dir / Path(*parts[1:])
        else:
            file_path = output_dir / filename
        
        file_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Use just the filename for rm-filename header
        clean_filename = Path(filename).name
        data = await self.download_file(file_hash, clean_filename)
        file_path.write_bytes(data)
        
        logger.debug(f"Downloaded {filename} ({len(data)} bytes)")
        return file_path
    
    # === Backup operations ===
    
    async def backup(
        self,
        output_dir: Path,
        *,
        include_raw: bool = True,
        max_concurrent: int = 10,
    ) -> dict[str, Any]:
        """
        Perform a full account backup.
        
        Downloads all documents and their files.
        
        Args:
            output_dir: Directory to save backup
            include_raw: Include raw .rm stroke files
            max_concurrent: Maximum concurrent downloads
            
        Returns:
            Backup statistics
        """
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        
        # Get root
        root = await self.get_root()
        logger.info(f"Starting backup from root hash {root.hash[:16]}...")
        
        # Save root index
        root_index = await self.get_root_index()
        (output_dir / "root.docSchema").write_text(
            json.dumps(root_index, indent=2)
        )
        
        # Collect all files to download
        files_to_download: list[tuple[str, str, str]] = []
        
        for entry in root_index.get("files", []):
            doc_id = entry.get("documentId", "")
            filename = entry.get("filename", "")
            file_hash = entry.get("hash", "")
            
            if not include_raw and filename.endswith(".rm"):
                continue
            
            files_to_download.append((file_hash, filename, doc_id))
        
        # Download with concurrency limit
        semaphore = asyncio.Semaphore(max_concurrent)
        
        async def download_with_limit(
            file_hash: str, filename: str, doc_id: str
        ) -> tuple[str, bool]:
            async with semaphore:
                try:
                    await self._download_file_to_disk(
                        file_hash, filename, output_dir / doc_id, doc_id
                    )
                    return filename, True
                except Exception as e:
                    logger.warning(f"Failed to download {filename}: {e}")
                    return filename, False
        
        tasks = [
            download_with_limit(h, f, d) for h, f, d in files_to_download
        ]
        results = await asyncio.gather(*tasks)
        
        # Compile stats
        succeeded = sum(1 for _, ok in results if ok)
        failed = sum(1 for _, ok in results if not ok)
        
        stats = {
            "root_hash": root.hash,
            "generation": root.generation,
            "total_files": len(files_to_download),
            "succeeded": succeeded,
            "failed": failed,
            "output_dir": str(output_dir),
        }
        
        # Save stats
        (output_dir / "backup_stats.json").write_text(
            json.dumps(stats, indent=2)
        )
        
        logger.info(
            f"Backup complete: {succeeded}/{len(files_to_download)} files"
        )
        return stats


class LocalSyncClient:
    """
    Client for syncing with a local reMarkable server.
    
    Used for offline sync or with custom sync servers.
    """
    
    def __init__(self, base_url: str = "http://localhost:8080") -> None:
        self.base_url = base_url
        self._http = None
    
    async def connect(self) -> None:
        """Connect to the local server."""
        import httpx
        self._http = httpx.AsyncClient(base_url=self.base_url)
    
    async def close(self) -> None:
        """Close the connection."""
        if self._http:
            await self._http.aclose()
    
    async def __aenter__(self) -> LocalSyncClient:
        await self.connect()
        return self
    
    async def __aexit__(self, *args: Any) -> None:
        await self.close()
