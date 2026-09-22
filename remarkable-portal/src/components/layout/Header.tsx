import { useState } from 'react';
import { Search, Bell, Grid, List, FolderPlus } from 'lucide-react';
import { Button } from '../ui/Button';

interface HeaderProps {
  title: string;
  subtitle?: string;
  onSearch?: (query: string) => void;
  onViewChange?: (view: 'grid' | 'list') => void;
  currentView?: 'grid' | 'list';
  onCreateFolder?: () => void;
  showCreateFolder?: boolean;
}

export function Header({ 
  title, 
  subtitle,
  onSearch, 
  onViewChange,
  currentView = 'grid',
  onCreateFolder,
  showCreateFolder = true
}: HeaderProps) {
  const [searchQuery, setSearchQuery] = useState('');
  
  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    const query = e.target.value;
    setSearchQuery(query);
    onSearch?.(query);
  };
  
  return (
    <header className="sticky top-0 z-40 bg-surface-dim/80 backdrop-blur-sm border-b border-border">
      <div className="px-6 py-4">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-text">{title}</h1>
            {subtitle && (
              <p className="text-sm text-text-muted mt-0.5">{subtitle}</p>
            )}
          </div>
          
          <div className="flex items-center gap-3">
            {onSearch && (
              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-subtle" />
                <input
                  type="text"
                  placeholder="Search documents..."
                  value={searchQuery}
                  onChange={handleSearch}
                  className="w-64 pl-10 pr-4 py-2 text-sm bg-surface border border-border-strong rounded-lg focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent"
                />
              </div>
            )}
            
            {onViewChange && (
              <div className="flex items-center bg-surface border border-border-strong rounded-lg p-0.5">
                <button
                  onClick={() => onViewChange('grid')}
                  className={`p-1.5 rounded-md transition-colors ${
                    currentView === 'grid' 
                      ? 'bg-accent text-white' 
                      : 'text-text-muted hover:text-text'
                  }`}
                >
                  <Grid className="h-4 w-4" />
                </button>
                <button
                  onClick={() => onViewChange('list')}
                  className={`p-1.5 rounded-md transition-colors ${
                    currentView === 'list' 
                      ? 'bg-accent text-white' 
                      : 'text-text-muted hover:text-text'
                  }`}
                >
                  <List className="h-4 w-4" />
                </button>
              </div>
            )}
            
            {showCreateFolder && onCreateFolder && (
              <Button variant="secondary" size="sm" onClick={onCreateFolder}>
                <FolderPlus className="h-4 w-4" />
                New Folder
              </Button>
            )}
            
            <button className="p-2 rounded-lg text-text-muted hover:text-text hover:bg-surface transition-colors relative">
              <Bell className="h-5 w-5" />
              <span className="absolute top-1.5 right-1.5 w-2 h-2 bg-accent rounded-full" />
            </button>
          </div>
        </div>
      </div>
    </header>
  );
}
