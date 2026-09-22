// src/components/sync/SyncStatusBar.tsx
import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import Animated, {
  useAnimatedStyle,
  withRepeat,
  withSequence,
  withTiming,
  useSharedValue,
  withSpring,
} from 'react-native-reanimated';
import { useSync, useNetworkStatus } from '@/hooks';

function formatLastSync(dateStr: string | null): string {
  if (!dateStr) return 'Never synced';
  try {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    
    return date.toLocaleDateString();
  } catch {
    return 'Unknown';
  }
}

export function SyncStatusBar() {
  const { status, startSync, isSyncing } = useSync();
  const { isOnline, isWifi } = useNetworkStatus();
  
  const rotation = useSharedValue(0);
  
  React.useEffect(() => {
    if (isSyncing || status?.state === 'syncing') {
      rotation.value = withRepeat(
        withTiming(360, { duration: 1000 }),
        -1,
        false
      );
    } else {
      rotation.value = withSpring(0);
    }
  }, [isSyncing, status?.state]);

  const spinStyle = useAnimatedStyle(() => ({
    transform: [{ rotate: `${rotation.value}deg` }],
  }));

  const getStatusIcon = () => {
    if (!isOnline) return 'cloud-offline-outline';
    if (status?.state === 'syncing' || isSyncing) return 'sync';
    if (status?.state === 'error') return 'alert-circle-outline';
    return 'cloud-done-outline';
  };

  const getStatusColor = () => {
    if (!isOnline) return '#6B7280';
    if (status?.state === 'error') return '#DC2626';
    if (status?.state === 'syncing' || isSyncing) return '#2563EB';
    return '#10B981';
  };

  const hasPending = (status?.pendingUploads ?? 0) > 0;

  return (
    <Pressable
      style={styles.container}
      onPress={() => isOnline && !isSyncing && startSync()}
      disabled={!isOnline || isSyncing}
    >
      <View style={styles.left}>
        <Animated.View style={[styles.iconContainer, spinStyle]}>
          <Ionicons name={getStatusIcon()} size={20} color={getStatusColor()} />
        </Animated.View>
        
        <View style={styles.textContainer}>
          <Text style={styles.statusText}>
            {!isOnline
              ? 'Offline'
              : status?.state === 'syncing' || isSyncing
              ? 'Syncing...'
              : status?.state === 'error'
              ? 'Sync error'
              : 'Synced'}
          </Text>
          <Text style={styles.lastSyncText}>
            {formatLastSync(status?.lastSync ?? null)}
          </Text>
        </View>
      </View>

      <View style={styles.right}>
        {hasPending && (
          <View style={styles.pendingBadge}>
            <Ionicons name="cloud-upload-outline" size={14} color="#fff" />
            <Text style={styles.pendingText}>{status?.pendingUploads}</Text>
          </View>
        )}
        
        <Ionicons
          name={isWifi ? 'wifi' : 'cellular'}
          size={16}
          color={isOnline ? '#10B981' : '#6B7280'}
        />
      </View>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#E5E7EB',
  },
  left: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  iconContainer: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#F3F4F6',
    justifyContent: 'center',
    alignItems: 'center',
  },
  textContainer: {
    marginLeft: 12,
  },
  statusText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#111827',
  },
  lastSyncText: {
    fontSize: 12,
    color: '#6B7280',
    marginTop: 2,
  },
  right: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  pendingBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F59E0B',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
    gap: 4,
  },
  pendingText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#fff',
  },
});
