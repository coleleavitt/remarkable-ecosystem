// src/hooks/useDocuments.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getAPI } from '@/services/api';
import * as Cache from '@/services/cache';
import type { Document, TreeItem } from '@/types';

export function useDocuments(parentId: string = '') {
  const queryClient = useQueryClient();

  const documentsQuery = useQuery({
    queryKey: ['documents', parentId],
    queryFn: async () => {
      try {
        const api = getAPI();
        const items = await api.listDocuments();
        
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

        // Cache for offline
        await Cache.saveDocuments(docs);
        return docs;
      } catch (error) {
        // Fall back to cache
        const cached = await Cache.getDocuments(parentId);
        if (cached.length > 0) return cached;
        throw error;
      }
    },
    staleTime: 30000, // 30 seconds
  });

  const deleteDoc = useMutation({
    mutationFn: async (id: string) => {
      const api = getAPI();
      await api.deleteDocument(id);
      await Cache.deleteDocument(id);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents', parentId] });
    },
  });

  const renameDoc = useMutation({
    mutationFn: async ({ id, name }: { id: string; name: string }) => {
      const api = getAPI();
      const updated = await api.renameDocument(id, name);
      await Cache.saveDocuments([updated]);
      return updated;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents', parentId] });
    },
  });

  const moveDoc = useMutation({
    mutationFn: async ({ id, newParent }: { id: string; newParent: string }) => {
      const api = getAPI();
      const updated = await api.moveDocument(id, newParent);
      await Cache.saveDocuments([updated]);
      return updated;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] });
    },
  });

  return {
    documents: documentsQuery.data ?? [],
    isLoading: documentsQuery.isLoading,
    error: documentsQuery.error,
    refetch: documentsQuery.refetch,
    deleteDocument: deleteDoc.mutate,
    renameDocument: renameDoc.mutate,
    moveDocument: moveDoc.mutate,
    isDeleting: deleteDoc.isPending,
    isRenaming: renameDoc.isPending,
    isMoving: moveDoc.isPending,
  };
}
