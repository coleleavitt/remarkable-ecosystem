// src/services/cache.ts
// SQLite-based offline cache for documents

import * as SQLite from 'expo-sqlite';
import * as FileSystem from 'expo-file-system';
import type { Document, CachedDocument, SyncRoot } from '@/types';

const DB_NAME = 'remarkable_cache.db';
const CACHE_DIR = `${FileSystem.documentDirectory}cache/`;

let db: SQLite.SQLiteDatabase | null = null;

export async function initializeCache(): Promise<void> {
  // Ensure cache directory exists
  const dirInfo = await FileSystem.getInfoAsync(CACHE_DIR);
  if (!dirInfo.exists) {
    await FileSystem.makeDirectoryAsync(CACHE_DIR, { intermediates: true });
  }

  db = await SQLite.openDatabaseAsync(DB_NAME);

  // Create tables
  await db.execAsync(`
    PRAGMA journal_mode = WAL;
    
    CREATE TABLE IF NOT EXISTS documents (
      id TEXT PRIMARY KEY,
      hash TEXT NOT NULL,
      type TEXT NOT NULL,
      visible_name TEXT NOT NULL,
      parent TEXT,
      modified_client TEXT,
      version INTEGER DEFAULT 0,
      current_page INTEGER DEFAULT 0,
      bookmarked INTEGER DEFAULT 0,
      pinned INTEGER DEFAULT 0,
      last_modified TEXT,
      size INTEGER DEFAULT 0,
      page_count INTEGER DEFAULT 0,
      updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    );
    
    CREATE TABLE IF NOT EXISTS cached_files (
      document_id TEXT PRIMARY KEY,
      hash TEXT NOT NULL,
      local_path TEXT NOT NULL,
      cached_at TEXT DEFAULT CURRENT_TIMESTAMP,
      size INTEGER DEFAULT 0,
      FOREIGN KEY (document_id) REFERENCES documents(id)
    );
    
    CREATE TABLE IF NOT EXISTS sync_state (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL,
      updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    );
    
    CREATE TABLE IF NOT EXISTS pending_uploads (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      type TEXT NOT NULL,
      parent TEXT,
      file_uri TEXT NOT NULL,
      created_at TEXT DEFAULT CURRENT_TIMESTAMP,
      retry_count INTEGER DEFAULT 0,
      last_error TEXT
    );
    
    CREATE INDEX IF NOT EXISTS idx_documents_parent ON documents(parent);
    CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(type);
  `);
}

export function getDb(): SQLite.SQLiteDatabase {
  if (!db) throw new Error('Database not initialized');
  return db;
}

// Document operations
export async function saveDocuments(documents: Document[]): Promise<void> {
  const database = getDb();
  
  const stmt = await database.prepareAsync(`
    INSERT OR REPLACE INTO documents 
    (id, hash, type, visible_name, parent, modified_client, version, 
     current_page, bookmarked, pinned, last_modified, size, page_count, updated_at)
    VALUES ($id, $hash, $type, $visible_name, $parent, $modified_client, $version,
            $current_page, $bookmarked, $pinned, $last_modified, $size, $page_count, datetime('now'))
  `);

  try {
    for (const doc of documents) {
      await stmt.executeAsync({
        $id: doc.id,
        $hash: doc.hash,
        $type: doc.type,
        $visible_name: doc.visibleName,
        $parent: doc.parent || '',
        $modified_client: doc.modifiedClient || '',
        $version: doc.version || 0,
        $current_page: doc.currentPage || 0,
        $bookmarked: doc.bookmarked ? 1 : 0,
        $pinned: doc.pinned ? 1 : 0,
        $last_modified: doc.lastModified || '',
        $size: doc.size || 0,
        $page_count: doc.pageCount || 0,
      });
    }
  } finally {
    await stmt.finalizeAsync();
  }
}

export async function getDocuments(parent = ''): Promise<Document[]> {
  const database = getDb();
  const rows = await database.getAllAsync<{
    id: string;
    hash: string;
    type: string;
    visible_name: string;
    parent: string;
    modified_client: string;
    version: number;
    current_page: number;
    bookmarked: number;
    pinned: number;
    last_modified: string;
    size: number;
    page_count: number;
  }>(`SELECT * FROM documents WHERE parent = ?`, [parent]);

  return rows.map((row) => ({
    id: row.id,
    hash: row.hash,
    type: row.type as Document['type'],
    visibleName: row.visible_name,
    parent: row.parent,
    modifiedClient: row.modified_client,
    version: row.version,
    currentPage: row.current_page,
    bookmarked: row.bookmarked === 1,
    pinned: row.pinned === 1,
    lastModified: row.last_modified,
    size: row.size,
    pageCount: row.page_count,
  }));
}

export async function getDocument(id: string): Promise<Document | null> {
  const database = getDb();
  const row = await database.getFirstAsync<{
    id: string;
    hash: string;
    type: string;
    visible_name: string;
    parent: string;
    modified_client: string;
    version: number;
    current_page: number;
    bookmarked: number;
    pinned: number;
    last_modified: string;
    size: number;
    page_count: number;
  }>(`SELECT * FROM documents WHERE id = ?`, [id]);

  if (!row) return null;

  return {
    id: row.id,
    hash: row.hash,
    type: row.type as Document['type'],
    visibleName: row.visible_name,
    parent: row.parent,
    modifiedClient: row.modified_client,
    version: row.version,
    currentPage: row.current_page,
    bookmarked: row.bookmarked === 1,
    pinned: row.pinned === 1,
    lastModified: row.last_modified,
    size: row.size,
    pageCount: row.page_count,
  };
}

export async function deleteDocument(id: string): Promise<void> {
  const database = getDb();
  await database.runAsync(`DELETE FROM documents WHERE id = ?`, [id]);
  
  // Also delete cached file
  const cached = await getCachedFile(id);
  if (cached) {
    await FileSystem.deleteAsync(cached.localPath, { idempotent: true });
    await database.runAsync(`DELETE FROM cached_files WHERE document_id = ?`, [id]);
  }
}

// Cache file operations
export async function cacheFile(
  documentId: string,
  hash: string,
  data: string,
  isBinary = true
): Promise<string> {
  const localPath = `${CACHE_DIR}${documentId}_${hash}`;
  
  if (isBinary) {
    await FileSystem.writeAsStringAsync(localPath, data, {
      encoding: FileSystem.EncodingType.Base64,
    });
  } else {
    await FileSystem.writeAsStringAsync(localPath, data);
  }

  const info = await FileSystem.getInfoAsync(localPath);
  const size = (info as { size?: number }).size || 0;

  const database = getDb();
  await database.runAsync(
    `INSERT OR REPLACE INTO cached_files (document_id, hash, local_path, cached_at, size)
     VALUES (?, ?, ?, datetime('now'), ?)`,
    [documentId, hash, localPath, size]
  );

  return localPath;
}

export async function getCachedFile(documentId: string): Promise<CachedDocument | null> {
  const database = getDb();
  const row = await database.getFirstAsync<{
    document_id: string;
    hash: string;
    local_path: string;
    cached_at: string;
    size: number;
  }>(`SELECT * FROM cached_files WHERE document_id = ?`, [documentId]);

  if (!row) return null;

  // Verify file still exists
  const exists = await FileSystem.getInfoAsync(row.local_path);
  if (!exists.exists) {
    await database.runAsync(`DELETE FROM cached_files WHERE document_id = ?`, [documentId]);
    return null;
  }

  return {
    id: row.document_id,
    hash: row.hash,
    localPath: row.local_path,
    cachedAt: row.cached_at,
    size: row.size,
  };
}

export async function isCached(documentId: string, hash: string): Promise<boolean> {
  const cached = await getCachedFile(documentId);
  return cached !== null && cached.hash === hash;
}

// Sync state operations
export async function saveSyncRoot(root: SyncRoot): Promise<void> {
  const database = getDb();
  await database.runAsync(
    `INSERT OR REPLACE INTO sync_state (key, value, updated_at)
     VALUES ('root', ?, datetime('now'))`,
    [JSON.stringify(root)]
  );
}

export async function getSyncRoot(): Promise<SyncRoot | null> {
  const database = getDb();
  const row = await database.getFirstAsync<{ value: string }>(
    `SELECT value FROM sync_state WHERE key = 'root'`
  );
  return row ? JSON.parse(row.value) : null;
}

export async function getLastSyncTime(): Promise<string | null> {
  const database = getDb();
  const row = await database.getFirstAsync<{ updated_at: string }>(
    `SELECT updated_at FROM sync_state WHERE key = 'root'`
  );
  return row?.updated_at || null;
}

// Pending upload operations
export async function addPendingUpload(
  id: string,
  name: string,
  type: string,
  parent: string,
  fileUri: string
): Promise<void> {
  const database = getDb();
  await database.runAsync(
    `INSERT INTO pending_uploads (id, name, type, parent, file_uri, created_at)
     VALUES (?, ?, ?, ?, ?, datetime('now'))`,
    [id, name, type, parent, fileUri]
  );
}

export async function getPendingUploads(): Promise<
  Array<{
    id: string;
    name: string;
    type: string;
    parent: string;
    fileUri: string;
    retryCount: number;
  }>
> {
  const database = getDb();
  const rows = await database.getAllAsync<{
    id: string;
    name: string;
    type: string;
    parent: string;
    file_uri: string;
    retry_count: number;
  }>(`SELECT * FROM pending_uploads ORDER BY created_at ASC`);

  return rows.map((row) => ({
    id: row.id,
    name: row.name,
    type: row.type,
    parent: row.parent,
    fileUri: row.file_uri,
    retryCount: row.retry_count,
  }));
}

export async function removePendingUpload(id: string): Promise<void> {
  const database = getDb();
  await database.runAsync(`DELETE FROM pending_uploads WHERE id = ?`, [id]);
}

export async function incrementUploadRetry(id: string, error: string): Promise<void> {
  const database = getDb();
  await database.runAsync(
    `UPDATE pending_uploads SET retry_count = retry_count + 1, last_error = ? WHERE id = ?`,
    [error, id]
  );
}

// Cache cleanup
export async function clearCache(): Promise<void> {
  const database = getDb();
  
  // Delete all cached files
  const files = await database.getAllAsync<{ local_path: string }>(
    `SELECT local_path FROM cached_files`
  );
  
  for (const file of files) {
    await FileSystem.deleteAsync(file.local_path, { idempotent: true });
  }
  
  await database.execAsync(`
    DELETE FROM cached_files;
    DELETE FROM documents;
    DELETE FROM sync_state;
  `);
}

export async function getCacheSize(): Promise<number> {
  const database = getDb();
  const row = await database.getFirstAsync<{ total: number }>(
    `SELECT COALESCE(SUM(size), 0) as total FROM cached_files`
  );
  return row?.total || 0;
}
