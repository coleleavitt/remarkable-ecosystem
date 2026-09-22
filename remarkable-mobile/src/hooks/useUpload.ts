// src/hooks/useUpload.ts
import { useMutation, useQueryClient } from '@tanstack/react-query';
import * as DocumentPicker from 'expo-document-picker';
import * as FileSystem from 'expo-file-system';
import { getAPI, RemarkableAPIError } from '@/services/api';
import * as Cache from '@/services/cache';
import { useSyncStore } from '@/stores';

function generateId(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

export function useUpload(currentFolder: string = '') {
  const queryClient = useQueryClient();
  const { isOnline } = useSyncStore();

  const uploadMutation = useMutation({
    mutationFn: async (file: { uri: string; name: string; type: string }) => {
      // Determine file type
      const ext = file.name.split('.').pop()?.toLowerCase();
      const docType = ext === 'epub' ? 'epub' : 'pdf';
      const docName = file.name.replace(/\.(pdf|epub)$/i, '');

      if (!isOnline) {
        // Queue for offline upload
        const id = generateId();
        await Cache.addPendingUpload(id, docName, docType, currentFolder, file.uri);
        return { id, name: docName, queued: true };
      }

      // Upload directly
      const api = getAPI();
      const result = await api.uploadDocument({
        name: docName,
        type: docType,
        parent: currentFolder,
        fileUri: file.uri,
      });
      return { ...result, queued: false };
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents', currentFolder] });
      queryClient.invalidateQueries({ queryKey: ['syncStatus'] });
    },
  });

  const pickAndUpload = async () => {
    try {
      const result = await DocumentPicker.getDocumentAsync({
        type: ['application/pdf', 'application/epub+zip'],
        copyToCacheDirectory: true,
        multiple: true,
      });

      if (result.canceled) return [];

      const uploads = result.assets.map((asset) =>
        uploadMutation.mutateAsync({
          uri: asset.uri,
          name: asset.name,
          type: asset.mimeType || 'application/pdf',
        })
      );

      return Promise.all(uploads);
    } catch (error) {
      throw error;
    }
  };

  return {
    upload: uploadMutation.mutate,
    uploadAsync: uploadMutation.mutateAsync,
    pickAndUpload,
    isUploading: uploadMutation.isPending,
    error: uploadMutation.error,
  };
}
