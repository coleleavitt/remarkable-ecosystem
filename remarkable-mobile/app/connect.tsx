// app/connect.tsx
import { useState } from 'react';
import {
  View,
  StyleSheet,
  Text,
  TextInput,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { Button, Card } from '@/components/ui';
import { useSettingsStore } from '@/stores/settingsStore';
import { initializeAPI } from '@/services/api';
import type { ServerConfig, AuthToken } from '@/types';

const PRESET_SERVERS: ServerConfig[] = [
  {
    baseUrl: 'http://10.11.99.1:8080',
    name: 'reMarkable (USB)',
    isCloud: false,
  },
  {
    baseUrl: 'http://remarkable.local:8080',
    name: 'Local Server (WiFi)',
    isCloud: false,
  },
];

export default function ConnectScreen() {
  const router = useRouter();
  const { setServerConfig, setAuthToken } = useSettingsStore();
  
  const [serverUrl, setServerUrl] = useState('');
  const [serverName, setServerName] = useState('');
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleConnect = async (config: ServerConfig) => {
    setIsConnecting(true);
    setError(null);

    try {
      // Initialize API with config
      const api = initializeAPI(config);
      
      // Test connection
      await api.getSyncRoot();
      
      // Save config
      await setServerConfig(config);
      
      // Navigate to main app
      router.replace('/(tabs)');
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Connection failed';
      setError(message);
      Alert.alert('Connection Failed', message);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleCustomConnect = () => {
    if (!serverUrl.trim()) {
      setError('Please enter a server URL');
      return;
    }

    const config: ServerConfig = {
      baseUrl: serverUrl.trim(),
      name: serverName.trim() || 'Custom Server',
      isCloud: false,
    };

    handleConnect(config);
  };

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
    >
      <ScrollView
        style={styles.scroll}
        contentContainerStyle={styles.content}
        keyboardShouldPersistTaps="handled"
      >
        <View style={styles.header}>
          <Ionicons name="cloud" size={64} color="#000" />
          <Text style={styles.title}>Connect to Server</Text>
          <Text style={styles.subtitle}>
            Choose a preset or enter a custom server URL
          </Text>
        </View>

        <Text style={styles.sectionTitle}>Quick Connect</Text>
        {PRESET_SERVERS.map((server) => (
          <Card
            key={server.baseUrl}
            style={styles.presetCard}
            onPress={() => handleConnect(server)}
          >
            <View style={styles.presetContent}>
              <Ionicons
                name={server.name.includes('USB') ? 'hardware-chip' : 'wifi'}
                size={24}
                color="#3B82F6"
              />
              <View style={styles.presetText}>
                <Text style={styles.presetName}>{server.name}</Text>
                <Text style={styles.presetUrl}>{server.baseUrl}</Text>
              </View>
              <Ionicons name="chevron-forward" size={20} color="#9CA3AF" />
            </View>
          </Card>
        ))}

        <View style={styles.divider}>
          <View style={styles.dividerLine} />
          <Text style={styles.dividerText}>or</Text>
          <View style={styles.dividerLine} />
        </View>

        <Text style={styles.sectionTitle}>Custom Server</Text>
        <Card style={styles.customCard}>
          <Text style={styles.label}>Server URL</Text>
          <TextInput
            style={styles.input}
            value={serverUrl}
            onChangeText={setServerUrl}
            placeholder="http://192.168.1.100:8080"
            autoCapitalize="none"
            autoCorrect={false}
            keyboardType="url"
          />

          <Text style={styles.label}>Server Name (optional)</Text>
          <TextInput
            style={styles.input}
            value={serverName}
            onChangeText={setServerName}
            placeholder="My Server"
            autoCapitalize="words"
          />

          <Button
            title={isConnecting ? 'Connecting...' : 'Connect'}
            onPress={handleCustomConnect}
            loading={isConnecting}
            disabled={isConnecting}
            style={styles.connectButton}
          />
        </Card>

        {error && (
          <View style={styles.errorContainer}>
            <Ionicons name="alert-circle" size={20} color="#DC2626" />
            <Text style={styles.errorText}>{error}</Text>
          </View>
        )}
      </ScrollView>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
  scroll: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  header: {
    alignItems: 'center',
    paddingVertical: 32,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#111827',
    marginTop: 16,
  },
  subtitle: {
    fontSize: 14,
    color: '#6B7280',
    marginTop: 8,
    textAlign: 'center',
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#6B7280',
    textTransform: 'uppercase',
    marginBottom: 12,
    marginTop: 8,
  },
  presetCard: {
    marginBottom: 8,
    padding: 16,
  },
  presetContent: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  presetText: {
    flex: 1,
    marginLeft: 12,
  },
  presetName: {
    fontSize: 16,
    fontWeight: '500',
    color: '#111827',
  },
  presetUrl: {
    fontSize: 12,
    color: '#6B7280',
    marginTop: 2,
  },
  divider: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 24,
  },
  dividerLine: {
    flex: 1,
    height: 1,
    backgroundColor: '#E5E7EB',
  },
  dividerText: {
    color: '#9CA3AF',
    paddingHorizontal: 16,
    fontSize: 14,
  },
  customCard: {
    padding: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#374151',
    marginBottom: 8,
  },
  input: {
    borderWidth: 1,
    borderColor: '#E5E7EB',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    marginBottom: 16,
    backgroundColor: '#fff',
  },
  connectButton: {
    marginTop: 8,
  },
  errorContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FEF2F2',
    padding: 12,
    borderRadius: 8,
    marginTop: 16,
  },
  errorText: {
    color: '#DC2626',
    marginLeft: 8,
    flex: 1,
  },
});
