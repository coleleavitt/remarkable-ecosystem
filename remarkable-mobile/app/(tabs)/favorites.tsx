// app/(tabs)/favorites.tsx
import { View, StyleSheet, FlatList } from 'react-native';
import { useRouter } from 'expo-router';
import { DocumentItem } from '@/components/documents/DocumentItem';
import { EmptyState, LoadingSpinner } from '@/components/ui';
import { useDocuments } from '@/hooks';
import type { Document } from '@/types';

export default function FavoritesScreen() {
  const router = useRouter();
  const { documents, isLoading } = useDocuments('');

  // Get bookmarked/pinned documents
  const favorites = documents.filter(
    (doc) => doc.bookmarked || doc.pinned
  );

  const handlePress = (doc: Document) => {
    if (doc.type === 'CollectionType') {
      // Navigate back to main tab and open folder
      // For simplicity, we'll just show a preview
      router.push(`/preview/${doc.id}`);
    } else {
      router.push(`/preview/${doc.id}`);
    }
  };

  if (isLoading) {
    return <LoadingSpinner message="Loading favorites..." />;
  }

  if (favorites.length === 0) {
    return (
      <EmptyState
        icon="star-outline"
        title="No favorites"
        message="Pin documents on your reMarkable to see them here"
      />
    );
  }

  return (
    <View style={styles.container}>
      <FlatList
        data={favorites}
        keyExtractor={(item) => item.id}
        renderItem={({ item }) => (
          <DocumentItem document={item} onPress={() => handlePress(item)} />
        )}
        contentContainerStyle={styles.list}
        ItemSeparatorComponent={() => <View style={styles.separator} />}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
  list: {
    paddingVertical: 8,
  },
  separator: {
    height: 4,
  },
});
