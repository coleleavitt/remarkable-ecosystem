// src/hooks/useNetworkStatus.ts
import { useEffect, useState } from 'react';
import * as Network from 'expo-network';
import { useSyncStore } from '@/stores';

export function useNetworkStatus() {
  const [networkType, setNetworkType] = useState<string>('unknown');
  const { isOnline, setOnline } = useSyncStore();

  useEffect(() => {
    let mounted = true;

    const checkNetwork = async () => {
      try {
        const state = await Network.getNetworkStateAsync();
        if (mounted) {
          setOnline(state.isConnected ?? false);
          setNetworkType(state.type ?? 'unknown');
        }
      } catch (error) {
        if (mounted) {
          setOnline(false);
          setNetworkType('unknown');
        }
      }
    };

    checkNetwork();
    
    // Poll network status
    const interval = setInterval(checkNetwork, 10000);
    
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [setOnline]);

  return {
    isOnline,
    networkType,
    isWifi: networkType === Network.NetworkStateType.WIFI,
    isCellular: networkType === Network.NetworkStateType.CELLULAR,
  };
}
