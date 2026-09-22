// src/components/documents/DocumentItem.tsx
import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { Document } from '@/types';

interface DocumentItemProps {
  document: Document;
  onPress: () => void;
  onLongPress?: () => void;
  selected?: boolean;
}

function getIcon(type: Document['type']): keyof typeof Ionicons.glyphMap {
  switch (type) {
    case 'CollectionType':
      return 'folder';
    case 'NotebookType':
      return 'document-text';
    case 'DocumentType':
      return 'document';
    default:
      return 'document-outline';
  }
}

function getIconColor(type: Document['type']): string {
  switch (type) {
    case 'CollectionType':
      return '#F59E0B';
    case 'NotebookType':
      return '#3B82F6';
    case 'DocumentType':
      return '#6B7280';
    default:
      return '#9CA3AF';
  }
}

function formatDate(dateStr: string | undefined): string {
  if (!dateStr) return '';
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  } catch {
    return '';
  }
}

export function DocumentItem({
  document,
  onPress,
  onLongPress,
  selected = false,
}: DocumentItemProps) {
  const isFolder = document.type === 'CollectionType';
  
  return (
    <Pressable
      style={({ pressed }) => [
        styles.container,
        selected && styles.selected,
        pressed && styles.pressed,
      ]}
      onPress={onPress}
      onLongPress={onLongPress}
    >
      <View style={styles.iconContainer}>
        <Ionicons
          name={getIcon(document.type)}
          size={32}
          color={getIconColor(document.type)}
        />
        {document.pinned && (
          <View style={styles.pinBadge}>
            <Ionicons name="pin" size={10} color="#000" />
          </View>
        )}
      </View>
      
      <View style={styles.content}>
        <Text style={styles.name} numberOfLines={2}>
          {document.visibleName}
        </Text>
        <View style={styles.meta}>
          {!isFolder && document.pageCount && (
            <Text style={styles.metaText}>
              {document.pageCount} {document.pageCount === 1 ? 'page' : 'pages'}
            </Text>
          )}
          {document.lastModified && (
            <Text style={styles.metaText}>
              {formatDate(document.lastModified)}
            </Text>
          )}
        </View>
      </View>
      
      <Ionicons
        name={isFolder ? 'chevron-forward' : 'ellipsis-vertical'}
        size={20}
        color="#9CA3AF"
      />
    </Pressable>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#fff',
    borderRadius: 12,
    marginHorizontal: 16,
    marginVertical: 4,
  },
  selected: {
    backgroundColor: '#F3F4F6',
    borderWidth: 2,
    borderColor: '#000',
  },
  pressed: {
    backgroundColor: '#F9FAFB',
  },
  iconContainer: {
    width: 48,
    height: 48,
    justifyContent: 'center',
    alignItems: 'center',
    position: 'relative',
  },
  pinBadge: {
    position: 'absolute',
    top: 0,
    right: 0,
    backgroundColor: '#FEF3C7',
    borderRadius: 6,
    padding: 2,
  },
  content: {
    flex: 1,
    marginLeft: 12,
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#111827',
  },
  meta: {
    flexDirection: 'row',
    marginTop: 4,
    gap: 8,
  },
  metaText: {
    fontSize: 12,
    color: '#6B7280',
  },
});
