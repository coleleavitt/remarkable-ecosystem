// src/components/preview/PageNavigator.tsx
import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Ionicons } from '@expo/vector-icons';

interface PageNavigatorProps {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}

export function PageNavigator({
  currentPage,
  totalPages,
  onPageChange,
}: PageNavigatorProps) {
  const canGoPrev = currentPage > 0;
  const canGoNext = currentPage < totalPages - 1;

  return (
    <View style={styles.container}>
      <Pressable
        style={[styles.button, !canGoPrev && styles.buttonDisabled]}
        onPress={() => canGoPrev && onPageChange(currentPage - 1)}
        disabled={!canGoPrev}
      >
        <Ionicons
          name="chevron-back"
          size={24}
          color={canGoPrev ? '#000' : '#D1D5DB'}
        />
      </Pressable>

      <View style={styles.pageInfo}>
        <Text style={styles.pageText}>
          {currentPage + 1} / {totalPages}
        </Text>
      </View>

      <Pressable
        style={[styles.button, !canGoNext && styles.buttonDisabled]}
        onPress={() => canGoNext && onPageChange(currentPage + 1)}
        disabled={!canGoNext}
      >
        <Ionicons
          name="chevron-forward"
          size={24}
          color={canGoNext ? '#000' : '#D1D5DB'}
        />
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 12,
    paddingHorizontal: 16,
    backgroundColor: '#fff',
    borderTopWidth: 1,
    borderTopColor: '#E5E7EB',
  },
  button: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#F3F4F6',
    justifyContent: 'center',
    alignItems: 'center',
  },
  buttonDisabled: {
    opacity: 0.5,
  },
  pageInfo: {
    paddingHorizontal: 24,
  },
  pageText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#374151',
  },
});
