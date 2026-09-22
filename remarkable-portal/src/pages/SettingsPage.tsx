import { useState } from 'react';
import { 
  User, 
  Cloud, 
  HardDrive, 
  Shield, 
  Bell, 
  Moon, 
  Sun,
  CheckCircle,
  XCircle,
  RefreshCw,
  Loader2
} from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { Header } from '../components/layout/Header';
import { Button } from '../components/ui/Button';

import { useAuth } from '../contexts/AuthContext';

export function SettingsPage() {
  const { user, refreshUser } = useAuth();
  const [syncing, setSyncing] = useState(false);
  const [darkMode, setDarkMode] = useState(false);
  
  const handleSync = async () => {
    setSyncing(true);
    // Simulate sync
    await new Promise(r => setTimeout(r, 2000));
    await refreshUser();
    setSyncing(false);
  };
  
  const storageUsedPercent = user 
    ? Math.round((user.storageUsed / user.storageLimit) * 100) 
    : 0;
  
  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  };
  
  return (
    <div className="min-h-screen">
      <Header 
        title="Settings" 
        subtitle="Manage your account and preferences"
        showCreateFolder={false}
      />
      
      <div className="p-6 max-w-3xl mx-auto space-y-6">
        {/* Account section */}
        <section className="card p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-lg bg-accent/10 flex items-center justify-center">
              <User className="h-5 w-5 text-accent" />
            </div>
            <div>
              <h2 className="font-semibold text-text">Account</h2>
              <p className="text-sm text-text-muted">Your profile information</p>
            </div>
          </div>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between py-3 border-b border-border">
              <div>
                <p className="text-sm font-medium text-text">Email</p>
                <p className="text-sm text-text-muted">{user?.email || 'Not set'}</p>
              </div>
            </div>
            
            <div className="flex items-center justify-between py-3 border-b border-border">
              <div>
                <p className="text-sm font-medium text-text">Name</p>
                <p className="text-sm text-text-muted">{user?.name || 'Not set'}</p>
              </div>
              <Button variant="ghost" size="sm">Edit</Button>
            </div>
            
            <div className="flex items-center justify-between py-3">
              <div>
                <p className="text-sm font-medium text-text">Member since</p>
                <p className="text-sm text-text-muted">
                  {user?.createdAt 
                    ? formatDistanceToNow(new Date(user.createdAt), { addSuffix: true })
                    : 'Unknown'}
                </p>
              </div>
            </div>
          </div>
        </section>
        
        {/* Sync section */}
        <section className="card p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-lg bg-blue-100 flex items-center justify-center">
              <Cloud className="h-5 w-5 text-blue-600" />
            </div>
            <div>
              <h2 className="font-semibold text-text">Sync Status</h2>
              <p className="text-sm text-text-muted">Cloud synchronization</p>
            </div>
          </div>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-surface-dim rounded-lg">
              <div className="flex items-center gap-3">
                {user?.syncStatus === 'synced' ? (
                  <CheckCircle className="h-5 w-5 text-success" />
                ) : user?.syncStatus === 'syncing' ? (
                  <RefreshCw className="h-5 w-5 text-accent animate-spin" />
                ) : (
                  <XCircle className="h-5 w-5 text-text-subtle" />
                )}
                <div>
                  <p className="text-sm font-medium text-text capitalize">
                    {user?.syncStatus || 'Unknown'}
                  </p>
                  {user?.lastSyncTime && (
                    <p className="text-xs text-text-muted">
                      Last synced {formatDistanceToNow(new Date(user.lastSyncTime), { addSuffix: true })}
                    </p>
                  )}
                </div>
              </div>
              <Button 
                variant="secondary" 
                size="sm" 
                onClick={handleSync}
                disabled={syncing}
              >
                {syncing ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <RefreshCw className="h-4 w-4" />
                )}
                Sync Now
              </Button>
            </div>
          </div>
        </section>
        
        {/* Storage section */}
        <section className="card p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-lg bg-amber-100 flex items-center justify-center">
              <HardDrive className="h-5 w-5 text-amber-600" />
            </div>
            <div>
              <h2 className="font-semibold text-text">Storage</h2>
              <p className="text-sm text-text-muted">Your storage usage</p>
            </div>
          </div>
          
          <div className="space-y-4">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-text">
                  {formatBytes(user?.storageUsed || 0)} used
                </span>
                <span className="text-sm text-text-muted">
                  {formatBytes(user?.storageLimit || 8 * 1024 * 1024 * 1024)} total
                </span>
              </div>
              <div className="h-2 bg-surface-dim rounded-full overflow-hidden">
                <div 
                  className="h-full bg-accent transition-all"
                  style={{ width: `${storageUsedPercent}%` }}
                />
              </div>
              <p className="text-xs text-text-muted mt-2">
                {100 - storageUsedPercent}% available
              </p>
            </div>
          </div>
        </section>
        
        {/* Preferences section */}
        <section className="card p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-lg bg-purple-100 flex items-center justify-center">
              <Bell className="h-5 w-5 text-purple-600" />
            </div>
            <div>
              <h2 className="font-semibold text-text">Preferences</h2>
              <p className="text-sm text-text-muted">Customize your experience</p>
            </div>
          </div>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between py-3 border-b border-border">
              <div>
                <p className="text-sm font-medium text-text">Dark Mode</p>
                <p className="text-sm text-text-muted">Use dark theme</p>
              </div>
              <button
                onClick={() => setDarkMode(!darkMode)}
                className={`
                  relative w-12 h-6 rounded-full transition-colors
                  ${darkMode ? 'bg-accent' : 'bg-surface-dim border border-border-strong'}
                `}
              >
                <div className={`
                  absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform flex items-center justify-center
                  ${darkMode ? 'translate-x-6' : 'translate-x-0.5'}
                `}>
                  {darkMode ? (
                    <Moon className="h-3 w-3 text-accent" />
                  ) : (
                    <Sun className="h-3 w-3 text-text-muted" />
                  )}
                </div>
              </button>
            </div>
            
            <div className="flex items-center justify-between py-3">
              <div>
                <p className="text-sm font-medium text-text">Notifications</p>
                <p className="text-sm text-text-muted">Email notifications for shares</p>
              </div>
              <button className="relative w-12 h-6 rounded-full bg-accent transition-colors">
                <div className="absolute top-0.5 translate-x-6 w-5 h-5 rounded-full bg-white shadow" />
              </button>
            </div>
          </div>
        </section>
        
        {/* Security section */}
        <section className="card p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-lg bg-red-100 flex items-center justify-center">
              <Shield className="h-5 w-5 text-red-600" />
            </div>
            <div>
              <h2 className="font-semibold text-text">Security</h2>
              <p className="text-sm text-text-muted">Protect your account</p>
            </div>
          </div>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between py-3 border-b border-border">
              <div>
                <p className="text-sm font-medium text-text">Change Password</p>
                <p className="text-sm text-text-muted">Update your password</p>
              </div>
              <Button variant="secondary" size="sm">Change</Button>
            </div>
            
            <div className="flex items-center justify-between py-3">
              <div>
                <p className="text-sm font-medium text-text">Active Sessions</p>
                <p className="text-sm text-text-muted">Manage your active sessions</p>
              </div>
              <Button variant="ghost" size="sm">View All</Button>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
