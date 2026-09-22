import { NavLink, useNavigate } from 'react-router-dom';
import { clsx } from 'clsx';
import { 
  FileText, 
  FolderOpen, 
  Settings, 
  Share2, 
  LogOut,
  Home,
  Upload,
  Cloud
} from 'lucide-react';
import { useAuth } from '../../contexts/AuthContext';

export function Sidebar() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  
  const handleLogout = () => {
    logout();
    navigate('/login');
  };
  
  const navItems = [
    { to: '/', icon: Home, label: 'All Documents' },
    { to: '/recent', icon: FileText, label: 'Recent' },
    { to: '/folders', icon: FolderOpen, label: 'Folders' },
    { to: '/shared', icon: Share2, label: 'Shared Links' },
  ];
  
  return (
    <aside className="w-64 bg-surface border-r border-border flex flex-col h-screen sticky top-0">
      {/* Logo */}
      <div className="p-6 border-b border-border">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-accent to-blue-600 flex items-center justify-center">
            <FileText className="h-5 w-5 text-white" />
          </div>
          <div>
            <h1 className="font-bold text-text">reMarkable</h1>
            <p className="text-xs text-text-muted">Portal</p>
          </div>
        </div>
      </div>
      
      {/* Upload button */}
      <div className="p-4">
        <NavLink
          to="/upload"
          className="w-full btn-primary flex items-center justify-center gap-2"
        >
          <Upload className="h-4 w-4" />
          Upload
        </NavLink>
      </div>
      
      {/* Navigation */}
      <nav className="flex-1 px-3 py-2 space-y-1">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) => clsx(
              'flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors',
              isActive
                ? 'bg-accent/10 text-accent'
                : 'text-text-muted hover:text-text hover:bg-surface-dim'
            )}
          >
            <Icon className="h-5 w-5" />
            {label}
          </NavLink>
        ))}
      </nav>
      
      {/* Sync status */}
      {user && (
        <div className="px-4 py-3 border-t border-border">
          <div className="flex items-center gap-2 text-xs text-text-muted">
            <Cloud className={clsx(
              'h-4 w-4',
              user.syncStatus === 'synced' && 'text-success',
              user.syncStatus === 'syncing' && 'text-accent animate-pulse',
              user.syncStatus === 'offline' && 'text-text-subtle'
            )} />
            <span className="capitalize">{user.syncStatus}</span>
          </div>
        </div>
      )}
      
      {/* User section */}
      <div className="p-4 border-t border-border">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-full bg-surface-dim flex items-center justify-center">
            <span className="text-sm font-medium text-text-muted">
              {user?.email?.charAt(0).toUpperCase() || 'U'}
            </span>
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-text truncate">
              {user?.name || user?.email || 'User'}
            </p>
            <p className="text-xs text-text-muted truncate">
              {user?.email}
            </p>
          </div>
        </div>
        
        <div className="mt-3 flex gap-2">
          <NavLink
            to="/settings"
            className="flex-1 btn-ghost text-xs justify-center"
          >
            <Settings className="h-3.5 w-3.5" />
            Settings
          </NavLink>
          <button
            onClick={handleLogout}
            className="flex-1 btn-ghost text-xs justify-center text-danger hover:text-danger"
          >
            <LogOut className="h-3.5 w-3.5" />
            Logout
          </button>
        </div>
      </div>
    </aside>
  );
}
