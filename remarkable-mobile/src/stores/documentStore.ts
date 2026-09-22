// src/stores/documentStore.ts
import { create } from 'zustand';
import type { Document, TreeItem } from '@/types';
import { getAPI } from '@/services/api';
import * as Cache from '@/services/cache';

interface DocumentState {
  documents: Map<string, Document>;
  currentFolder: string;
  folderPath: Array<{ id: string; name: string }>;
  isLoading: boolean;
  error: string | null;
  selectedDocuments: Set<string>;
  
  // Actions
  loadDocuments: (parentId?: string) => Promise<void>;
  loadFromCache: (parentId?: string) => Promise<void>;
  navigateToFolder: (folderId: string, folderName: string) => void;
  navigateUp: () => void;
  navigateToRoot: () => void;
  selectDocument: (id: string) => void;
  deselectDocument: (id: string) => void;
  clearSelection: () => void;
  deleteDocument: (id: string) => Promise<void>;
  renameDocument: (id: string, newName: string) => Promise<void>;
  moveDocument: (id: string, newParent: string) => Promise<void>;
  refreshDocuments: () => Promise<void>;
}

export const useDocumentStore = create<DocumentState>((set, get) => ({
  documents: new Map(),
  currentFolder: '',
  folderPath: [{ id: '', name: 'My Files' }],
  isLoading: false,
  error: null,
  selectedDocuments: new Set(),

  loadDocuments: async (parentId = '') => {
    set({ isLoading: true, error: null });
    try {
      const api = getAPI();
      const items = await api.listDocuments();
      
      // Filter to current folder and convert to Document type
      const docs = items
        .filter((item: TreeItem) => (item.parent || '') === parentId)
        .map((item: TreeItem): Document => ({
          id: item.id,
          hash: item.hash,
          type: item.type,
          visibleName: item.visibleName,
          parent: item.parent || '',
          modifiedClient: item.modifiedClient,
          version: 0,
          currentPage: 0,
          bookmarked: false,
          pinned: item.pinned,
          lastModified: item.lastModified,
        }));

      // Save to cache
      await Cache.saveDocuments(docs);

      const docsMap = new Map<string, Document>();
      docs.forEach((doc) => docsMap.set(doc.id, doc));
      
      set({ documents: docsMap, currentFolder: parentId, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to load documents',
        isLoading: false,
      });
      // Fall back to cache
      await get().loadFromCache(parentId);
    }
  },

  loadFromCache: async (parentId = '') => {
    try {
      const docs = await Cache.getDocuments(parentId);
      const docsMap = new Map<string, Document>();
      docs.forEach((doc) => docsMap.set(doc.id, doc));
      set({ documents: docsMap, currentFolder: parentId });
    } catch (error) {
      console.error('Failed to load from cache:', error);
    }
  },

  navigateToFolder: (folderId: string, folderName: string) => {
    const { folderPath } = get();
    set({
      folderPath: [...folderPath, { id: folderId, name: folderName }],
    });
    get().loadDocuments(folderId);
  },

  navigateUp: () => {
    const { folderPath } = get();
    if (folderPath.length > 1) {
      const newPath = folderPath.slice(0, -1);
      const parentId = newPath[newPath.length - 1].id;
      set({ folderPath: newPath });
      get().loadDocuments(parentId);
    }
  },

  navigateToRoot: () => {
    set({ folderPath: [{ id: '', name: 'My Files' }] });
    get().loadDocuments('');
  },

  selectDocument: (id: string) => {
    const { selectedDocuments } = get();
    const newSelection = new Set(selectedDocuments);
    newSelection.add(id);
    set({ selectedDocuments: newSelection });
  },

  deselectDocument: (id: string) => {
    const { selectedDocuments } = get();
    const newSelection = new Set(selectedDocuments);
    newSelection.delete(id);
    set({ selectedDocuments: newSelection });
  },

  clearSelection: () => {
    set({ selectedDocuments: new Set() });
  },

  deleteDocument: async (id: string) => {
    try {
      const api = getAPI();
      await api.deleteDocument(id);
      await Cache.deleteDocument(id);
      
      const { documents } = get();
      const newDocs = new Map(documents);
      newDocs.delete(id);
      set({ documents: newDocs });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Delete failed' });
      throw error;
    }
  },

  renameDocument: async (id: string, newName: string) => {
    try {
      const api = getAPI();
      const updated = await api.renameDocument(id, newName);
      
      const { documents } = get();
      const newDocs = new Map(documents);
      newDocs.set(id, updated);
      set({ documents: newDocs });
      
      await Cache.saveDocuments([updated]);
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Rename failed' });
      throw error;
    }
  },

  moveDocument: async (id: string, newParent: string) => {
    try {
      const api = getAPI();
      const updated = await api.moveDocument(id, newParent);
      
      // Remove from current view if moved to different folder
      const { documents, currentFolder } = get();
      if (newParent !== currentFolder) {
        const newDocs = new Map(documents);
        newDocs.delete(id);
        set({ documents: newDocs });
      }
      
      await Cache.saveDocuments([updated]);
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Move failed' });
      throw error;
    }
  },

  refreshDocuments: async () => {
    const { currentFolder } = get();
    await get().loadDocuments(currentFolder);
  },
}));
