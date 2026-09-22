// src/components/documents/DocumentBrowser.tsx
import React, { useState, useCallback } from 'react';
import { View, StyleSheet, Text, Pressable } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import { DocumentList } from './DocumentList';
import { useDocuments, useUpload } from '@/hooks';
import { useDocumentStore } from '@/stores';
import { Button } from '@/components/ui';
import type { Document } from '@/types';

export function DocumentBrowser() {
  const router = useRouter();
  const { currentFolder, folderPath, navigateToFolder, navigateUp } = useDocumentStore();
  const { documents, isLoading, error, refetch } = useDocuments(currentFolder);
  const { pickAndUpload, isUploading } = useUpload(currentFolder);
  const [isSelecting, setIsSelecting] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const handleDocumentPress = useCallback((doc: Document) => {
    if (isSelecting) {
      const newSelected = new Set(selectedIds);
      if (newSelected.has(doc.id)) {
        newSelected.delete(doc.id);
      } else {
        newSelected.add(doc.id);
      }
      setSelectedIds(newSelected);
      return;
    }

    if (doc.type === 'CollectionType') {
      navigateToFolder(doc.id, doc.visibleName);
    } else {
      router.push(`/preview/${doc.id}`);
    }
  }, [isSelecting, selectedIds, navigateToFolder, router]);

  const handleDocumentLongPress = useCallback((doc: Document) => {
    if (!isSelecting) {
      setIsSelecting(true);
      setSelectedIds(new Set([doc.id]));
    }
  }, [isSelecting]);

  const cancelSelection = () => {
    setIsSelecting(false);
    setSelectedIds(new Set());
  };

  const handleUpload = async () => {
    try {
      await pickAndUpload();
      refetch();
    } catch (error) {
      console.error('Upload failed:', error);
    }
  };

  const breadcrumbHeader = (
    <View style={styles.breadcrumb}>
      {folderPath.map((item, index) => (
        <View key={item.id} style={styles.breadcrumbItem}>
          {index > 0 && (
            <Ionicons name="chevron-forward" size={14} color="#9CA3AF" />
          )}
          <Pressable
            onPress={() => {
              if (index === folderPath.length - 1) return;
              // Navigate to this folder
              const targetPath = folderPath.slice(0, index + 1);
              useDocumentStore.setState({ folderPath: targetPath });
            }}
          >
            <Text
              style={[
                styles.breadcrumbText,
                index === folderPath.length - 1 && styles.breadcrumbActive,
              ]}
            >
              {item.name}
            </Text>
          </Pressable>
        </View>
      ))}
    </View>
  );

  return (
    <View style={styles.container}>
      {isSelecting && (
        <View style={styles.selectionBar}>
          <Button
            title="Cancel"
            onPress={cancelSelection}
            variant="outline"
            size="small"
          />
          <Text style={styles.selectionCount}>
            {selectedIds.size} selected
          </Text>
        </View>
      )}
      
      <DocumentList
        documents={documents}
        isLoading={isLoading}
        error={error}
        onDocumentPress={handleDocumentPress}
        onDocumentLongPress={handleDocumentLongPress}
        onRefresh={refetch}
        selectedIds={selectedIds}
        ListHeaderComponent={breadcrumbHeader}
      />
      
      {!isSelecting && (
        <View style={styles.fab}>
          <Pressable
            style={styles.fabButton}
            onPress={handleUpload}
            disabled={isUploading}
          >
            <Ionicons name="add" size={28} color="#fff" />
          </Pressable>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
  breadcrumb: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    flexWrap: 'wrap',
  },
  breadcrumbItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  breadcrumbText: {
    fontSize: 14,
    color: '#6B7280',
    paddingHorizontal: 4,
  },
  breadcrumbActive: {
    color: '#111827',
    fontWeight: '600',
  },
  selectionBar: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#E5E7EB',
  },
  selectionCount: {
    marginLeft: 12,
    fontSize: 14,
    color: '#374151',
  },
  fab: {
    position: 'absolute',
    right: 16,
    bottom: 16,
  },
  fabButton: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#000',
    justifyContent: 'center',
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.3,
    shadowRadius: 8,
    elevation: 8,
  },
});
