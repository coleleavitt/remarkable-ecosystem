// src/stores/settingsStore.ts
import { create } from 'zustand';
import AsyncStorage from '@react-native-async-storage/async-storage';
import type { ServerConfig, AuthToken } from '@/types';

const STORAGE_KEYS = {
  SERVER_CONFIG: '@remarkable/server_config',
  AUTH_TOKEN: '@remarkable/auth_token',
  SETTINGS: '@remarkable/settings',
};

interface Settings {
  autoSync: boolean;
  syncOnWifi: boolean;
  cacheSize: number; // MB
  theme: 'light' | 'dark' | 'system';
  notificationsEnabled: boolean;
}

const DEFAULT_SETTINGS: Settings = {
  autoSync: true,
  syncOnWifi: true,
  cacheSize: 500,
  theme: 'system',
  notificationsEnabled: true,
};

interface SettingsState {
  serverConfig: ServerConfig | null;
  authToken: AuthToken | null;
  settings: Settings;
  isInitialized: boolean;
  
  // Actions
  initialize: () => Promise<void>;
  setServerConfig: (config: ServerConfig) => Promise<void>;
  setAuthToken: (token: AuthToken | null) => Promise<void>;
  updateSettings: (updates: Partial<Settings>) => Promise<void>;
  clearAll: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  serverConfig: null,
  authToken: null,
  settings: DEFAULT_SETTINGS,
  isInitialized: false,

  initialize: async () => {
    try {
      const [configJson, tokenJson, settingsJson] = await Promise.all([
        AsyncStorage.getItem(STORAGE_KEYS.SERVER_CONFIG),
        AsyncStorage.getItem(STORAGE_KEYS.AUTH_TOKEN),
        AsyncStorage.getItem(STORAGE_KEYS.SETTINGS),
      ]);

      set({
        serverConfig: configJson ? JSON.parse(configJson) : null,
        authToken: tokenJson ? JSON.parse(tokenJson) : null,
        settings: settingsJson ? { ...DEFAULT_SETTINGS, ...JSON.parse(settingsJson) } : DEFAULT_SETTINGS,
        isInitialized: true,
      });
    } catch (error) {
      console.error('Failed to initialize settings:', error);
      set({ isInitialized: true });
    }
  },

  setServerConfig: async (config: ServerConfig) => {
    await AsyncStorage.setItem(STORAGE_KEYS.SERVER_CONFIG, JSON.stringify(config));
    set({ serverConfig: config });
  },

  setAuthToken: async (token: AuthToken | null) => {
    if (token) {
      await AsyncStorage.setItem(STORAGE_KEYS.AUTH_TOKEN, JSON.stringify(token));
    } else {
      await AsyncStorage.removeItem(STORAGE_KEYS.AUTH_TOKEN);
    }
    set({ authToken: token });
  },

  updateSettings: async (updates: Partial<Settings>) => {
    const { settings } = get();
    const newSettings = { ...settings, ...updates };
    await AsyncStorage.setItem(STORAGE_KEYS.SETTINGS, JSON.stringify(newSettings));
    set({ settings: newSettings });
  },

  clearAll: async () => {
    await Promise.all([
      AsyncStorage.removeItem(STORAGE_KEYS.SERVER_CONFIG),
      AsyncStorage.removeItem(STORAGE_KEYS.AUTH_TOKEN),
      AsyncStorage.removeItem(STORAGE_KEYS.SETTINGS),
    ]);
    set({
      serverConfig: null,
      authToken: null,
      settings: DEFAULT_SETTINGS,
    });
  },
}));
