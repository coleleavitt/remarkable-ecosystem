// Custom hooks for sync state management
import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import api from '../api';
import type { SyncStatusInfo, SyncConfig, DocumentState, ConflictInfo } from '../types';

// Hook for sync status with real-time updates
export function useSyncStatus() {
  const [status, setStatus] = useState<SyncStatusInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const newStatus = await api.getSyncStatus();
      setStatus(newStatus);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    refresh().finally(() => setLoading(false));

    // Listen for sync events
    const unlistenStarted = listen('sync-started', () => {
      refresh();
    });

    const unlistenCompleted = listen('sync-completed', () => {
      refresh();
    });

    const unlistenError = listen<string>('sync-error', (event) => {
      setError(event.payload);
      refresh();
    });

    return () => {
      unlistenStarted.then(fn => fn());
      unlistenCompleted.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, [refresh]);

  return { status, loading, error, refresh };
}

// Hook for sync configuration
export function useSyncConfig() {
  const [config, setConfig] = useState<SyncConfig | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getSyncConfig()
      .then(setConfig)
      .finally(() => setLoading(false));
  }, []);

  const updateConfig = async (newConfig: SyncConfig) => {
    await api.setSyncConfig(newConfig);
    setConfig(newConfig);
  };

  return { config, loading, updateConfig };
}

// Hook for documents
export function useDocuments() {
  const [documents, setDocuments] = useState<DocumentState[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const docs = await api.getDocuments();
      setDocuments(docs);
    } catch (e) {
      console.error('Failed to fetch documents:', e);
    }
  }, []);

  useEffect(() => {
    refresh().finally(() => setLoading(false));

    const unlisten = listen('sync-completed', () => {
      refresh();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [refresh]);

  return { documents, loading, refresh };
}

// Hook for folders
export function useFolders() {
  const [folders, setFolders] = useState<DocumentState[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getFolders()
      .then(setFolders)
      .finally(() => setLoading(false));
  }, []);

  return { folders, loading };
}

// Hook for conflicts
export function useConflicts() {
  const [conflicts, setConflicts] = useState<ConflictInfo[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const c = await api.getConflicts();
      setConflicts(c);
    } catch (e) {
      console.error('Failed to fetch conflicts:', e);
    }
  }, []);

  useEffect(() => {
    refresh().finally(() => setLoading(false));

    const unlisten = listen('sync-completed', () => {
      refresh();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [refresh]);

  return { conflicts, loading, refresh };
}
