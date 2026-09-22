// src/services/notifications.ts
// Push notification service for sync events

import * as Notifications from 'expo-notifications';
import * as Device from 'expo-device';
import Constants from 'expo-constants';
import { Platform } from 'react-native';
import type { SyncNotification } from '@/types';

// Configure notification behavior
Notifications.setNotificationHandler({
  handleNotification: async () => ({
    shouldShowAlert: true,
    shouldPlaySound: true,
    shouldSetBadge: true,
    shouldShowBanner: true,
    shouldShowList: true,
  }),
});

export async function registerForPushNotifications(): Promise<string | null> {
  if (!Device.isDevice) {
    console.log('Push notifications require a physical device');
    return null;
  }

  const { status: existingStatus } = await Notifications.getPermissionsAsync();
  let finalStatus = existingStatus;

  if (existingStatus !== 'granted') {
    const { status } = await Notifications.requestPermissionsAsync();
    finalStatus = status;
  }

  if (finalStatus !== 'granted') {
    console.log('Failed to get push notification permissions');
    return null;
  }

  try {
    const projectId = Constants.expoConfig?.extra?.eas?.projectId;
    const token = await Notifications.getExpoPushTokenAsync({ projectId });
    return token.data;
  } catch (error) {
    console.error('Failed to get push token:', error);
    return null;
  }
}

export async function sendLocalNotification(notification: SyncNotification): Promise<void> {
  const title = getNotificationTitle(notification.type);
  
  await Notifications.scheduleNotificationAsync({
    content: {
      title,
      body: notification.message,
      data: {
        type: notification.type,
        documentId: notification.documentId,
      },
      sound: true,
    },
    trigger: null, // Immediate
  });
}

function getNotificationTitle(type: SyncNotification['type']): string {
  switch (type) {
    case 'sync_complete':
      return 'Sync Complete';
    case 'sync_error':
      return 'Sync Error';
    case 'document_added':
      return 'New Document';
    case 'document_modified':
      return 'Document Updated';
    default:
      return 'reMarkable Sync';
  }
}

export function addNotificationReceivedListener(
  callback: (notification: Notifications.Notification) => void
): Notifications.EventSubscription {
  return Notifications.addNotificationReceivedListener(callback);
}

export function addNotificationResponseListener(
  callback: (response: Notifications.NotificationResponse) => void
): Notifications.EventSubscription {
  return Notifications.addNotificationResponseReceivedListener(callback);
}

// Android notification channel setup
if (Platform.OS === 'android') {
  Notifications.setNotificationChannelAsync('sync', {
    name: 'Sync Updates',
    importance: Notifications.AndroidImportance.DEFAULT,
    vibrationPattern: [0, 250, 250, 250],
    lightColor: '#000000',
  });
}
