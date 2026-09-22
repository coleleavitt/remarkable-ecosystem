"""
Pydantic models for reMarkable API objects.

Covers document schemas, sync metadata, device info, and CRDT structures.
"""

from __future__ import annotations

import hashlib
from datetime import datetime
from enum import Enum, IntEnum
from typing import Any
from uuid import UUID

from pydantic import BaseModel, Field, computed_field


class DocumentType(str, Enum):
    """Document type in reMarkable library."""
    DOCUMENT = "DocumentType"
    COLLECTION = "CollectionType"


class SyncState(str, Enum):
    """Sync state of a document."""
    SYNCED = "synced"
    MODIFIED = "modified"
    NEW = "new"
    DELETED = "deleted"


class PenType(IntEnum):
    """Pen tool types supported by reMarkable."""
    BALLPOINT_1 = 2
    BALLPOINT_2 = 15
    FINELINER_1 = 4
    FINELINER_2 = 17
    MARKER_1 = 3
    MARKER_2 = 16
    PENCIL_1 = 1
    PENCIL_2 = 14
    MECHANICAL_PENCIL_1 = 7
    MECHANICAL_PENCIL_2 = 13
    PAINTBRUSH_1 = 5
    PAINTBRUSH_2 = 12
    HIGHLIGHTER_1 = 8
    HIGHLIGHTER_2 = 18
    ERASER = 6
    ERASE_AREA = 9
    CALLIGRAPHY = 21
    SELECTION = 10
    UNKNOWN = 0


class PenColor(IntEnum):
    """Pen colors available on reMarkable."""
    BLACK = 0
    GRAY = 1
    WHITE = 2
    YELLOW = 3
    GREEN = 4
    PINK = 5
    BLUE = 6
    RED = 7
    GRAY_OVERLAP = 8
    GREEN_2 = 9
    CYAN = 10
    MAGENTA = 11
    YELLOW_2 = 12


class DeviceInfo(BaseModel):
    """Device registration and identification."""
    device_id: str
    device_type: str = "remarkable"
    device_desc: str = "reMarkable"
    
    @computed_field
    @property
    def is_tablet(self) -> bool:
        return self.device_type == "remarkable"


class TokenPair(BaseModel):
    """Authentication tokens from reMarkable cloud."""
    device_token: str
    user_token: str
    expires_at: datetime | None = None
    scopes: list[str] = Field(default_factory=list)
    region: str = "eu"
    
    @computed_field
    @property
    def is_expired(self) -> bool:
        if self.expires_at is None:
            return False
        return datetime.now() >= self.expires_at
    
    @property
    def has_sync_scope(self) -> bool:
        return any("sync" in s for s in self.scopes)
    
    @property
    def has_write_scope(self) -> bool:
        # sync:fox is read-only; write requires subscription claim
        return "sync:tortoise" in self.scopes or "sync:turtle" in self.scopes


class SyncRoot(BaseModel):
    """Sync root hash and generation."""
    hash: str
    generation: int
    schema_version: int = 3
    
    @classmethod
    def from_response(cls, data: dict[str, Any]) -> SyncRoot:
        return cls(
            hash=data.get("hash", ""),
            generation=data.get("generation", 0),
            schema_version=data.get("schemaVersion", 3),
        )


class FileEntry(BaseModel):
    """A file entry in the document tree."""
    hash: str
    document_id: str
    filename: str
    size: int = 0
    
    @computed_field
    @property
    def is_schema(self) -> bool:
        return self.filename.endswith(".docSchema")
    
    @computed_field
    @property
    def is_rm_file(self) -> bool:
        return self.filename.endswith(".rm")


class DocumentSchema(BaseModel):
    """Document schema (metadata and page list)."""
    id: UUID
    type: DocumentType = DocumentType.DOCUMENT
    visible_name: str = ""
    parent: UUID | None = None
    pinned: bool = False
    last_modified: datetime | None = None
    version: int = 1
    pages: list[str] = Field(default_factory=list)
    cover_page_number: int = 0
    file_type: str = ""  # "pdf", "epub", "notebook"
    
    @classmethod
    def from_schema_data(cls, doc_id: str, data: dict[str, Any]) -> DocumentSchema:
        """Parse document schema JSON."""
        return cls(
            id=UUID(doc_id),
            type=DocumentType(data.get("type", "DocumentType")),
            visible_name=data.get("visibleName", ""),
            parent=UUID(data["parent"]) if data.get("parent") else None,
            pinned=data.get("pinned", False),
            last_modified=datetime.fromisoformat(data["lastModified"].replace("Z", "+00:00"))
                if data.get("lastModified") else None,
            version=data.get("version", 1),
            pages=data.get("pages", []),
            cover_page_number=data.get("coverPageNumber", 0),
            file_type=data.get("fileType", ""),
        )


class Collection(BaseModel):
    """A folder/collection in the document tree."""
    id: UUID
    name: str
    parent: UUID | None = None
    pinned: bool = False
    documents: list[UUID] = Field(default_factory=list)
    subcollections: list[UUID] = Field(default_factory=list)


class Document(BaseModel):
    """A document with metadata and pages."""
    id: UUID
    name: str
    parent: UUID | None = None
    pinned: bool = False
    last_modified: datetime | None = None
    pages: list[str] = Field(default_factory=list)
    file_type: str = ""
    current_page: int = 0
    bookmarked_pages: list[int] = Field(default_factory=list)
    
    @computed_field
    @property
    def page_count(self) -> int:
        return len(self.pages)
    
    @computed_field
    @property  
    def is_notebook(self) -> bool:
        return self.file_type == "" or self.file_type == "notebook"
    
    @computed_field
    @property
    def is_pdf(self) -> bool:
        return self.file_type == "pdf"
    
    @computed_field
    @property
    def is_epub(self) -> bool:
        return self.file_type == "epub"


class PageData(BaseModel):
    """Page metadata within a document."""
    id: str
    document_id: UUID
    index: int
    template: str = ""
    has_strokes: bool = False
    stroke_count: int = 0


# CRDT operation types for sync
class CrdtOperation(str, Enum):
    """CRDT operation types for document sync."""
    ADD_ITEM = "AddItem"
    DELETE_ITEM = "DeleteItem"
    MOVE_ITEM = "MoveItem"
    UPDATE_ITEM = "UpdateItem"
    ADD_LAYER = "AddLayer"
    DELETE_LAYER = "DeleteLayer"
    UPDATE_TEXT = "UpdateText"
    ADD_ANCHOR = "AddAnchor"
    DELETE_ANCHOR = "DeleteAnchor"
    UPDATE_ANCHOR = "UpdateAnchor"
    ADD_BOOKMARK = "AddBookmark"
    DELETE_BOOKMARK = "DeleteBookmark"
    UPDATE_CRDT_TEXT = "UpdateCrdtText"
    UNKNOWN = "Unknown"


class CrdtAuthor(BaseModel):
    """Author ID for CRDT operations."""
    author_id: int
    timestamp: int | None = None


class SyncBatch(BaseModel):
    """A batch of files to sync."""
    batch_number: int
    files: list[FileEntry] = Field(default_factory=list)
    parent_hash: str
    sync_id: str
    
    def compute_hashes(self) -> dict[str, str]:
        """Compute SHA256 hashes for all files."""
        return {f.filename: f.hash for f in self.files}


def compute_file_hash(data: bytes) -> str:
    """Compute SHA256 hash of file content (reMarkable standard)."""
    return hashlib.sha256(data).hexdigest()
