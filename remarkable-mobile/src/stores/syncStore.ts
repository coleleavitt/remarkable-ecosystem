// src/stores/syncStore.ts
import { create } from 'zustand';
import type { SyncStatus, SyncRoot } from '@/types';
import { getAPI } from '@/services/api';
import * as Cache from '@/services/cache';

interface SyncState {
  status: SyncStatus;
  syncRoot: SyncRoot | null;
  isOnline: boolean;
  
  // Actions
  checkStatus: () => Promise<void>;
  startSync: () => Promise<void>;
  setOnline: (online: boolean) => void;
  processPendingUploads: () => Promise<void>;
}

export const useSyncStore = create<SyncState>((set, get) => ({
  status: {
    state: 'idle',
    lastSync: null,
    pendingUploads: 0,
    pendingDownloads: 0,
  },
  syncRoot: null,
  isOnline: true,

  checkStatus: async () => {
    try {
      const api = getAPI();
      const serverStatus = await api.getSyncStatus();
      const root = await api.getSyncRoot();
      const pending = await Cache.getPendingUploads();
      
      set({
        status: {
          state: serverStatus.syncing ? 'syncing' : 'idle',
          lastSync: serverStatus.lastSync,
          pendingUploads: pending.length,
          pendingDownloads: 0,
        },
        syncRoot: root,
        isOnline: true,
      });
      
      // Save sync root to cache
      await Cache.saveSyncRoot(root);
    } catch (error) {
      // Try to get last known state from cache
      const cachedRoot = await Cache.getSyncRoot();
      const lastSync = await Cache.getLastSyncTime();
      const pending = await Cache.getPendingUploads();
      
      set({
        status: {
          state: 'offline',
          lastSync,
          pendingUploads: pending.length,
          pendingDownloads: 0,
          error: error instanceof Error ? error.message : 'Connection failed',
        },
        syncRoot: cachedRoot,
        isOnline: false,
      });
    }
  },

  startSync: async () => {
    const { isOnline } = get();
    if (!isOnline) {
      set((state) => ({
        status: { ...state.status, state: 'offline', error: 'No connection' },
      }));
      return;
    }

    set((state) => ({
      status: { ...state.status, state: 'syncing', progress: 0 },
    }));

    try {
      const api = getAPI();
      await api.triggerSync();
      
      // Process pending uploads
      await get().processPendingUploads();
      
      // Refresh status
      await get().checkStatus();
    } catch (error) {
      set((state) => ({
        status: {
          ...state.status,
          state: 'error',
          error: error instanceof Error ? error.message : 'Sync failed',
        },
      }));
    }
  },

  setOnline: (online: boolean) => {
    set({ isOnline: online });
    if (online) {
      get().checkStatus();
    } else {
      set((state) => ({
        status: { ...state.status, state: 'offline' },
      }));
    }
  },

  processPendingUploads: async () => {
    const pending = await Cache.getPendingUploads();
    if (pending.length === 0) return;

    const api = getAPI();
    
    for (const upload of pending) {
      try {
        await api.uploadDocument({
          name: upload.name,
          type: upload.type as 'pdf' | 'epub',
          parent: upload.parent,
          fileUri: upload.fileUri,
        });
        await Cache.removePendingUpload(upload.id);
        
        set((state) => ({
          status: {
            ...state.status,
            pendingUploads: state.status.pendingUploads - 1,
          },
        }));
      } catch (error) {
        await Cache.incrementUploadRetry(
          upload.id,
          error instanceof Error ? error.message : 'Upload failed'
        );
      }
    }
  },
}));
