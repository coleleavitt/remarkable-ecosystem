// app/settings.tsx
import { View, StyleSheet, ScrollView, Text, Switch, Pressable, Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useSettingsStore } from '@/stores/settingsStore';
import { clearCache, getCacheSize } from '@/services/cache';
import { Button, Card } from '@/components/ui';
import { useState, useEffect } from 'react';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export default function SettingsScreen() {
  const router = useRouter();
  const { serverConfig, settings, updateSettings, clearAll } = useSettingsStore();
  const [cacheSize, setCacheSize] = useState(0);

  useEffect(() => {
    getCacheSize().then(setCacheSize);
  }, []);

  const handleClearCache = () => {
    Alert.alert(
      'Clear Cache',
      'This will remove all cached documents. They will be re-downloaded when needed.',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Clear',
          style: 'destructive',
          onPress: async () => {
            await clearCache();
            setCacheSize(0);
          },
        },
      ]
    );
  };

  const handleDisconnect = () => {
    Alert.alert(
      'Disconnect',
      'This will remove your server connection and all cached data.',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Disconnect',
          style: 'destructive',
          onPress: async () => {
            await clearAll();
            router.replace('/connect');
          },
        },
      ]
    );
  };

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      {/* Server Section */}
      <Text style={styles.sectionTitle}>Server</Text>
      <Card style={styles.card}>
        <View style={styles.row}>
          <Ionicons name="server-outline" size={24} color="#6B7280" />
          <View style={styles.rowContent}>
            <Text style={styles.rowTitle}>
              {serverConfig?.name || 'Not connected'}
            </Text>
            <Text style={styles.rowSubtitle}>
              {serverConfig?.baseUrl || 'No server configured'}
            </Text>
          </View>
        </View>
        <Button
          title="Change Server"
          onPress={() => router.push('/connect')}
          variant="outline"
          size="small"
          style={styles.changeButton}
        />
      </Card>

      {/* Sync Section */}
      <Text style={styles.sectionTitle}>Sync</Text>
      <Card style={styles.card}>
        <View style={styles.settingRow}>
          <View style={styles.settingInfo}>
            <Text style={styles.settingTitle}>Auto Sync</Text>
            <Text style={styles.settingDescription}>
              Automatically sync when changes are detected
            </Text>
          </View>
          <Switch
            value={settings.autoSync}
            onValueChange={(value) => updateSettings({ autoSync: value })}
            trackColor={{ true: '#000' }}
            thumbColor="#fff"
          />
        </View>

        <View style={styles.divider} />

        <View style={styles.settingRow}>
          <View style={styles.settingInfo}>
            <Text style={styles.settingTitle}>Sync on WiFi Only</Text>
            <Text style={styles.settingDescription}>
              Only sync when connected to WiFi
            </Text>
          </View>
          <Switch
            value={settings.syncOnWifi}
            onValueChange={(value) => updateSettings({ syncOnWifi: value })}
            trackColor={{ true: '#000' }}
            thumbColor="#fff"
          />
        </View>
      </Card>

      {/* Notifications Section */}
      <Text style={styles.sectionTitle}>Notifications</Text>
      <Card style={styles.card}>
        <View style={styles.settingRow}>
          <View style={styles.settingInfo}>
            <Text style={styles.settingTitle}>Push Notifications</Text>
            <Text style={styles.settingDescription}>
              Get notified about sync events
            </Text>
          </View>
          <Switch
            value={settings.notificationsEnabled}
            onValueChange={(value) =>
              updateSettings({ notificationsEnabled: value })
            }
            trackColor={{ true: '#000' }}
            thumbColor="#fff"
          />
        </View>
      </Card>

      {/* Storage Section */}
      <Text style={styles.sectionTitle}>Storage</Text>
      <Card style={styles.card}>
        <View style={styles.row}>
          <Ionicons name="folder-outline" size={24} color="#6B7280" />
          <View style={styles.rowContent}>
            <Text style={styles.rowTitle}>Cache Size</Text>
            <Text style={styles.rowSubtitle}>{formatBytes(cacheSize)}</Text>
          </View>
        </View>
        <Button
          title="Clear Cache"
          onPress={handleClearCache}
          variant="outline"
          size="small"
          style={styles.changeButton}
        />
      </Card>

      {/* Danger Zone */}
      <Text style={styles.sectionTitle}>Account</Text>
      <Card style={[styles.card, styles.dangerCard]}>
        <Button
          title="Disconnect from Server"
          onPress={handleDisconnect}
          variant="danger"
        />
      </Card>

      {/* App Info */}
      <View style={styles.footer}>
        <Text style={styles.footerText}>reMarkable Sync v1.0.0</Text>
        <Text style={styles.footerSubtext}>
          Made with ❤️ for reMarkable users
        </Text>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#6B7280',
    textTransform: 'uppercase',
    marginBottom: 8,
    marginTop: 16,
    marginLeft: 4,
  },
  card: {
    padding: 16,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  rowContent: {
    marginLeft: 12,
    flex: 1,
  },
  rowTitle: {
    fontSize: 16,
    fontWeight: '500',
    color: '#111827',
  },
  rowSubtitle: {
    fontSize: 14,
    color: '#6B7280',
    marginTop: 2,
  },
  changeButton: {
    marginTop: 12,
  },
  settingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  settingInfo: {
    flex: 1,
    marginRight: 12,
  },
  settingTitle: {
    fontSize: 16,
    fontWeight: '500',
    color: '#111827',
  },
  settingDescription: {
    fontSize: 14,
    color: '#6B7280',
    marginTop: 2,
  },
  divider: {
    height: 1,
    backgroundColor: '#E5E7EB',
    marginVertical: 12,
  },
  dangerCard: {
    borderColor: '#FEE2E2',
    backgroundColor: '#FEF2F2',
  },
  footer: {
    alignItems: 'center',
    marginTop: 32,
  },
  footerText: {
    fontSize: 14,
    color: '#6B7280',
  },
  footerSubtext: {
    fontSize: 12,
    color: '#9CA3AF',
    marginTop: 4,
  },
});
