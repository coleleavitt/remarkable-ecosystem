// Document types from reMarkable API
export interface Document {
  id: string;
  hash: string;
  type: 'DocumentType' | 'CollectionType';
  visibleName: string;
  parent: string;
  version: number;
  pinned: boolean;
  lastModified: string;
  lastOpened?: string;
  createdTime?: string;
  currentPage?: number;
  bookmarked?: boolean;
  fileType?: 'notebook' | 'pdf' | 'epub';
  pageCount?: number;
  sizeInBytes?: number;
}

export interface Folder extends Document {
  type: 'CollectionType';
}

export interface File extends Document {
  type: 'DocumentType';
  fileType: 'notebook' | 'pdf' | 'epub';
  pageCount: number;
}

export interface GenTreeItem {
  id: string;
  hash: string;
  type: 'DocumentType' | 'CollectionType';
  visibleName: string;
  parent: string;
  version: number;
  pinned: boolean;
  lastModified: string;
  fileType?: string;
  pageCount?: number;
}

export interface GenTreeResponse {
  generation: number;
  hash: string;
  schemaVersion: number;
  items: GenTreeItem[];
  pageCount: number;
  documentCount: number;
  folderCount: number;
}

export interface SyncRootResponse {
  generation: number;
  hash: string;
  schemaVersion: number;
}

export interface ShareLink {
  id: string;
  documentId: string;
  token: string;
  expiresAt: string;
  createdAt: string;
  viewCount: number;
  maxViews?: number;
}

export interface User {
  id: string;
  email: string;
  name?: string;
  createdAt: string;
  syncStatus: 'synced' | 'syncing' | 'offline';
  lastSyncTime?: string;
  storageUsed: number;
  storageLimit: number;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken?: string;
  expiresAt: string;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface UploadProgress {
  fileId: string;
  fileName: string;
  progress: number;
  status: 'pending' | 'uploading' | 'processing' | 'complete' | 'error';
  error?: string;
}

// Preview types
export interface PagePreview {
  pageNumber: number;
  svgContent?: string;
  loading: boolean;
  error?: string;
}

export interface ExportOptions {
  format: 'svg' | 'png' | 'pdf';
  dpi?: number;
  pages?: number[];
  includeBackground?: boolean;
}
