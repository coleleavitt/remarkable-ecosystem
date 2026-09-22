// app/preview/[id].tsx
import { View, StyleSheet, Text, Pressable, Share } from 'react-native';
import { useLocalSearchParams, useNavigation, useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { DocumentPreview } from '@/components/preview';
import { useQuery } from '@tanstack/react-query';
import { getAPI } from '@/services/api';
import { LoadingSpinner, ErrorState } from '@/components/ui';
import { useLayoutEffect } from 'react';

export default function PreviewScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const navigation = useNavigation();
  const router = useRouter();

  const { data: document, isLoading, error } = useQuery({
    queryKey: ['document', id],
    queryFn: async () => {
      const api = getAPI();
      return api.getDocument(id);
    },
    enabled: !!id,
  });

  useLayoutEffect(() => {
    if (document) {
      navigation.setOptions({
        title: document.visibleName,
        headerRight: () => (
          <View style={styles.headerButtons}>
            <Pressable
              style={styles.headerButton}
              onPress={handleShare}
            >
              <Ionicons name="share-outline" size={22} color="#000" />
            </Pressable>
            <Pressable
              style={styles.headerButton}
              onPress={handleOptions}
            >
              <Ionicons name="ellipsis-vertical" size={22} color="#000" />
            </Pressable>
          </View>
        ),
      });
    }
  }, [document, navigation]);

  const handleShare = async () => {
    if (!document) return;
    try {
      await Share.share({
        title: document.visibleName,
        message: `Check out "${document.visibleName}" from my reMarkable`,
      });
    } catch (error) {
      console.error('Share failed:', error);
    }
  };

  const handleOptions = () => {
    // TODO: Show options menu (rename, delete, move)
    console.log('Options pressed');
  };

  if (isLoading) {
    return <LoadingSpinner message="Loading document..." />;
  }

  if (error) {
    return (
      <ErrorState
        message={error instanceof Error ? error.message : 'Failed to load document'}
        onRetry={() => router.back()}
      />
    );
  }

  if (!id) {
    return (
      <ErrorState
        title="Missing Document"
        message="No document ID provided"
        onRetry={() => router.back()}
      />
    );
  }

  return (
    <View style={styles.container}>
      <DocumentPreview documentId={id} />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F3F4F6',
  },
  headerButtons: {
    flexDirection: 'row',
    gap: 8,
  },
  headerButton: {
    padding: 8,
  },
});
