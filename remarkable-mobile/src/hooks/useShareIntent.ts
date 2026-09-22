// src/hooks/useShareIntent.ts
// Handle shared files from other apps (iOS Share Extension and Android Intent)

import { useEffect, useState } from 'react';
import * as Linking from 'expo-linking';
import * as FileSystem from 'expo-file-system';
import { useRouter } from 'expo-router';
import { useUpload } from './useUpload';

interface SharedFile {
  uri: string;
  name: string;
  type: string;
}

export function useShareIntent() {
  const router = useRouter();
  const { uploadAsync, isUploading } = useUpload('');
  const [sharedFiles, setSharedFiles] = useState<SharedFile[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);

  useEffect(() => {
    // Handle initial URL (app launched via share)
    Linking.getInitialURL().then((url) => {
      if (url) {
        handleIncomingUrl(url);
      }
    });

    // Handle URLs while app is running
    const subscription = Linking.addEventListener('url', ({ url }) => {
      handleIncomingUrl(url);
    });

    return () => {
      subscription.remove();
    };
  }, []);

  const handleIncomingUrl = async (url: string) => {
    try {
      // Parse the URL
      const parsed = Linking.parse(url);
      
      // Check if it's a file:// or content:// URL
      if (url.startsWith('file://') || url.startsWith('content://')) {
        await processFileUrl(url);
      } else if (parsed.queryParams?.files) {
        // Handle multiple files passed as query param
        const files = JSON.parse(parsed.queryParams.files as string);
        setSharedFiles(files);
      }
    } catch (error) {
      console.error('Error handling shared URL:', error);
    }
  };

  const processFileUrl = async (url: string) => {
    setIsProcessing(true);
    
    try {
      // Get file info
      const info = await FileSystem.getInfoAsync(url);
      if (!info.exists) {
        console.error('Shared file does not exist');
        return;
      }

      // Extract filename and type
      const fileName = url.split('/').pop() || 'document';
      const ext = fileName.split('.').pop()?.toLowerCase();
      const type = ext === 'epub' ? 'application/epub+zip' : 'application/pdf';

      // Copy to cache directory for processing
      const cacheUri = `${FileSystem.cacheDirectory}${fileName}`;
      await FileSystem.copyAsync({ from: url, to: cacheUri });

      setSharedFiles([{
        uri: cacheUri,
        name: fileName,
        type,
      }]);
    } catch (error) {
      console.error('Error processing file:', error);
    } finally {
      setIsProcessing(false);
    }
  };

  const uploadSharedFiles = async () => {
    if (sharedFiles.length === 0) return;

    setIsProcessing(true);
    
    try {
      for (const file of sharedFiles) {
        await uploadAsync({
          uri: file.uri,
          name: file.name,
          type: file.type,
        });
      }
      
      // Clear processed files
      setSharedFiles([]);
      
      // Navigate to documents
      router.replace('/(tabs)');
    } catch (error) {
      console.error('Upload failed:', error);
      throw error;
    } finally {
      setIsProcessing(false);
    }
  };

  const clearSharedFiles = () => {
    setSharedFiles([]);
  };

  return {
    sharedFiles,
    isProcessing: isProcessing || isUploading,
    uploadSharedFiles,
    clearSharedFiles,
    hasSharedFiles: sharedFiles.length > 0,
  };
}
