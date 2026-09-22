// app/(tabs)/recent.tsx
import { View, StyleSheet, FlatList, Text } from 'react-native';
import { useRouter } from 'expo-router';
import { DocumentItem } from '@/components/documents/DocumentItem';
import { EmptyState, LoadingSpinner } from '@/components/ui';
import { useDocuments } from '@/hooks';
import type { Document } from '@/types';

export default function RecentScreen() {
  const router = useRouter();
  const { documents, isLoading } = useDocuments('');

  // Get all documents sorted by last modified (most recent first)
  const recentDocs = [...documents]
    .filter((doc) => doc.type !== 'CollectionType')
    .sort((a, b) => {
      const dateA = a.lastModified ? new Date(a.lastModified).getTime() : 0;
      const dateB = b.lastModified ? new Date(b.lastModified).getTime() : 0;
      return dateB - dateA;
    })
    .slice(0, 20);

  const handlePress = (doc: Document) => {
    router.push(`/preview/${doc.id}`);
  };

  if (isLoading) {
    return <LoadingSpinner message="Loading recent documents..." />;
  }

  if (recentDocs.length === 0) {
    return (
      <EmptyState
        icon="time-outline"
        title="No recent documents"
        message="Your recently viewed documents will appear here"
      />
    );
  }

  return (
    <View style={styles.container}>
      <FlatList
        data={recentDocs}
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
