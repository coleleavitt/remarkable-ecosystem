// app/_layout.tsx
import { useEffect } from 'react';
import { Stack } from 'expo-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { StatusBar } from 'expo-status-bar';
import * as SplashScreen from 'expo-splash-screen';
import { useSettingsStore } from '@/stores/settingsStore';
import { initializeAPI } from '@/services/api';
import { initializeCache } from '@/services/cache';
import {
  registerForPushNotifications,
  addNotificationResponseListener,
} from '@/services/notifications';
import { useRouter } from 'expo-router';

// Keep splash screen visible while loading
SplashScreen.preventAutoHideAsync();

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      staleTime: 30000,
    },
  },
});

export default function RootLayout() {
  const router = useRouter();
  const { initialize, serverConfig, authToken, isInitialized } = useSettingsStore();

  useEffect(() => {
    async function setup() {
      try {
        // Initialize settings from storage
        await initialize();
        
        // Initialize SQLite cache
        await initializeCache();
        
        // Register for push notifications
        await registerForPushNotifications();
        
        // Handle notification taps
        const subscription = addNotificationResponseListener((response) => {
          const data = response.notification.request.content.data;
          if (data.documentId) {
            router.push(`/preview/${data.documentId}`);
          }
        });

        return () => subscription.remove();
      } catch (error) {
        console.error('Setup failed:', error);
      } finally {
        SplashScreen.hideAsync();
      }
    }
    
    setup();
  }, []);

  useEffect(() => {
    // Initialize API when config is available
    if (serverConfig) {
      const api = initializeAPI(serverConfig);
      if (authToken) {
        api.setToken(authToken);
      }
    }
  }, [serverConfig, authToken]);

  return (
    <QueryClientProvider client={queryClient}>
      <StatusBar style="auto" />
      <Stack
        screenOptions={{
          headerStyle: { backgroundColor: '#fff' },
          headerTintColor: '#000',
          headerBackTitleVisible: false,
          contentStyle: { backgroundColor: '#F9FAFB' },
        }}
      >
        <Stack.Screen
          name="(tabs)"
          options={{ headerShown: false }}
        />
        <Stack.Screen
          name="preview/[id]"
          options={{
            title: 'Preview',
            presentation: 'card',
          }}
        />
        <Stack.Screen
          name="settings"
          options={{
            title: 'Settings',
            presentation: 'modal',
          }}
        />
        <Stack.Screen
          name="connect"
          options={{
            title: 'Connect to Server',
            presentation: 'fullScreenModal',
          }}
        />
      </Stack>
    </QueryClientProvider>
  );
}
