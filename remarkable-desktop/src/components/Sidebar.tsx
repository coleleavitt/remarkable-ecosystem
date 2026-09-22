import { Cloud, FolderTree, AlertTriangle, Settings as SettingsIcon, Info } from 'lucide-react';
import { useSyncStatus } from '../hooks';

interface SidebarProps {
  currentView: string;
  onViewChange: (view: string) => void;
}

export function Sidebar({ currentView, onViewChange }: SidebarProps) {
  const { status } = useSyncStatus();
  
  const navItems = [
    { id: 'documents', label: 'Documents', icon: FolderTree },
    { id: 'conflicts', label: 'Conflicts', icon: AlertTriangle, badge: status?.conflicts_count },
    { id: 'settings', label: 'Settings', icon: SettingsIcon },
    { id: 'about', label: 'About', icon: Info },
  ];
  
  return (
    <div className="w-56 bg-gray-50 border-r flex flex-col">
      {/* Logo */}
      <div className="p-4 border-b">
        <div className="flex items-center gap-2">
          <Cloud className="w-8 h-8 text-gray-700" />
          <div>
            <h1 className="font-bold text-gray-800">reMarkable</h1>
            <span className="text-xs text-gray-500">Desktop Sync</span>
          </div>
        </div>
      </div>
      
      {/* Navigation */}
      <nav className="flex-1 p-2">
        {navItems.map(item => (
          <button
            key={item.id}
            onClick={() => onViewChange(item.id)}
            className={`w-full flex items-center gap-3 px-4 py-2 rounded-lg text-left mb-1 transition-colors ${
              currentView === item.id
                ? 'bg-blue-100 text-blue-700'
                : 'text-gray-600 hover:bg-gray-100'
            }`}
          >
            <item.icon className="w-5 h-5" />
            <span className="flex-1">{item.label}</span>
            {item.badge !== undefined && item.badge > 0 && (
              <span className="px-2 py-0.5 bg-amber-500 text-white text-xs rounded-full">
                {item.badge}
              </span>
            )}
          </button>
        ))}
      </nav>
      
      {/* Status footer */}
      <div className="p-4 border-t text-xs text-gray-500">
        <div>Version 0.1.0</div>
        {status?.last_sync && (
          <div>Last sync: {new Date(status.last_sync).toLocaleTimeString()}</div>
        )}
      </div>
    </div>
  );
}

export default Sidebar;
