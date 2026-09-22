// src/hooks/useSync.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getAPI } from '@/services/api';
import * as Cache from '@/services/cache';
import { useSyncStore } from '@/stores';

export function useSync() {
  const queryClient = useQueryClient();
  const { setOnline, processPendingUploads } = useSyncStore();

  const statusQuery = useQuery({
    queryKey: ['syncStatus'],
    queryFn: async () => {
      try {
        const api = getAPI();
        const [status, root] = await Promise.all([
          api.getSyncStatus(),
          api.getSyncRoot(),
        ]);
        const pending = await Cache.getPendingUploads();
        
        setOnline(true);
        await Cache.saveSyncRoot(root);
        
        return {
          state: status.syncing ? 'syncing' : 'idle' as const,
          lastSync: status.lastSync,
          pendingUploads: pending.length,
          pendingDownloads: 0,
          root,
        };
      } catch (error) {
        setOnline(false);
        const cachedRoot = await Cache.getSyncRoot();
        const lastSync = await Cache.getLastSyncTime();
        const pending = await Cache.getPendingUploads();
        
        return {
          state: 'offline' as const,
          lastSync,
          pendingUploads: pending.length,
          pendingDownloads: 0,
          root: cachedRoot,
          error: error instanceof Error ? error.message : 'Connection failed',
        };
      }
    },
    refetchInterval: 30000, // Poll every 30 seconds
  });

  const syncMutation = useMutation({
    mutationFn: async () => {
      const api = getAPI();
      await api.triggerSync();
      await processPendingUploads();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['syncStatus'] });
      queryClient.invalidateQueries({ queryKey: ['documents'] });
    },
  });

  return {
    status: statusQuery.data,
    isLoading: statusQuery.isLoading,
    error: statusQuery.error,
    refetch: statusQuery.refetch,
    startSync: syncMutation.mutate,
    isSyncing: syncMutation.isPending,
  };
}
