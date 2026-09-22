import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { sendNotification } from '@tauri-apps/plugin-notification';

import { StatusBar } from './components/StatusBar';
import { Sidebar } from './components/Sidebar';
import { DocumentList } from './components/DocumentList';
import { ConflictResolver } from './components/ConflictResolver';
import { Settings } from './components/Settings';
import { About } from './components/About';
import api from './api';

import './App.css';

function App() {
  const [currentView, setCurrentView] = useState('documents');

  useEffect(() => {
    // Listen for tray menu events
    const unlistenSync = listen('trigger-sync', async () => {
      try {
        await api.startSync();
      } catch (e) {
        console.error('Sync failed:', e);
      }
    });

    const unlistenOffline = listen('toggle-offline', async () => {
      try {
        const current = await api.getOfflineMode();
        await api.setOfflineMode(!current);
      } catch (e) {
        console.error('Toggle offline failed:', e);
      }
    });

    const unlistenNotification = listen<string>('show-notification', async (event) => {
      try {
        await sendNotification({
          title: 'reMarkable Sync',
          body: event.payload,
        });
      } catch (e) {
        console.warn('Notification failed:', e);
      }
    });

    return () => {
      unlistenSync.then(fn => fn());
      unlistenOffline.then(fn => fn());
      unlistenNotification.then(fn => fn());
    };
  }, []);

  const renderContent = () => {
    switch (currentView) {
      case 'documents':
        return <DocumentList />;
      case 'conflicts':
        return <ConflictResolver />;
      case 'settings':
        return <Settings />;
      case 'about':
        return <About />;
      default:
        return <DocumentList />;
    }
  };

  return (
    <div className="h-screen flex flex-col bg-white">
      <StatusBar />
      
      <div className="flex-1 flex overflow-hidden">
        <Sidebar currentView={currentView} onViewChange={setCurrentView} />
        
        <main className="flex-1 overflow-auto">
          {renderContent()}
        </main>
      </div>
    </div>
  );
}

export default App;
