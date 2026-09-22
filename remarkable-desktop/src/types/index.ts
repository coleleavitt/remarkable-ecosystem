// Types for reMarkable sync

export interface SyncConfig {
  server_url: string;
  device_token: string | null;
  sync_interval_secs: number;
  local_path: string;
  selected_folders: string[];
  offline_mode: boolean;
  auto_start: boolean;
  notifications_enabled: boolean;
  conflict_strategy: ConflictStrategy;
}

export type ConflictStrategy = 'ask_user' | 'keep_local' | 'keep_remote' | 'keep_both';

export interface DocumentState {
  id: string;
  name: string;
  parent_id: string | null;
  doc_type: DocumentType;
  local_hash: string | null;
  remote_hash: string | null;
  last_synced: string | null;
  last_modified_local: string | null;
  last_modified_remote: string | null;
  sync_status: SyncStatus;
}

export type DocumentType = 'folder' | 'document' | 'pdf' | 'epub';
export type SyncStatus = 'synced' | 'local_only' | 'remote_only' | 'modified' | 'conflict' | 'syncing' | 'error';

export interface ConflictInfo {
  document_id: string;
  document_name: string;
  local_modified: string;
  remote_modified: string;
  local_size: number;
  remote_size: number;
}

export interface SyncStatusInfo {
  is_running: boolean;
  is_syncing: boolean;
  last_sync: string | null;
  next_sync: string | null;
  documents_synced: number;
  documents_pending: number;
  conflicts_count: number;
  errors: string[];
  offline_mode: boolean;
}
