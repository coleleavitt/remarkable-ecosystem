// src/components/preview/DocumentPreview.tsx
import React, { useState } from 'react';
import {
  View,
  StyleSheet,
  Dimensions,
  ScrollView,
  ActivityIndicator,
} from 'react-native';
import { SvgXml } from 'react-native-svg';
import { usePreview, usePageCount } from '@/hooks/usePreview';
import { PageNavigator } from './PageNavigator';
import { ErrorState, LoadingSpinner } from '@/components/ui';

interface DocumentPreviewProps {
  documentId: string;
  initialPage?: number;
}

const { width: SCREEN_WIDTH } = Dimensions.get('window');
const REMARKABLE_WIDTH = 1404;
const REMARKABLE_HEIGHT = 1872;
const ASPECT_RATIO = REMARKABLE_HEIGHT / REMARKABLE_WIDTH;

export function DocumentPreview({
  documentId,
  initialPage = 0,
}: DocumentPreviewProps) {
  const [currentPage, setCurrentPage] = useState(initialPage);
  const { data: pageCount } = usePageCount(documentId);
  const { data: preview, isLoading, error, refetch } = usePreview(
    documentId,
    currentPage
  );

  const contentWidth = SCREEN_WIDTH - 32;
  const contentHeight = contentWidth * ASPECT_RATIO;

  if (error) {
    return (
      <ErrorState
        message={error.message}
        onRetry={() => refetch()}
      />
    );
  }

  return (
    <View style={styles.container}>
      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
        bounces={false}
      >
        <View
          style={[
            styles.svgContainer,
            { width: contentWidth, height: contentHeight },
          ]}
        >
          {isLoading ? (
            <View style={styles.loadingOverlay}>
              <ActivityIndicator size="large" color="#000" />
            </View>
          ) : preview?.svg ? (
            <SvgXml
              xml={preview.svg}
              width={contentWidth}
              height={contentHeight}
              style={styles.svg}
            />
          ) : (
            <LoadingSpinner />
          )}
        </View>
      </ScrollView>

      <PageNavigator
        currentPage={currentPage}
        totalPages={pageCount || 1}
        onPageChange={setCurrentPage}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F3F4F6',
  },
  scrollView: {
    flex: 1,
  },
  scrollContent: {
    padding: 16,
    alignItems: 'center',
  },
  svgContainer: {
    backgroundColor: '#fff',
    borderRadius: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 8,
    elevation: 4,
    overflow: 'hidden',
    justifyContent: 'center',
    alignItems: 'center',
  },
  svg: {
    backgroundColor: '#fff',
  },
  loadingOverlay: {
    ...StyleSheet.absoluteFillObject,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: 'rgba(255, 255, 255, 0.9)',
  },
});
