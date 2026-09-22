import { useState, useEffect } from 'react';
import { Save, TestTube, Check, X, Folder } from 'lucide-react';
import { useSyncConfig, useFolders } from '../hooks';
import api from '../api';
import type { SyncConfig, ConflictStrategy } from '../types';

export function Settings() {
  const { config, loading, updateConfig } = useSyncConfig();
  const { folders } = useFolders();
  const [localConfig, setLocalConfig] = useState<SyncConfig | null>(null);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (config) {
      setLocalConfig(config);
    }
  }, [config]);

  if (loading || !localConfig) {
    return <div className="p-4">Loading settings...</div>;
  }

  const handleSave = async () => {
    setSaving(true);
    try {
      await updateConfig(localConfig);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error('Failed to save:', e);
    } finally {
      setSaving(false);
    }
  };

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await api.testConnection();
      setTestResult(result);
    } catch (e) {
      setTestResult(false);
    } finally {
      setTesting(false);
    }
  };

  const handleFolderToggle = (folderId: string) => {
    const selected = localConfig.selected_folders.includes(folderId)
      ? localConfig.selected_folders.filter(id => id !== folderId)
      : [...localConfig.selected_folders, folderId];
    setLocalConfig({ ...localConfig, selected_folders: selected });
  };

  return (
    <div className="p-6 max-w-2xl">
      <h2 className="text-2xl font-bold mb-6">Settings</h2>

      {/* Server Configuration */}
      <section className="mb-8">
        <h3 className="text-lg font-semibold mb-4 text-gray-700">Server</h3>
        
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-600 mb-1">
              Server URL
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={localConfig.server_url}
                onChange={e => setLocalConfig({ ...localConfig, server_url: e.target.value })}
                className="flex-1 px-3 py-2 border rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="https://tectonic.remarkable.com"
              />
              <button
                onClick={handleTest}
                disabled={testing}
                className="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 flex items-center gap-2"
              >
                <TestTube className="w-4 h-4" />
                Test
              </button>
            </div>
            {testResult !== null && (
              <div className={`mt-2 flex items-center gap-2 text-sm ${testResult ? 'text-green-600' : 'text-red-600'}`}>
                {testResult ? <Check className="w-4 h-4" /> : <X className="w-4 h-4" />}
                {testResult ? 'Connection successful' : 'Connection failed'}
              </div>
            )}
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-600 mb-1">
              Device Token
            </label>
            <input
              type="password"
              value={localConfig.device_token || ''}
              onChange={e => setLocalConfig({ ...localConfig, device_token: e.target.value || null })}
              className="w-full px-3 py-2 border rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="Optional: Device authentication token"
            />
          </div>
        </div>
      </section>

      {/* Sync Configuration */}
      <section className="mb-8">
        <h3 className="text-lg font-semibold mb-4 text-gray-700">Sync Settings</h3>
        
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-600 mb-1">
              Sync Interval (seconds)
            </label>
            <input
              type="number"
              min="60"
              max="86400"
              value={localConfig.sync_interval_secs}
              onChange={e => setLocalConfig({ ...localConfig, sync_interval_secs: parseInt(e.target.value) || 300 })}
              className="w-32 px-3 py-2 border rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
            <span className="ml-2 text-sm text-gray-500">
              ({Math.round(localConfig.sync_interval_secs / 60)} minutes)
            </span>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-600 mb-1">
              Local Storage Path
            </label>
            <input
              type="text"
              value={localConfig.local_path}
              onChange={e => setLocalConfig({ ...localConfig, local_path: e.target.value })}
              className="w-full px-3 py-2 border rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-600 mb-1">
              Conflict Resolution
            </label>
            <select
              value={localConfig.conflict_strategy}
              onChange={e => setLocalConfig({ ...localConfig, conflict_strategy: e.target.value as ConflictStrategy })}
              className="px-3 py-2 border rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="ask_user">Ask me each time</option>
              <option value="keep_local">Keep local version</option>
              <option value="keep_remote">Keep remote version</option>
              <option value="keep_both">Keep both versions</option>
            </select>
          </div>

          <div className="flex items-center gap-6">
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={localConfig.auto_start}
                onChange={e => setLocalConfig({ ...localConfig, auto_start: e.target.checked })}
                className="rounded border-gray-300"
              />
              <span className="text-sm text-gray-700">Auto-start sync on launch</span>
            </label>

            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={localConfig.notifications_enabled}
                onChange={e => setLocalConfig({ ...localConfig, notifications_enabled: e.target.checked })}
                className="rounded border-gray-300"
              />
              <span className="text-sm text-gray-700">Show notifications</span>
            </label>
          </div>
        </div>
      </section>

      {/* Folder Selection */}
      <section className="mb-8">
        <h3 className="text-lg font-semibold mb-4 text-gray-700">
          Sync Folders
          <span className="ml-2 text-sm font-normal text-gray-500">
            (Leave empty to sync all)
          </span>
        </h3>
        
        <div className="border rounded max-h-64 overflow-auto">
          {folders.length === 0 ? (
            <div className="p-4 text-gray-500 text-center">
              No folders available. Run a sync first.
            </div>
          ) : (
            folders.map(folder => (
              <label
                key={folder.id}
                className="flex items-center gap-3 px-4 py-2 hover:bg-gray-50 cursor-pointer border-b border-gray-100 last:border-0"
              >
                <input
                  type="checkbox"
                  checked={localConfig.selected_folders.includes(folder.id)}
                  onChange={() => handleFolderToggle(folder.id)}
                  className="rounded border-gray-300"
                />
                <Folder className="w-4 h-4 text-amber-500" />
                <span>{folder.name}</span>
              </label>
            ))
          )}
        </div>
      </section>

      {/* Save Button */}
      <div className="flex items-center gap-4">
        <button
          onClick={handleSave}
          disabled={saving}
          className="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 flex items-center gap-2"
        >
          <Save className="w-4 h-4" />
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
        
        {saved && (
          <span className="text-green-600 flex items-center gap-1">
            <Check className="w-4 h-4" />
            Saved!
          </span>
        )}
      </div>
    </div>
  );
}

export default Settings;
