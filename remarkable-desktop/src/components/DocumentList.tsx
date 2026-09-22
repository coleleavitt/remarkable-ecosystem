import { File, Folder, AlertTriangle, Check, Clock, Upload, Download } from 'lucide-react';
import { useDocuments } from '../hooks';
import type { DocumentState, SyncStatus } from '../types';

function getStatusIcon(status: SyncStatus) {
  switch (status) {
    case 'synced':
      return <Check className="w-4 h-4 text-green-600" />;
    case 'syncing':
      return <Clock className="w-4 h-4 text-blue-600 animate-pulse" />;
    case 'conflict':
      return <AlertTriangle className="w-4 h-4 text-amber-600" />;
    case 'local_only':
      return <Upload className="w-4 h-4 text-purple-600" />;
    case 'remote_only':
      return <Download className="w-4 h-4 text-blue-600" />;
    case 'modified':
      return <Clock className="w-4 h-4 text-orange-600" />;
    case 'error':
      return <AlertTriangle className="w-4 h-4 text-red-600" />;
    default:
      return null;
  }
}

function DocumentItem({ doc }: { doc: DocumentState }) {
  const isFolder = doc.doc_type === 'folder';
  
  return (
    <div className="flex items-center gap-3 px-4 py-2 hover:bg-gray-50 border-b border-gray-100">
      {isFolder ? (
        <Folder className="w-5 h-5 text-amber-500" />
      ) : (
        <File className="w-5 h-5 text-gray-400" />
      )}
      
      <span className="flex-1 truncate">{doc.name}</span>
      
      <div className="flex items-center gap-2">
        {getStatusIcon(doc.sync_status)}
        <span className="text-xs text-gray-500 capitalize">
          {doc.sync_status.replace('_', ' ')}
        </span>
      </div>
      
      {doc.last_synced && (
        <span className="text-xs text-gray-400">
          {new Date(doc.last_synced).toLocaleDateString()}
        </span>
      )}
    </div>
  );
}

export function DocumentList() {
  const { documents, loading } = useDocuments();
  
  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Clock className="w-6 h-6 animate-spin text-gray-400" />
      </div>
    );
  }
  
  // Organize documents by parent
  const rootDocs = documents.filter(d => !d.parent_id);
  const folders = rootDocs.filter(d => d.doc_type === 'folder');
  const files = rootDocs.filter(d => d.doc_type !== 'folder');
  
  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 bg-gray-50 border-b">
        <h2 className="font-semibold text-gray-700">Documents</h2>
        <span className="text-sm text-gray-500">{documents.length} items</span>
      </div>
      
      <div className="flex-1 overflow-auto">
        {documents.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-8 text-gray-500">
            <Folder className="w-12 h-12 mb-2 text-gray-300" />
            <p>No documents synced yet</p>
            <p className="text-sm">Documents will appear here after syncing</p>
          </div>
        ) : (
          <>
            {folders.map(doc => (
              <DocumentItem key={doc.id} doc={doc} />
            ))}
            {files.map(doc => (
              <DocumentItem key={doc.id} doc={doc} />
            ))}
          </>
        )}
      </div>
    </div>
  );
}

export default DocumentList;
