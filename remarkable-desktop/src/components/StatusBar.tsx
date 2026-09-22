import { RefreshCw, Cloud, CloudOff, AlertTriangle } from 'lucide-react';
import { useSyncStatus } from '../hooks';
import api from '../api';

export function StatusBar() {
  const { status, loading, error } = useSyncStatus();

  if (loading) {
    return (
      <div className="bg-gray-100 px-4 py-2 flex items-center justify-between text-sm">
        <span>Loading...</span>
      </div>
    );
  }

  const handleSync = async () => {
    try {
      await api.startSync();
    } catch (e) {
      console.error('Sync failed:', e);
    }
  };

  const handleToggleOffline = async () => {
    if (status) {
      await api.setOfflineMode(!status.offline_mode);
    }
  };

  return (
    <div className="bg-gray-100 border-b px-4 py-2 flex items-center justify-between text-sm">
      <div className="flex items-center gap-4">
        {status?.is_syncing ? (
          <div className="flex items-center gap-2 text-blue-600">
            <RefreshCw className="w-4 h-4 animate-spin" />
            <span>Syncing...</span>
          </div>
        ) : status?.offline_mode ? (
          <div className="flex items-center gap-2 text-orange-600">
            <CloudOff className="w-4 h-4" />
            <span>Offline Mode</span>
          </div>
        ) : (
          <div className="flex items-center gap-2 text-green-600">
            <Cloud className="w-4 h-4" />
            <span>Connected</span>
          </div>
        )}

        {status && status.conflicts_count > 0 && (
          <div className="flex items-center gap-2 text-amber-600">
            <AlertTriangle className="w-4 h-4" />
            <span>{status.conflicts_count} conflict{status.conflicts_count !== 1 ? 's' : ''}</span>
          </div>
        )}

        {error && (
          <span className="text-red-600">Error: {error}</span>
        )}
      </div>

      <div className="flex items-center gap-4">
        <span className="text-gray-600">
          {status?.documents_synced ?? 0} documents synced
        </span>

        <button
          onClick={handleSync}
          disabled={status?.is_syncing}
          className="px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
        >
          <RefreshCw className="w-4 h-4" />
          Sync Now
        </button>

        <button
          onClick={handleToggleOffline}
          className={`px-3 py-1 rounded flex items-center gap-1 ${
            status?.offline_mode
              ? 'bg-orange-600 text-white hover:bg-orange-700'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
          }`}
        >
          {status?.offline_mode ? <CloudOff className="w-4 h-4" /> : <Cloud className="w-4 h-4" />}
          {status?.offline_mode ? 'Go Online' : 'Go Offline'}
        </button>
      </div>
    </div>
  );
}

export default StatusBar;
