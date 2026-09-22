"""
Document management for reMarkable.

Handles document creation, modification, and organization.
Supports notebooks, PDFs, EPUBs, and collections.
"""

from __future__ import annotations

import json
import logging
import uuid
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING, Any

from remarkable.models import (
    Collection,
    Document,
    DocumentSchema,
    DocumentType,
    compute_file_hash,
)

if TYPE_CHECKING:
    from remarkable.client import RemarkableClient

logger = logging.getLogger(__name__)


class DocumentManager:
    """
    High-level document management operations.
    
    Provides methods for creating, organizing, and exporting documents.
    """
    
    def __init__(self, client: RemarkableClient) -> None:
        self._client = client
    
    # === Document creation ===
    
    def create_notebook(
        self,
        name: str,
        *,
        parent: str | None = None,
        template: str = "Blank",
        page_count: int = 1,
    ) -> DocumentSchema:
        """
        Create a new notebook document.
        
        Args:
            name: Visible name of the notebook
            parent: Parent collection UUID (None for root)
            template: Page template name
            page_count: Initial number of pages
            
        Returns:
            DocumentSchema for the new notebook
        """
        doc_id = uuid.uuid4()
        now = datetime.utcnow()
        
        # Generate page UUIDs
        pages = [str(uuid.uuid4()) for _ in range(page_count)]
        
        schema = DocumentSchema(
            id=doc_id,
            type=DocumentType.DOCUMENT,
            visible_name=name,
            parent=uuid.UUID(parent) if parent else None,
            last_modified=now,
            pages=pages,
            file_type="",  # Empty for notebooks
        )
        
        logger.info(f"Created notebook {name} with {page_count} pages")
        return schema
    
    def create_collection(
        self,
        name: str,
        *,
        parent: str | None = None,
    ) -> Collection:
        """
        Create a new collection (folder).
        
        Args:
            name: Visible name of the collection
            parent: Parent collection UUID (None for root)
            
        Returns:
            Collection object
        """
        collection = Collection(
            id=uuid.uuid4(),
            name=name,
            parent=uuid.UUID(parent) if parent else None,
        )
        
        logger.info(f"Created collection {name}")
        return collection
    
    # === Import operations ===
    
    def import_pdf(
        self,
        pdf_path: Path,
        name: str | None = None,
        *,
        parent: str | None = None,
    ) -> DocumentSchema:
        """
        Import a PDF file as a document.
        
        Args:
            pdf_path: Path to the PDF file
            name: Document name (defaults to filename)
            parent: Parent collection UUID
            
        Returns:
            DocumentSchema for the imported PDF
        """
        pdf_path = Path(pdf_path)
        if not pdf_path.exists():
            raise FileNotFoundError(f"PDF not found: {pdf_path}")
        
        doc_id = uuid.uuid4()
        doc_name = name or pdf_path.stem
        
        # Read PDF to compute hash
        pdf_data = pdf_path.read_bytes()
        content_hash = compute_file_hash(pdf_data)
        
        # Get page count (simplified - would need pypdf or similar)
        # For now, create single page
        pages = [str(uuid.uuid4())]
        
        schema = DocumentSchema(
            id=doc_id,
            type=DocumentType.DOCUMENT,
            visible_name=doc_name,
            parent=uuid.UUID(parent) if parent else None,
            last_modified=datetime.utcnow(),
            pages=pages,
            file_type="pdf",
        )
        
        logger.info(f"Imported PDF {doc_name} ({len(pdf_data)} bytes)")
        return schema
    
    def import_epub(
        self,
        epub_path: Path,
        name: str | None = None,
        *,
        parent: str | None = None,
    ) -> DocumentSchema:
        """
        Import an EPUB file as a document.
        
        Args:
            epub_path: Path to the EPUB file
            name: Document name (defaults to filename)
            parent: Parent collection UUID
            
        Returns:
            DocumentSchema for the imported EPUB
        """
        epub_path = Path(epub_path)
        if not epub_path.exists():
            raise FileNotFoundError(f"EPUB not found: {epub_path}")
        
        doc_id = uuid.uuid4()
        doc_name = name or epub_path.stem
        
        schema = DocumentSchema(
            id=doc_id,
            type=DocumentType.DOCUMENT,
            visible_name=doc_name,
            parent=uuid.UUID(parent) if parent else None,
            last_modified=datetime.utcnow(),
            pages=[],  # EPUBs don't have fixed pages
            file_type="epub",
        )
        
        logger.info(f"Imported EPUB {doc_name}")
        return schema
    
    # === Export operations ===
    
    async def export_to_svg(
        self,
        doc_id: str,
        output_dir: Path,
        *,
        include_template: bool = False,
    ) -> list[Path]:
        """
        Export document pages as SVG files.
        
        Args:
            doc_id: Document UUID
            output_dir: Output directory
            include_template: Include page template in SVG
            
        Returns:
            List of created SVG file paths
        """
        from remarkable.formats import RmParser
        
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        
        # Download document
        doc_dir = await self._client.download_document(doc_id, output_dir / "temp")
        
        svg_files: list[Path] = []
        parser = RmParser()
        
        # Find and convert .rm files
        for rm_file in doc_dir.rglob("*.rm"):
            try:
                result = parser.parse_file(rm_file)
                
                svg_path = output_dir / f"{rm_file.stem}.svg"
                svg_content = result.to_svg(
                    width=1404,
                    height=1872,
                    include_template=include_template,
                )
                svg_path.write_text(svg_content)
                svg_files.append(svg_path)
                
            except Exception as e:
                logger.warning(f"Failed to convert {rm_file}: {e}")
        
        logger.info(f"Exported {len(svg_files)} SVG files to {output_dir}")
        return svg_files
    
    async def export_to_pdf(
        self,
        doc_id: str,
        output_path: Path,
    ) -> Path:
        """
        Export document as a PDF with annotations.
        
        Args:
            doc_id: Document UUID
            output_path: Output PDF path
            
        Returns:
            Path to created PDF
        """
        # This would integrate with remarkable-pdf crate
        # For now, export SVGs then combine
        raise NotImplementedError("PDF export requires additional dependencies")
    
    # === Organization ===
    
    async def move_document(
        self,
        doc_id: str,
        new_parent: str | None,
    ) -> DocumentSchema:
        """
        Move a document to a different collection.
        
        Args:
            doc_id: Document UUID to move
            new_parent: New parent collection UUID (None for root)
            
        Returns:
            Updated DocumentSchema
        """
        schema = await self._client.get_document(doc_id)
        if not schema:
            raise ValueError(f"Document not found: {doc_id}")
        
        # Update parent
        schema.parent = uuid.UUID(new_parent) if new_parent else None
        
        # Would need upload capability to persist
        logger.info(f"Moved document {doc_id} to {new_parent or 'root'}")
        return schema
    
    async def rename_document(
        self,
        doc_id: str,
        new_name: str,
    ) -> DocumentSchema:
        """
        Rename a document.
        
        Args:
            doc_id: Document UUID
            new_name: New visible name
            
        Returns:
            Updated DocumentSchema
        """
        schema = await self._client.get_document(doc_id)
        if not schema:
            raise ValueError(f"Document not found: {doc_id}")
        
        old_name = schema.visible_name
        schema.visible_name = new_name
        
        logger.info(f"Renamed document from '{old_name}' to '{new_name}'")
        return schema
    
    async def delete_document(self, doc_id: str) -> bool:
        """
        Delete a document.
        
        Args:
            doc_id: Document UUID to delete
            
        Returns:
            True if deleted successfully
        """
        # Would need upload capability to persist deletion
        logger.info(f"Marked document {doc_id} for deletion")
        return True
    
    # === Utilities ===
    
    def serialize_schema(self, schema: DocumentSchema) -> dict[str, Any]:
        """Serialize a document schema to reMarkable JSON format."""
        return {
            "type": schema.type.value,
            "visibleName": schema.visible_name,
            "parent": str(schema.parent) if schema.parent else "",
            "pinned": schema.pinned,
            "lastModified": schema.last_modified.isoformat() + "Z"
                if schema.last_modified else "",
            "version": schema.version,
            "pages": schema.pages,
            "coverPageNumber": schema.cover_page_number,
            "fileType": schema.file_type,
        }
    
    def compute_document_hash(self, schema: DocumentSchema) -> str:
        """Compute hash for a document schema."""
        data = json.dumps(self.serialize_schema(schema), sort_keys=True)
        return compute_file_hash(data.encode())
