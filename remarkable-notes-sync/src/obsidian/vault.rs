//! Obsidian vault operations

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::rm_format::{
    RmPage, DocumentMetadata, MarkdownConfig, 
    export_document_markdown, export_svg, SvgConfig,
    sanitize_filename, extract_wikilinks,
};

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault not found: {0}")]
    NotFound(PathBuf),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid vault structure")]
    InvalidStructure,
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
}

/// Obsidian vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Path to vault root
    pub path: PathBuf,
    /// Subfolder for reMarkable notes
    pub remarkable_folder: String,
    /// Embed SVGs inline or as separate files
    pub embed_svg: bool,
    /// Include frontmatter
    pub frontmatter: bool,
    /// Tag prefix for reMarkable tags
    pub tag_prefix: String,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            remarkable_folder: "reMarkable".to_string(),
            embed_svg: false,  // Separate files for Obsidian
            frontmatter: true,
            tag_prefix: "remarkable/".to_string(),
        }
    }
}

/// Represents an Obsidian vault
pub struct ObsidianVault {
    config: VaultConfig,
    /// Document index: remarkable_id -> local path
    index: HashMap<String, PathBuf>,
}

impl ObsidianVault {
    /// Open or create a vault
    pub fn open(config: VaultConfig) -> Result<Self, VaultError> {
        if !config.path.exists() {
            return Err(VaultError::NotFound(config.path.clone()));
        }
        
        let mut vault = Self {
            config,
            index: HashMap::new(),
        };
        
        vault.rebuild_index()?;
        Ok(vault)
    }
    
    /// Rebuild the document index from disk
    pub fn rebuild_index(&mut self) -> Result<(), VaultError> {
        self.index.clear();
        
        let rm_folder = self.config.path.join(&self.config.remarkable_folder);
        if !rm_folder.exists() {
            fs::create_dir_all(&rm_folder)?;
        }
        
        // Scan for markdown files with remarkable_id in frontmatter
        self.scan_folder(&rm_folder)?;
        
        Ok(())
    }
    
    fn scan_folder(&mut self, folder: &Path) -> Result<(), VaultError> {
        for entry in fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.scan_folder(&path)?;
            } else if path.extension().is_some_and(|e| e == "md") {
                if let Some(id) = self.extract_remarkable_id(&path)? {
                    self.index.insert(id, path);
                }
            }
        }
        
        Ok(())
    }
    
    fn extract_remarkable_id(&self, path: &Path) -> Result<Option<String>, VaultError> {
        let content = fs::read_to_string(path)?;
        
        // Parse frontmatter
        if !content.starts_with("---") {
            return Ok(None);
        }
        
        if let Some(end) = content[3..].find("---") {
            let frontmatter = &content[3..3 + end];
            for line in frontmatter.lines() {
                if let Some(id) = line.strip_prefix("remarkable_id:") {
                    return Ok(Some(id.trim().to_string()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get the path for a document in the vault
    pub fn document_path(&self, remarkable_id: &str) -> Option<&PathBuf> {
        self.index.get(remarkable_id)
    }
    
    /// Create or update a document in the vault
    pub fn sync_document(
        &mut self,
        pages: &[RmPage],
        metadata: &DocumentMetadata,
        remarkable_id: &str,
    ) -> Result<PathBuf, VaultError> {
        // Determine target path
        let folder = if let Some(rm_folder) = &metadata.folder {
            self.config.path
                .join(&self.config.remarkable_folder)
                .join(rm_folder)
        } else {
            self.config.path.join(&self.config.remarkable_folder)
        };
        
        fs::create_dir_all(&folder)?;
        
        let filename = format!("{}.md", sanitize_filename(&metadata.title));
        let md_path = folder.join(&filename);
        
        // Create metadata with remarkable ID
        let mut doc_meta = metadata.clone();
        doc_meta.remarkable_id = Some(remarkable_id.to_string());
        
        // Add tags with prefix
        doc_meta.tags = metadata.tags.iter()
            .map(|t| format!("{}{}", self.config.tag_prefix, t))
            .collect();
        
        // Generate markdown config
        let config = MarkdownConfig {
            svg_config: SvgConfig::default(),
            embed_svg: self.config.embed_svg,
            frontmatter: self.config.frontmatter,
            include_text: true,
        };
        
        // Generate content
        let content = export_document_markdown(pages, &doc_meta, &config);
        
        // Write markdown file
        fs::write(&md_path, &content)?;
        
        // If not embedding SVGs, write them separately
        if !self.config.embed_svg {
            let svg_config = SvgConfig::default();
            for (i, page) in pages.iter().enumerate() {
                if page.stroke_count() > 0 {
                    let svg_name = if pages.len() > 1 {
                        format!("{}-page-{}.svg", sanitize_filename(&metadata.title), i + 1)
                    } else {
                        format!("{}.svg", sanitize_filename(&metadata.title))
                    };
                    let svg_path = folder.join(&svg_name);
                    let svg = export_svg(page, &svg_config);
                    fs::write(&svg_path, svg)?;
                }
            }
        }
        
        // Update index
        self.index.insert(remarkable_id.to_string(), md_path.clone());
        
        Ok(md_path)
    }
    
    /// List all documents in the vault
    pub fn list_documents(&self) -> Vec<(&String, &PathBuf)> {
        self.index.iter().collect()
    }
    
    /// Get all wikilinks in the vault pointing to reMarkable notes
    pub fn find_backlinks(&self, remarkable_id: &str) -> Result<Vec<PathBuf>, VaultError> {
        let mut backlinks = Vec::new();
        
        let target_path = match self.index.get(remarkable_id) {
            Some(p) => p,
            None => return Ok(backlinks),
        };
        
        let target_name = target_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        // Scan all markdown files
        self.find_backlinks_in_folder(&self.config.path, target_name, &mut backlinks)?;
        
        Ok(backlinks)
    }
    
    fn find_backlinks_in_folder(
        &self,
        folder: &Path,
        target: &str,
        backlinks: &mut Vec<PathBuf>,
    ) -> Result<(), VaultError> {
        for entry in fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() && !path.file_name().is_some_and(|n| n.to_str().unwrap_or("").starts_with('.')) {
                self.find_backlinks_in_folder(&path, target, backlinks)?;
            } else if path.extension().is_some_and(|e| e == "md") {
                let content = fs::read_to_string(&path)?;
                let links = extract_wikilinks(&content);
                
                if links.iter().any(|l| l == target) {
                    backlinks.push(path);
                }
            }
        }
        
        Ok(())
    }
    
    /// Delete a document from the vault
    pub fn delete_document(&mut self, remarkable_id: &str) -> Result<(), VaultError> {
        if let Some(path) = self.index.remove(remarkable_id) {
            if path.exists() {
                fs::remove_file(&path)?;
            }
            
            // Also remove associated SVG files
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(parent) = path.parent() {
                    for entry in fs::read_dir(parent)? {
                        let entry = entry?;
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        
                        if name_str.starts_with(stem) && name_str.ends_with(".svg") {
                            fs::remove_file(entry.path())?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_vault_creation() {
        let dir = tempdir().unwrap();
        let config = VaultConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let vault = ObsidianVault::open(config).unwrap();
        assert!(vault.list_documents().is_empty());
    }
}
