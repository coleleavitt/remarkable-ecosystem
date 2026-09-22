import type { 
  GenTreeResponse, 
  SyncRootResponse, 
  ShareLink, 
  User, 
  AuthTokens, 
  LoginCredentials,
  Document,
  GenTreeItem
} from '../types';

const API_BASE = '/api';

class ApiError extends Error {
  status: number;
  
  constructor(status: number, message: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

async function fetchWithAuth<T>(
  url: string, 
  options: RequestInit = {}
): Promise<T> {
  const token = localStorage.getItem('accessToken');
  
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...options.headers,
  };
  
  if (token) {
    (headers as Record<string, string>)['Authorization'] = `Bearer ${token}`;
  }
  
  const response = await fetch(`${API_BASE}${url}`, {
    ...options,
    headers,
  });
  
  if (!response.ok) {
    if (response.status === 401) {
      localStorage.removeItem('accessToken');
      localStorage.removeItem('refreshToken');
      window.location.href = '/login';
    }
    throw new ApiError(response.status, await response.text());
  }
  
  return response.json();
}

// Auth API
export const auth = {
  async login(credentials: LoginCredentials): Promise<AuthTokens> {
    const response = await fetch(`${API_BASE}/token/json/2/user/new`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(credentials),
    });
    
    if (!response.ok) {
      throw new ApiError(response.status, 'Invalid credentials');
    }
    
    const data = await response.json();
    localStorage.setItem('accessToken', data.accessToken);
    if (data.refreshToken) {
      localStorage.setItem('refreshToken', data.refreshToken);
    }
    return data;
  },
  
  async logout(): Promise<void> {
    localStorage.removeItem('accessToken');
    localStorage.removeItem('refreshToken');
  },
  
  async getUser(): Promise<User> {
    return fetchWithAuth('/user/profile');
  },
  
  isAuthenticated(): boolean {
    return !!localStorage.getItem('accessToken');
  }
};

// Sync API
export const sync = {
  async getRoot(): Promise<SyncRootResponse> {
    return fetchWithAuth('/sync/v3/root');
  },
  
  async getGenTree(): Promise<GenTreeResponse> {
    return fetchWithAuth('/gentree/v1/');
  },
  
  async getFile(hash: string): Promise<ArrayBuffer> {
    const token = localStorage.getItem('accessToken');
    const response = await fetch(`${API_BASE}/sync/v3/files/${hash}`, {
      headers: token ? { 'Authorization': `Bearer ${token}` } : {},
    });
    
    if (!response.ok) {
      throw new ApiError(response.status, 'Failed to fetch file');
    }
    
    return response.arrayBuffer();
  },
  
  async uploadFile(
    file: File, 
    parentId: string,
    onProgress?: (progress: number) => void
  ): Promise<Document> {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('parent', parentId);
    
    const token = localStorage.getItem('accessToken');
    
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();
      
      xhr.upload.addEventListener('progress', (e) => {
        if (e.lengthComputable && onProgress) {
          onProgress((e.loaded / e.total) * 100);
        }
      });
      
      xhr.addEventListener('load', () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve(JSON.parse(xhr.responseText));
        } else {
          reject(new ApiError(xhr.status, xhr.responseText));
        }
      });
      
      xhr.addEventListener('error', () => {
        reject(new ApiError(0, 'Network error'));
      });
      
      xhr.open('POST', `${API_BASE}/upload/v1/file`);
      if (token) {
        xhr.setRequestHeader('Authorization', `Bearer ${token}`);
      }
      xhr.send(formData);
    });
  }
};

// Documents API
export const documents = {
  async getAll(): Promise<GenTreeItem[]> {
    const tree = await sync.getGenTree();
    return tree.items;
  },
  
  async create(data: { 
    type: 'DocumentType' | 'CollectionType';
    visibleName: string;
    parent: string;
  }): Promise<Document> {
    return fetchWithAuth('/documents/v1/create', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },
  
  async rename(id: string, name: string): Promise<Document> {
    return fetchWithAuth(`/documents/v1/${id}/rename`, {
      method: 'PUT',
      body: JSON.stringify({ visibleName: name }),
    });
  },
  
  async move(id: string, parentId: string): Promise<Document> {
    return fetchWithAuth(`/documents/v1/${id}/move`, {
      method: 'PUT',
      body: JSON.stringify({ parent: parentId }),
    });
  },
  
  async delete(id: string): Promise<void> {
    await fetchWithAuth(`/documents/v1/${id}`, {
      method: 'DELETE',
    });
  },
  
  async getMetadata(id: string): Promise<Document> {
    return fetchWithAuth(`/documents/v1/${id}`);
  }
};

// Share API
export const shares = {
  async create(documentId: string, options?: {
    expiresIn?: number;
    maxViews?: number;
  }): Promise<ShareLink> {
    return fetchWithAuth('/share/v1/link', {
      method: 'POST',
      body: JSON.stringify({ documentId, ...options }),
    });
  },
  
  async list(): Promise<ShareLink[]> {
    return fetchWithAuth('/share/v1/links');
  },
  
  async delete(id: string): Promise<void> {
    await fetchWithAuth(`/share/v1/link/${id}`, {
      method: 'DELETE',
    });
  },
  
  async getByToken(token: string): Promise<{ document: Document; pages: string[] }> {
    const response = await fetch(`${API_BASE}/share/v1/view/${token}`);
    if (!response.ok) {
      throw new ApiError(response.status, 'Share link not found or expired');
    }
    return response.json();
  }
};

// Export API
export const exportApi = {
  async toPng(hash: string, page: number, dpi = 150): Promise<Blob> {
    const token = localStorage.getItem('accessToken');
    const response = await fetch(
      `${API_BASE}/export/v1/png/${hash}?page=${page}&dpi=${dpi}`,
      { headers: token ? { 'Authorization': `Bearer ${token}` } : {} }
    );
    
    if (!response.ok) {
      throw new ApiError(response.status, 'Export failed');
    }
    
    return response.blob();
  },
  
  async toSvg(hash: string, page: number): Promise<string> {
    const token = localStorage.getItem('accessToken');
    const response = await fetch(
      `${API_BASE}/export/v1/svg/${hash}?page=${page}`,
      { headers: token ? { 'Authorization': `Bearer ${token}` } : {} }
    );
    
    if (!response.ok) {
      throw new ApiError(response.status, 'Export failed');
    }
    
    return response.text();
  },
  
  async toPdf(hash: string, pages?: number[]): Promise<Blob> {
    const token = localStorage.getItem('accessToken');
    const params = pages ? `?pages=${pages.join(',')}` : '';
    const response = await fetch(
      `${API_BASE}/export/v1/pdf/${hash}${params}`,
      { headers: token ? { 'Authorization': `Bearer ${token}` } : {} }
    );
    
    if (!response.ok) {
      throw new ApiError(response.status, 'Export failed');
    }
    
    return response.blob();
  }
};

export { ApiError };
