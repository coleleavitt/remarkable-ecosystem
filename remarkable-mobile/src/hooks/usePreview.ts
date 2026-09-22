// src/hooks/usePreview.ts
import { useQuery } from '@tanstack/react-query';
import { getAPI } from '@/services/api';
import * as Cache from '@/services/cache';
import * as FileSystem from 'expo-file-system';

export function usePreview(documentId: string, page: number = 0) {
  return useQuery({
    queryKey: ['preview', documentId, page],
    queryFn: async () => {
      // Check if we have a cached version
      const cached = await Cache.getCachedFile(`${documentId}_preview_${page}`);
      if (cached) {
        const content = await FileSystem.readAsStringAsync(cached.localPath);
        return { svg: content, fromCache: true };
      }

      // Fetch from API
      const api = getAPI();
      const svg = await api.getDocumentSVG(documentId, page);
      
      // Cache it
      await Cache.cacheFile(`${documentId}_preview_${page}`, `page_${page}`, svg, false);
      
      return { svg, fromCache: false };
    },
    staleTime: 60000, // 1 minute
    enabled: !!documentId,
  });
}

export function usePageCount(documentId: string) {
  return useQuery({
    queryKey: ['pageCount', documentId],
    queryFn: async () => {
      const api = getAPI();
      const doc = await api.getDocument(documentId);
      return doc.pageCount || 1;
    },
    enabled: !!documentId,
  });
}
