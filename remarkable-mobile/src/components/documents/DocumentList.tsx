// src/components/documents/DocumentList.tsx
import React from 'react';
import { FlatList, StyleSheet, RefreshControl, View } from 'react-native';
import { DocumentItem } from './DocumentItem';
import { EmptyState, LoadingSpinner, ErrorState } from '@/components/ui';
import type { Document } from '@/types';

interface DocumentListProps {
  documents: Document[];
  isLoading: boolean;
  error?: Error | null;
  onDocumentPress: (doc: Document) => void;
  onDocumentLongPress?: (doc: Document) => void;
  onRefresh?: () => void;
  selectedIds?: Set<string>;
  ListHeaderComponent?: React.ReactElement;
}

export function DocumentList({
  documents,
  isLoading,
  error,
  onDocumentPress,
  onDocumentLongPress,
  onRefresh,
  selectedIds = new Set(),
  ListHeaderComponent,
}: DocumentListProps) {
  // Sort: folders first, then by name
  const sortedDocs = [...documents].sort((a, b) => {
    if (a.type === 'CollectionType' && b.type !== 'CollectionType') return -1;
    if (a.type !== 'CollectionType' && b.type === 'CollectionType') return 1;
    if (a.pinned && !b.pinned) return -1;
    if (!a.pinned && b.pinned) return 1;
    return a.visibleName.localeCompare(b.visibleName);
  });

  if (isLoading && documents.length === 0) {
    return <LoadingSpinner message="Loading documents..." />;
  }

  if (error && documents.length === 0) {
    return (
      <ErrorState
        message={error.message}
        onRetry={onRefresh}
      />
    );
  }

  if (documents.length === 0) {
    return (
      <EmptyState
        icon="folder-open-outline"
        title="No documents"
        message="Upload a PDF or EPUB to get started"
      />
    );
  }

  return (
    <FlatList
      data={sortedDocs}
      keyExtractor={(item) => item.id}
      renderItem={({ item }) => (
        <DocumentItem
          document={item}
          onPress={() => onDocumentPress(item)}
          onLongPress={() => onDocumentLongPress?.(item)}
          selected={selectedIds.has(item.id)}
        />
      )}
      contentContainerStyle={styles.list}
      ListHeaderComponent={ListHeaderComponent}
      refreshControl={
        onRefresh ? (
          <RefreshControl refreshing={isLoading} onRefresh={onRefresh} />
        ) : undefined
      }
      ItemSeparatorComponent={() => <View style={styles.separator} />}
    />
  );
}

const styles = StyleSheet.create({
  list: {
    paddingVertical: 8,
  },
  separator: {
    height: 4,
  },
});
