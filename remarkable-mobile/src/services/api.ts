// src/services/api.ts
// API client for remarkable-server integration

import * as FileSystem from 'expo-file-system';
import type {
  Document,
  SyncRoot,
  GenTreeResponse,
  TreeItem,
  AuthToken,
  ServerConfig,
  UploadRequest,
} from '@/types';

const DEFAULT_TIMEOUT = 30000;

export class RemarkableAPIError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public response?: string
  ) {
    super(message);
    this.name = 'RemarkableAPIError';
  }
}

export class RemarkableAPI {
  private baseUrl: string;
  private token: AuthToken | null = null;
  private timeout: number;

  constructor(config: ServerConfig, timeout = DEFAULT_TIMEOUT) {
    this.baseUrl = config.baseUrl.replace(/\/+$/, '');
    this.timeout = timeout;
  }

  setToken(token: AuthToken): void {
    this.token = token;
  }

  private get headers(): Record<string, string> {
    const h: Record<string, string> = {
      'Content-Type': 'application/json',
      Accept: 'application/json',
      'User-Agent': 'remarkable-mobile/1.0.0',
    };
    if (this.token) {
      h['Authorization'] = `${this.token.tokenType} ${this.token.accessToken}`;
    }
    return h;
  }

  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        ...options,
        headers: { ...this.headers, ...options.headers },
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const text = await response.text().catch(() => '');
        throw new RemarkableAPIError(
          `API request failed: ${response.status}`,
          response.status,
          text
        );
      }

      const contentType = response.headers.get('content-type');
      if (contentType?.includes('application/json')) {
        return response.json();
      }
      return response.text() as unknown as T;
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof RemarkableAPIError) throw error;
      if (error instanceof Error && error.name === 'AbortError') {
        throw new RemarkableAPIError('Request timeout');
      }
      throw new RemarkableAPIError(`Network error: ${error}`);
    }
  }

  // Discovery endpoints
  async discover(): Promise<{
    notifications: string;
    webapp: string;
    mqttbroker: string;
  }> {
    return this.request('/discovery/v1/endpoints');
  }

  // Auth endpoints
  async registerDevice(code: string): Promise<{ token: string }> {
    return this.request('/token/json/2/device/new', {
      method: 'POST',
      body: JSON.stringify({ code }),
    });
  }

  async getUserToken(): Promise<AuthToken> {
    return this.request('/token/json/2/user/new', {
      method: 'POST',
    });
  }

  // Sync v3 endpoints
  async getSyncRoot(): Promise<SyncRoot> {
    return this.request('/sync/v3/root');
  }

  async getGenTree(): Promise<GenTreeResponse> {
    return this.request('/gentree/v1/');
  }

  async listDocuments(): Promise<TreeItem[]> {
    const tree = await this.getGenTree();
    return tree.items;
  }

  async getDocument(id: string): Promise<Document> {
    return this.request(`/sync/v3/files/${id}`);
  }

  async downloadDocument(
    id: string,
    localPath: string,
    onProgress?: (progress: number) => void
  ): Promise<string> {
    const downloadResumable = FileSystem.createDownloadResumable(
      `${this.baseUrl}/sync/v3/files/${id}/download`,
      localPath,
      {
        headers: this.headers,
      },
      (downloadProgress) => {
        const progress =
          downloadProgress.totalBytesWritten /
          downloadProgress.totalBytesExpectedToWrite;
        onProgress?.(progress);
      }
    );

    const result = await downloadResumable.downloadAsync();
    if (!result?.uri) {
      throw new RemarkableAPIError('Download failed');
    }
    return result.uri;
  }

  async uploadDocument(request: UploadRequest): Promise<Document> {
    const fileInfo = await FileSystem.getInfoAsync(request.fileUri);
    if (!fileInfo.exists) {
      throw new RemarkableAPIError('File not found');
    }

    // Read file as base64
    const base64 = await FileSystem.readAsStringAsync(request.fileUri, {
      encoding: FileSystem.EncodingType.Base64,
    });

    return this.request('/sync/v3/files', {
      method: 'POST',
      body: JSON.stringify({
        name: request.name,
        type: request.type,
        parent: request.parent,
        content: base64,
      }),
    });
  }

  async deleteDocument(id: string): Promise<void> {
    await this.request(`/sync/v3/files/${id}`, {
      method: 'DELETE',
    });
  }

  async moveDocument(id: string, newParent: string): Promise<Document> {
    return this.request(`/sync/v3/files/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({ parent: newParent }),
    });
  }

  async renameDocument(id: string, newName: string): Promise<Document> {
    return this.request(`/sync/v3/files/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({ visibleName: newName }),
    });
  }

  // SVG export endpoint
  async getDocumentSVG(id: string, page = 0): Promise<string> {
    return this.request(`/export/svg/${id}?page=${page}`);
  }

  async getDocumentPNG(id: string, page = 0, dpi = 150): Promise<string> {
    return this.request(`/export/png/${id}?page=${page}&dpi=${dpi}`);
  }

  // Sync status
  async getSyncStatus(): Promise<{ syncing: boolean; lastSync: string }> {
    return this.request('/sync/v3/status');
  }

  async triggerSync(): Promise<void> {
    await this.request('/sync/v3/sync', { method: 'POST' });
  }
}

// Singleton instance
let apiInstance: RemarkableAPI | null = null;

export function getAPI(): RemarkableAPI {
  if (!apiInstance) {
    throw new Error('API not initialized. Call initializeAPI first.');
  }
  return apiInstance;
}

export function initializeAPI(config: ServerConfig): RemarkableAPI {
  apiInstance = new RemarkableAPI(config);
  return apiInstance;
}

export function resetAPI(): void {
  apiInstance = null;
}
