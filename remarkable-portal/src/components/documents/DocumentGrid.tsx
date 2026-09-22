import { DocumentCard } from './DocumentCard';
import type { Document } from '../../types';

interface DocumentGridProps {
  documents: Document[];
  view?: 'grid' | 'list';
  onRename?: (id: string, name: string) => void;
  onDelete?: (id: string) => void;
  onShare?: (id: string) => void;
  onDownload?: (id: string) => void;
  emptyMessage?: string;
}

export function DocumentGrid({
  documents,
  view = 'grid',
  onRename,
  onDelete,
  onShare,
  onDownload,
  emptyMessage = 'No documents found'
}: DocumentGridProps) {
  if (documents.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <div className="w-16 h-16 bg-surface-dim rounded-full flex items-center justify-center mb-4">
          <svg className="w-8 h-8 text-text-subtle" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>
        <p className="text-text-muted">{emptyMessage}</p>
      </div>
    );
  }
  
  if (view === 'list') {
    return (
      <div className="space-y-1">
        {documents.map((doc) => (
          <DocumentCard
            key={doc.id}
            document={doc}
            view="list"
            onRename={onRename}
            onDelete={onDelete}
            onShare={onShare}
            onDownload={onDownload}
          />
        ))}
      </div>
    );
  }
  
  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      {documents.map((doc) => (
        <DocumentCard
          key={doc.id}
          document={doc}
          view="grid"
          onRename={onRename}
          onDelete={onDelete}
          onShare={onShare}
          onDownload={onDownload}
        />
      ))}
    </div>
  );
}
