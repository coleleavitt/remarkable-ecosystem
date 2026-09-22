"""
Obsidian vault synchronization
"""

import os
import re
import json
from pathlib import Path
from typing import Optional, List, Dict, Any, Iterator
from dataclasses import dataclass
from datetime import datetime

from .core import RmPage, export_svg, export_markdown


@dataclass
class VaultConfig:
    """Obsidian vault configuration"""
    path: Path
    remarkable_folder: str = "reMarkable"
    embed_svg: bool = False
    frontmatter: bool = True
    tag_prefix: str = "remarkable/"


class ObsidianVault:
    """Obsidian vault manager"""
    
    def __init__(self, path: str, config: Optional[VaultConfig] = None):
        self.path = Path(path).expanduser()
        
        if not self.path.exists():
            raise FileNotFoundError(f"Vault not found: {self.path}")
        
        self.config = config or VaultConfig(path=self.path)
        self.rm_folder = self.path / self.config.remarkable_folder
        self.rm_folder.mkdir(exist_ok=True)
        
        # Index: remarkable_id -> local path
        self._index: Dict[str, Path] = {}
        self._rebuild_index()
    
    def _rebuild_index(self) -> None:
        """Rebuild document index from disk"""
        self._index.clear()
        
        for md_file in self.rm_folder.rglob("*.md"):
            rm_id = self._extract_remarkable_id(md_file)
            if rm_id:
                self._index[rm_id] = md_file
    
    def _extract_remarkable_id(self, path: Path) -> Optional[str]:
        """Extract remarkable_id from frontmatter"""
        try:
            content = path.read_text()
            
            if not content.startswith('---'):
                return None
            
            end = content.find('---', 3)
            if end == -1:
                return None
            
            frontmatter = content[3:end]
            for line in frontmatter.split('\n'):
                if line.startswith('remarkable_id:'):
                    return line.split(':', 1)[1].strip()
            
        except Exception:
            pass
        
        return None
    
    def sync_page(
        self,
        page: RmPage,
        doc: Any,  # RemarkableDocument
        page_number: int = 0,
    ) -> Path:
        """Sync a single page to the vault"""
        # Determine filename
        title = getattr(doc, 'visible_name', '') or f"Untitled-{doc.id[:8]}"
        safe_title = self._sanitize_filename(title)
        
        # Create folder structure
        parent = getattr(doc, 'parent', '')
        if parent:
            folder = self.rm_folder / parent
        else:
            folder = self.rm_folder
        folder.mkdir(parents=True, exist_ok=True)
        
        # Generate content
        md_content = export_markdown(page, title, embed_svg=self.config.embed_svg)
        
        # Add remarkable_id to frontmatter
        md_content = self._inject_frontmatter(md_content, {
            'remarkable_id': doc.id,
            'remarkable_folder': parent or '/',
        })
        
        # Write files
        md_path = folder / f"{safe_title}.md"
        md_path.write_text(md_content)
        
        if not self.config.embed_svg and page.strokes:
            svg_content = export_svg(page)
            svg_path = folder / f"{safe_title}.svg"
            svg_path.write_text(svg_content)
        
        # Update index
        self._index[doc.id] = md_path
        
        return md_path
    
    def sync_document(
        self,
        pages: List[RmPage],
        doc: Any,
    ) -> Path:
        """Sync a multi-page document"""
        if len(pages) == 1:
            return self.sync_page(pages[0], doc)
        
        # Multi-page document
        title = getattr(doc, 'visible_name', '') or f"Untitled-{doc.id[:8]}"
        safe_title = self._sanitize_filename(title)
        
        parent = getattr(doc, 'parent', '')
        folder = (self.rm_folder / parent) if parent else self.rm_folder
        folder.mkdir(parents=True, exist_ok=True)
        
        # Create main document with page references
        lines = [
            '---',
            f'title: "{title}"',
            f'remarkable_id: {doc.id}',
            f'pages: {len(pages)}',
            'source: reMarkable',
            '---',
            '',
            f'# {title}',
            '',
        ]
        
        for i, page in enumerate(pages):
            page_title = f"{safe_title}-page-{i+1}"
            
            if self.config.embed_svg and page.strokes:
                svg = export_svg(page)
                lines.append(f'## Page {i+1}')
                lines.append('')
                lines.append(svg)
                lines.append('')
            elif page.strokes:
                lines.append(f'## Page {i+1}')
                lines.append('')
                lines.append(f'![[{page_title}.svg]]')
                lines.append('')
                
                # Write SVG file
                svg_path = folder / f"{page_title}.svg"
                svg_path.write_text(export_svg(page))
        
        lines.extend([
            '---',
            '*Synced from reMarkable*',
        ])
        
        md_path = folder / f"{safe_title}.md"
        md_path.write_text('\n'.join(lines))
        
        self._index[doc.id] = md_path
        return md_path
    
    def get_document_path(self, remarkable_id: str) -> Optional[Path]:
        """Get local path for a reMarkable document"""
        return self._index.get(remarkable_id)
    
    def list_documents(self) -> List[tuple]:
        """List all synced documents"""
        return list(self._index.items())
    
    def find_backlinks(self, remarkable_id: str) -> List[Path]:
        """Find documents linking to a reMarkable note"""
        target_path = self._index.get(remarkable_id)
        if not target_path:
            return []
        
        target_name = target_path.stem
        backlinks = []
        
        for md_file in self.path.rglob("*.md"):
            if md_file == target_path:
                continue
            
            try:
                content = md_file.read_text()
                # Find wikilinks
                if f'[[{target_name}]]' in content or f'[[{target_name}|' in content:
                    backlinks.append(md_file)
            except Exception:
                pass
        
        return backlinks
    
    def delete_document(self, remarkable_id: str) -> bool:
        """Delete a synced document"""
        path = self._index.pop(remarkable_id, None)
        if path and path.exists():
            path.unlink()
            
            # Also delete associated SVGs
            for svg in path.parent.glob(f"{path.stem}*.svg"):
                svg.unlink()
            
            return True
        return False
    
    def _sanitize_filename(self, name: str) -> str:
        """Sanitize filename"""
        return re.sub(r'[/<>:"|?*\\]', '-', name).strip('-')
    
    def _inject_frontmatter(self, content: str, extra: Dict[str, str]) -> str:
        """Inject additional frontmatter fields"""
        if not content.startswith('---'):
            return content
        
        end = content.find('---', 3)
        if end == -1:
            return content
        
        frontmatter = content[3:end].rstrip()
        for key, value in extra.items():
            if f'{key}:' not in frontmatter:
                frontmatter += f'\n{key}: {value}'
        
        return f'---{frontmatter}\n{content[end:]}'


def watch_vault(
    path: str,
    callback: callable,
    extensions: List[str] = None,
    debounce_ms: int = 500,
) -> None:
    """
    Watch Obsidian vault for changes.
    
    Args:
        path: Path to vault
        callback: Function called with (event_type, file_path)
        extensions: File extensions to watch (default: ['.md', '.svg'])
        debounce_ms: Debounce delay in milliseconds
    """
    from watchdog.observers import Observer
    from watchdog.events import FileSystemEventHandler
    import time
    
    extensions = extensions or ['.md', '.svg']
    
    class Handler(FileSystemEventHandler):
        def __init__(self):
            self.last_events = {}
        
        def _should_process(self, path: str) -> bool:
            ext = Path(path).suffix.lower()
            return ext in extensions
        
        def _debounce(self, path: str) -> bool:
            now = time.time() * 1000
            last = self.last_events.get(path, 0)
            if now - last < debounce_ms:
                return False
            self.last_events[path] = now
            return True
        
        def on_modified(self, event):
            if not event.is_directory and self._should_process(event.src_path):
                if self._debounce(event.src_path):
                    callback('modified', event.src_path)
        
        def on_created(self, event):
            if not event.is_directory and self._should_process(event.src_path):
                if self._debounce(event.src_path):
                    callback('created', event.src_path)
        
        def on_deleted(self, event):
            if not event.is_directory and self._should_process(event.src_path):
                callback('deleted', event.src_path)
    
    observer = Observer()
    observer.schedule(Handler(), path, recursive=True)
    observer.start()
    
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    
    observer.join()
