// src/components/upload/UploadSheet.tsx
import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Modal,
  Pressable,
  TextInput,
  Platform,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import * as DocumentPicker from 'expo-document-picker';
import { Button } from '@/components/ui';
import { useUpload } from '@/hooks';

interface UploadSheetProps {
  visible: boolean;
  onClose: () => void;
  currentFolder: string;
}

export function UploadSheet({ visible, onClose, currentFolder }: UploadSheetProps) {
  const [selectedFile, setSelectedFile] = useState<{
    uri: string;
    name: string;
    type: string;
  } | null>(null);
  const [customName, setCustomName] = useState('');
  
  const { uploadAsync, isUploading } = useUpload(currentFolder);

  const handlePickFile = async () => {
    try {
      const result = await DocumentPicker.getDocumentAsync({
        type: ['application/pdf', 'application/epub+zip'],
        copyToCacheDirectory: true,
      });

      if (!result.canceled && result.assets.length > 0) {
        const asset = result.assets[0];
        setSelectedFile({
          uri: asset.uri,
          name: asset.name,
          type: asset.mimeType || 'application/pdf',
        });
        setCustomName(asset.name.replace(/\.(pdf|epub)$/i, ''));
      }
    } catch (error) {
      console.error('Failed to pick file:', error);
    }
  };

  const handleUpload = async () => {
    if (!selectedFile) return;

    try {
      await uploadAsync({
        uri: selectedFile.uri,
        name: customName || selectedFile.name,
        type: selectedFile.type,
      });
      handleClose();
    } catch (error) {
      console.error('Upload failed:', error);
    }
  };

  const handleClose = () => {
    setSelectedFile(null);
    setCustomName('');
    onClose();
  };

  return (
    <Modal
      visible={visible}
      animationType="slide"
      transparent
      onRequestClose={handleClose}
    >
      <View style={styles.overlay}>
        <View style={styles.sheet}>
          <View style={styles.header}>
            <Text style={styles.title}>Upload Document</Text>
            <Pressable onPress={handleClose}>
              <Ionicons name="close" size={24} color="#6B7280" />
            </Pressable>
          </View>

          {!selectedFile ? (
            <Pressable style={styles.dropzone} onPress={handlePickFile}>
              <Ionicons name="cloud-upload-outline" size={48} color="#9CA3AF" />
              <Text style={styles.dropzoneText}>
                Tap to select PDF or EPUB
              </Text>
            </Pressable>
          ) : (
            <View style={styles.selectedFile}>
              <View style={styles.fileInfo}>
                <Ionicons
                  name={selectedFile.type.includes('epub') ? 'book' : 'document'}
                  size={32}
                  color="#3B82F6"
                />
                <View style={styles.fileDetails}>
                  <Text style={styles.fileName} numberOfLines={1}>
                    {selectedFile.name}
                  </Text>
                  <Text style={styles.fileType}>
                    {selectedFile.type.includes('epub') ? 'EPUB' : 'PDF'}
                  </Text>
                </View>
              </View>

              <View style={styles.nameInput}>
                <Text style={styles.label}>Document name</Text>
                <TextInput
                  style={styles.input}
                  value={customName}
                  onChangeText={setCustomName}
                  placeholder="Enter name..."
                  autoCapitalize="words"
                />
              </View>
            </View>
          )}

          <View style={styles.actions}>
            <Button
              title="Cancel"
              onPress={handleClose}
              variant="outline"
              style={styles.button}
            />
            <Button
              title={isUploading ? 'Uploading...' : 'Upload'}
              onPress={handleUpload}
              disabled={!selectedFile}
              loading={isUploading}
              style={styles.button}
            />
          </View>
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'flex-end',
  },
  sheet: {
    backgroundColor: '#fff',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    padding: 20,
    paddingBottom: Platform.OS === 'ios' ? 40 : 20,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 20,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#111827',
  },
  dropzone: {
    borderWidth: 2,
    borderColor: '#E5E7EB',
    borderStyle: 'dashed',
    borderRadius: 12,
    padding: 40,
    alignItems: 'center',
    marginBottom: 20,
  },
  dropzoneText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6B7280',
  },
  selectedFile: {
    marginBottom: 20,
  },
  fileInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    backgroundColor: '#F3F4F6',
    borderRadius: 12,
    marginBottom: 16,
  },
  fileDetails: {
    marginLeft: 12,
    flex: 1,
  },
  fileName: {
    fontSize: 14,
    fontWeight: '500',
    color: '#111827',
  },
  fileType: {
    fontSize: 12,
    color: '#6B7280',
    marginTop: 2,
  },
  nameInput: {
    marginBottom: 8,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#374151',
    marginBottom: 8,
  },
  input: {
    borderWidth: 1,
    borderColor: '#E5E7EB',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
  },
  actions: {
    flexDirection: 'row',
    gap: 12,
  },
  button: {
    flex: 1,
  },
});
