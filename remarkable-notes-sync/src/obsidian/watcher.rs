//! File system watcher for Obsidian vault changes

use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WatchError {
    #[error("Watch error: {0}")]
    Notify(#[from] notify::Error),
    
    #[error("Channel error")]
    Channel,
}

/// Types of file changes
#[derive(Debug, Clone)]
pub enum FileChange {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Debounce duration in milliseconds
    pub debounce_ms: u64,
    /// Watch subdirectories recursively
    pub recursive: bool,
    /// File extensions to watch (empty = all)
    pub extensions: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 500,
            recursive: true,
            extensions: vec!["md".to_string(), "svg".to_string()],
        }
    }
}

/// Simple file watcher for vault changes
pub struct VaultWatcher {
    config: WatchConfig,
    watcher: RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
}

impl VaultWatcher {
    /// Create a new watcher for the given path
    pub fn new(path: &Path, config: WatchConfig) -> Result<Self, WatchError> {
        let (tx, rx) = channel();
        
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_millis(config.debounce_ms)),
        )?;
        
        let mode = if config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        
        watcher.watch(path, mode)?;
        
        Ok(Self {
            config,
            watcher,
            receiver: rx,
        })
    }
    
    /// Poll for changes (non-blocking)
    pub fn poll(&self) -> Option<Vec<FileChange>> {
        let mut changes = Vec::new();
        
        while let Ok(result) = self.receiver.try_recv() {
            if let Ok(event) = result {
                for path in event.paths {
                    if self.should_process(&path) {
                        let change = match event.kind {
                            notify::EventKind::Create(_) => FileChange::Created(path),
                            notify::EventKind::Modify(_) => FileChange::Modified(path),
                            notify::EventKind::Remove(_) => FileChange::Deleted(path),
                            _ => continue,
                        };
                        changes.push(change);
                    }
                }
            }
        }
        
        if changes.is_empty() {
            None
        } else {
            Some(changes)
        }
    }
    
    /// Wait for the next batch of changes (blocking)
    pub fn wait(&self) -> Result<Vec<FileChange>, WatchError> {
        let result = self.receiver.recv()
            .map_err(|_| WatchError::Channel)?;
        
        let event = result?;
        let mut changes = Vec::new();
        
        for path in event.paths {
            if self.should_process(&path) {
                let change = match event.kind {
                    notify::EventKind::Create(_) => FileChange::Created(path),
                    notify::EventKind::Modify(_) => FileChange::Modified(path),
                    notify::EventKind::Remove(_) => FileChange::Deleted(path),
                    _ => continue,
                };
                changes.push(change);
            }
        }
        
        Ok(changes)
    }
    
    fn should_process(&self, path: &Path) -> bool {
        if self.config.extensions.is_empty() {
            return true;
        }
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            self.config.extensions.iter().any(|e| e == ext)
        } else {
            false
        }
    }
    
    /// Stop watching
    pub fn stop(&mut self, path: &Path) -> Result<(), WatchError> {
        self.watcher.unwatch(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_watcher_creation() {
        let dir = tempdir().unwrap();
        let config = WatchConfig::default();
        
        let watcher = VaultWatcher::new(dir.path(), config);
        assert!(watcher.is_ok());
    }
}
