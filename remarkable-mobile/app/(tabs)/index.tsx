// app/(tabs)/index.tsx
import { View, StyleSheet } from 'react-native';
import { useRouter } from 'expo-router';
import { DocumentBrowser } from '@/components/documents';
import { SyncStatusBar } from '@/components/sync';
import { useSettingsStore } from '@/stores/settingsStore';
import { Button, EmptyState } from '@/components/ui';
import { useEffect } from 'react';

export default function DocumentsScreen() {
  const router = useRouter();
  const { serverConfig, isInitialized } = useSettingsStore();

  useEffect(() => {
    // Redirect to connect screen if no server configured
    if (isInitialized && !serverConfig) {
      router.replace('/connect');
    }
  }, [isInitialized, serverConfig]);

  if (!serverConfig) {
    return (
      <EmptyState
        icon="cloud-offline-outline"
        title="Not Connected"
        message="Connect to a reMarkable server to sync your documents"
        actionTitle="Connect"
        onAction={() => router.push('/connect')}
      />
    );
  }

  return (
    <View style={styles.container}>
      <SyncStatusBar />
      <DocumentBrowser />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
});
