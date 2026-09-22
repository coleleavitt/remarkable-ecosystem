// app/share.tsx
// Screen shown when files are shared to the app

import { View, StyleSheet, Text, FlatList } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { Button, Card, LoadingSpinner } from '@/components/ui';
import { useShareIntent } from '@/hooks/useShareIntent';
import { useSettingsStore } from '@/stores/settingsStore';

export default function ShareScreen() {
  const router = useRouter();
  const { serverConfig } = useSettingsStore();
  const {
    sharedFiles,
    isProcessing,
    uploadSharedFiles,
    clearSharedFiles,
    hasSharedFiles,
  } = useShareIntent();

  if (!serverConfig) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Ionicons name="alert-circle" size={48} color="#F59E0B" />
          <Text style={styles.title}>Not Connected</Text>
          <Text style={styles.subtitle}>
            Connect to a server first to upload files
          </Text>
        </View>
        <Button
          title="Connect"
          onPress={() => router.push('/connect')}
          style={styles.button}
        />
      </View>
    );
  }

  if (!hasSharedFiles) {
    return (
      <View style={styles.container}>
        <LoadingSpinner message="Processing shared files..." />
      </View>
    );
  }

  const handleUpload = async () => {
    try {
      await uploadSharedFiles();
    } catch (error) {
      console.error('Upload failed:', error);
    }
  };

  const handleCancel = () => {
    clearSharedFiles();
    router.back();
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Ionicons name="cloud-upload" size={48} color="#3B82F6" />
        <Text style={styles.title}>Upload to reMarkable</Text>
        <Text style={styles.subtitle}>
          {sharedFiles.length} file{sharedFiles.length !== 1 ? 's' : ''} ready to upload
        </Text>
      </View>

      <FlatList
        data={sharedFiles}
        keyExtractor={(item) => item.uri}
        renderItem={({ item }) => (
          <Card style={styles.fileCard}>
            <View style={styles.fileRow}>
              <Ionicons
                name={item.type.includes('epub') ? 'book' : 'document'}
                size={32}
                color="#3B82F6"
              />
              <View style={styles.fileInfo}>
                <Text style={styles.fileName} numberOfLines={2}>
                  {item.name}
                </Text>
                <Text style={styles.fileType}>
                  {item.type.includes('epub') ? 'EPUB' : 'PDF'}
                </Text>
              </View>
            </View>
          </Card>
        )}
        contentContainerStyle={styles.list}
      />

      <View style={styles.actions}>
        <Button
          title="Cancel"
          onPress={handleCancel}
          variant="outline"
          style={styles.actionButton}
          disabled={isProcessing}
        />
        <Button
          title={isProcessing ? 'Uploading...' : 'Upload'}
          onPress={handleUpload}
          style={styles.actionButton}
          loading={isProcessing}
          disabled={isProcessing}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
    padding: 16,
  },
  header: {
    alignItems: 'center',
    paddingVertical: 24,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#111827',
    marginTop: 16,
  },
  subtitle: {
    fontSize: 14,
    color: '#6B7280',
    marginTop: 8,
  },
  list: {
    paddingVertical: 8,
  },
  fileCard: {
    marginBottom: 8,
    padding: 16,
  },
  fileRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  fileInfo: {
    marginLeft: 12,
    flex: 1,
  },
  fileName: {
    fontSize: 16,
    fontWeight: '500',
    color: '#111827',
  },
  fileType: {
    fontSize: 12,
    color: '#6B7280',
    marginTop: 2,
  },
  actions: {
    flexDirection: 'row',
    gap: 12,
    paddingTop: 16,
    paddingBottom: 32,
  },
  actionButton: {
    flex: 1,
  },
  button: {
    marginTop: 16,
  },
});
