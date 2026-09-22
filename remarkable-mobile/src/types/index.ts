// src/types/index.ts
// Type definitions for reMarkable sync API

export type DocumentType = 'DocumentType' | 'CollectionType' | 'NotebookType';

export interface Document {
  id: string;
  hash: string;
  type: DocumentType;
  visibleName: string;
  parent: string;
  modifiedClient: string;
  version: number;
  currentPage: number;
  bookmarked: boolean;
  pinned?: boolean;
  lastModified?: string;
  lastOpened?: string;
  size?: number;
  pageCount?: number;
}

export interface Folder {
  id: string;
  name: string;
  parent: string;
  children: (Document | Folder)[];
  modifiedClient: string;
}

export interface SyncRoot {
  hash: string;
  generation: number;
  schemaVersion: string;
}

export interface GenTreeResponse {
  generation: number;
  hash: string;
  schemaVersion: string;
  items: TreeItem[];
  pageCount: number;
  documentCount: number;
  folderCount: number;
}

export interface TreeItem {
  id: string;
  hash: string;
  type: DocumentType;
  visibleName: string;
  parent: string;
  lastModified: string;
  pinned: boolean;
  modifiedClient: string;
}

export interface SyncStatus {
  state: 'idle' | 'syncing' | 'error' | 'offline';
  lastSync: string | null;
  pendingUploads: number;
  pendingDownloads: number;
  progress?: number;
  error?: string;
}

export interface AuthToken {
  accessToken: string;
  tokenType: string;
  expiresAt: string | null;
  tectonicRegion: string | null;
}

export interface ServerConfig {
  baseUrl: string;
  name: string;
  isCloud: boolean;
}

export interface UploadRequest {
  name: string;
  type: 'pdf' | 'epub';
  parent: string;
  fileUri: string;
}

export interface DownloadProgress {
  documentId: string;
  bytesDownloaded: number;
  totalBytes: number;
  progress: number;
}

export interface CachedDocument {
  id: string;
  hash: string;
  localPath: string;
  cachedAt: string;
  size: number;
}

// SVG rendering types
export interface Stroke {
  pen: PenType;
  color: StrokeColor;
  width: number;
  points: Point[];
}

export interface Point {
  x: number;
  y: number;
  pressure: number;
  tilt?: number;
  speed?: number;
}

export type PenType =
  | 'ballpoint'
  | 'ballpoint_v2'
  | 'fineliner'
  | 'fineliner_v2'
  | 'marker'
  | 'marker_v2'
  | 'pencil'
  | 'pencil_v2'
  | 'mechanical_pencil'
  | 'sharp_pencil'
  | 'brush'
  | 'highlighter'
  | 'highlighter_v2'
  | 'eraser'
  | 'eraser_area'
  | 'calligraphy'
  | 'paint'
  | 'paintbrush';

export type StrokeColor =
  | 'black'
  | 'gray'
  | 'white'
  | 'yellow'
  | 'green'
  | 'pink'
  | 'blue'
  | 'red'
  | 'gray_overlap';

// Push notification types
export interface SyncNotification {
  type: 'sync_complete' | 'sync_error' | 'document_added' | 'document_modified';
  documentId?: string;
  documentName?: string;
  message: string;
  timestamp: string;
}
