import type {
  DocumentListItem,
  VersionListItem,
  DocumentVersion,
  DocumentDiff,
  MergeResult,
  MergeStrategy,
  ConflictResolution,
  Timeline,
} from './types'

const API_BASE = '/api'

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  })
  
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(error.error || 'Request failed')
  }
  
  return res.json()
}

// Documents
export async function listDocuments(): Promise<DocumentListItem[]> {
  return fetchJson(`${API_BASE}/documents`)
}

export async function uploadDocument(formData: FormData): Promise<DocumentListItem> {
  const res = await fetch(`${API_BASE}/documents`, {
    method: 'POST',
    body: formData,
  })
  
  if (!res.ok) {
    throw new Error('Upload failed')
  }
  
  return res.json()
}

export async function getDocument(id: string): Promise<DocumentListItem> {
  return fetchJson(`${API_BASE}/documents/${id}`)
}

export async function listVersions(docId: string): Promise<VersionListItem[]> {
  return fetchJson(`${API_BASE}/documents/${docId}/versions`)
}

export async function getVersion(docId: string, versionId: string): Promise<DocumentVersion> {
  return fetchJson(`${API_BASE}/documents/${docId}/versions/${versionId}`)
}

// Diff
export async function computeDiff(
  documentId: string,
  versionAId: string,
  versionBId: string,
): Promise<{ id: string; diff: DocumentDiff }> {
  return fetchJson(`${API_BASE}/diff`, {
    method: 'POST',
    body: JSON.stringify({
      document_id: documentId,
      version_a_id: versionAId,
      version_b_id: versionBId,
    }),
  })
}

export async function getDiff(id: string): Promise<DocumentDiff> {
  return fetchJson(`${API_BASE}/diff/${id}`)
}

// Merge
export async function mergeVersions(
  documentId: string,
  versionAId: string,
  versionBId: string,
  strategy: MergeStrategy = 'last_writer_wins',
): Promise<{ id: string; result: MergeResult }> {
  return fetchJson(`${API_BASE}/merge`, {
    method: 'POST',
    body: JSON.stringify({
      document_id: documentId,
      version_a_id: versionAId,
      version_b_id: versionBId,
      strategy,
    }),
  })
}

export async function resolveConflict(
  mergeId: string,
  conflictId: string,
  resolution: ConflictResolution,
): Promise<MergeResult> {
  return fetchJson(`${API_BASE}/merge/${mergeId}/resolve`, {
    method: 'POST',
    body: JSON.stringify({
      conflict_id: conflictId,
      resolution,
    }),
  })
}

export async function exportMerged(mergeId: string): Promise<DocumentVersion> {
  return fetchJson(`${API_BASE}/merge/${mergeId}/export`)
}

// Timeline
export async function getTimeline(docId: string): Promise<Timeline> {
  return fetchJson(`${API_BASE}/timeline/${docId}`)
}

export async function filterTimeline(
  docId: string,
  filters: {
    author_id?: number
    category?: string
    start_counter?: number
    end_counter?: number
  },
): Promise<Timeline> {
  return fetchJson(`${API_BASE}/timeline/${docId}/filter`, {
    method: 'POST',
    body: JSON.stringify(filters),
  })
}
