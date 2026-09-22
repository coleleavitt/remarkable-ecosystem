// API layer for Tauri commands
import { invoke } from '@tauri-apps/api/core';
import type { 
  SyncConfig, 
  SyncStatusInfo, 
  DocumentState, 
  ConflictInfo,
  ConflictStrategy 
} from '../types';

export const api = {
  // Sync status
  async getSyncStatus(): Promise<SyncStatusInfo> {
    return invoke('get_sync_status');
  },

  // Configuration
  async getSyncConfig(): Promise<SyncConfig> {
    return invoke('get_sync_config');
  },

  async setSyncConfig(config: SyncConfig): Promise<void> {
    return invoke('set_sync_config', { config });
  },

  // Sync control
  async startSync(): Promise<void> {
    return invoke('start_sync');
  },

  async stopSync(): Promise<void> {
    return invoke('stop_sync');
  },

  // Documents
  async getDocuments(): Promise<DocumentState[]> {
    return invoke('get_documents');
  },

  async getFolders(): Promise<DocumentState[]> {
    return invoke('get_folders');
  },

  async setSelectedFolders(folders: string[]): Promise<void> {
    return invoke('set_selected_folders', { folders });
  },

  // Conflicts
  async getConflicts(): Promise<ConflictInfo[]> {
    return invoke('get_conflicts');
  },

  async resolveConflict(documentId: string, strategy: ConflictStrategy): Promise<void> {
    return invoke('resolve_conflict', { 
      document_id: documentId, 
      strategy 
    });
  },

  // Offline mode
  async setOfflineMode(offline: boolean): Promise<void> {
    return invoke('set_offline_mode', { offline });
  },

  async getOfflineMode(): Promise<boolean> {
    return invoke('get_offline_mode');
  },

  // Connection test
  async testConnection(): Promise<boolean> {
    return invoke('test_connection');
  },

  // Import/Export
  async exportDocument(documentId: string, exportPath: string): Promise<void> {
    return invoke('export_document', { 
      document_id: documentId, 
      export_path: exportPath 
    });
  },

  async importDocument(importPath: string, parentId?: string): Promise<string> {
    return invoke('import_document', { 
      import_path: importPath, 
      parent_id: parentId 
    });
  },
};

export default api;
