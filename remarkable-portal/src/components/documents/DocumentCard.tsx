
import { useNavigate } from 'react-router-dom';
import { clsx } from 'clsx';
import { 
  FileText, 
  Folder, 
  MoreVertical, 
  Download, 
  Share2, 
  Pencil, 
  Trash2,
  Star,
  Eye,
  File
} from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { Dropdown } from '../ui/Dropdown';
import type { Document } from '../../types';

interface DocumentCardProps {
  document: Document;
  view?: 'grid' | 'list';
  onRename?: (id: string, name: string) => void;
  onDelete?: (id: string) => void;
  onShare?: (id: string) => void;
  onDownload?: (id: string) => void;
}

const fileTypeIcons = {
  notebook: FileText,
  pdf: File,
  epub: File,
};

const fileTypeColors = {
  notebook: 'text-blue-500',
  pdf: 'text-red-500',
  epub: 'text-green-500',
};

export function DocumentCard({ 
  document, 
  view = 'grid',
  onRename,
  onDelete,
  onShare,
  onDownload
}: DocumentCardProps) {
  const navigate = useNavigate();
  const isFolder = document.type === 'CollectionType';
  const Icon = isFolder ? Folder : (fileTypeIcons[document.fileType as keyof typeof fileTypeIcons] || FileText);
  const iconColor = isFolder ? 'text-amber-500' : (fileTypeColors[document.fileType as keyof typeof fileTypeColors] || 'text-text-muted');
  
  const handleClick = () => {
    if (isFolder) {
      navigate(`/folder/${document.id}`);
    } else {
      navigate(`/preview/${document.id}`);
    }
  };
  
  const menuItems = [
    ...(!isFolder ? [{
      label: 'Preview',
      icon: <Eye className="h-4 w-4" />,
      onClick: () => navigate(`/preview/${document.id}`),
    }] : []),
    ...(onShare && !isFolder ? [{
      label: 'Share',
      icon: <Share2 className="h-4 w-4" />,
      onClick: () => onShare(document.id),
    }] : []),
    ...(onDownload && !isFolder ? [{
      label: 'Download',
      icon: <Download className="h-4 w-4" />,
      onClick: () => onDownload(document.id),
    }] : []),
    ...(onRename ? [{
      label: 'Rename',
      icon: <Pencil className="h-4 w-4" />,
      onClick: () => onRename(document.id, document.visibleName),
    }] : []),
    ...(onDelete ? [{
      label: 'Delete',
      icon: <Trash2 className="h-4 w-4" />,
      onClick: () => onDelete(document.id),
      danger: true,
    }] : []),
  ];
  
  if (view === 'list') {
    return (
      <div 
        className="flex items-center gap-4 px-4 py-3 bg-surface hover:bg-surface-dim rounded-lg cursor-pointer transition-colors group"
        onClick={handleClick}
      >
        <Icon className={clsx('h-5 w-5', iconColor)} />
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-text truncate">
            {document.visibleName}
          </p>
          <p className="text-xs text-text-muted">
            {document.lastModified && formatDistanceToNow(new Date(document.lastModified), { addSuffix: true })}
            {!isFolder && document.pageCount && ` • ${document.pageCount} pages`}
          </p>
        </div>
        {document.pinned && (
          <Star className="h-4 w-4 text-amber-400 fill-amber-400" />
        )}
        <div onClick={(e) => e.stopPropagation()}>
          <Dropdown
            trigger={
              <button className="p-1.5 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-surface-dim transition-all">
                <MoreVertical className="h-4 w-4 text-text-muted" />
              </button>
            }
            items={menuItems}
          />
        </div>
      </div>
    );
  }
  
  return (
    <div 
      className="group card-hover p-4"
      onClick={handleClick}
    >
      {/* Thumbnail */}
      <div className="aspect-[4/3] bg-surface-dim rounded-lg mb-3 flex items-center justify-center overflow-hidden">
        <Icon className={clsx('h-12 w-12', iconColor)} />
      </div>
      
      {/* Info */}
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-text truncate">
            {document.visibleName}
          </p>
          <p className="text-xs text-text-muted mt-0.5">
            {document.lastModified && formatDistanceToNow(new Date(document.lastModified), { addSuffix: true })}
          </p>
        </div>
        <div className="flex items-center gap-1">
          {document.pinned && (
            <Star className="h-4 w-4 text-amber-400 fill-amber-400" />
          )}
          <div onClick={(e) => e.stopPropagation()}>
            <Dropdown
              trigger={
                <button className="p-1 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-surface-dim transition-all">
                  <MoreVertical className="h-4 w-4 text-text-muted" />
                </button>
              }
              items={menuItems}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
