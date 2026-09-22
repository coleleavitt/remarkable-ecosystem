import { AlertTriangle, Check, Clock } from 'lucide-react';
import { useConflicts } from '../hooks';
import api from '../api';
import type { ConflictInfo, ConflictStrategy } from '../types';

function ConflictCard({ conflict, onResolve }: { conflict: ConflictInfo; onResolve: () => void }) {
  const handleResolve = async (strategy: ConflictStrategy) => {
    try {
      await api.resolveConflict(conflict.document_id, strategy);
      onResolve();
    } catch (e) {
      console.error('Failed to resolve conflict:', e);
    }
  };

  const localDate = new Date(conflict.local_modified);
  const remoteDate = new Date(conflict.remote_modified);
  const localNewer = localDate > remoteDate;

  return (
    <div className="border rounded-lg p-4 bg-white shadow-sm">
      <div className="flex items-start gap-3">
        <AlertTriangle className="w-5 h-5 text-amber-500 mt-1 flex-shrink-0" />
        <div className="flex-1">
          <h3 className="font-semibold text-gray-800">{conflict.document_name}</h3>
          <p className="text-sm text-gray-500 mt-1">
            This document has been modified both locally and on the server.
          </p>
          
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className={`p-3 rounded border ${localNewer ? 'border-green-300 bg-green-50' : 'border-gray-200'}`}>
              <div className="flex items-center gap-2 text-sm font-medium text-gray-700">
                <Clock className="w-4 h-4" />
                Local Version
                {localNewer && <span className="text-xs bg-green-200 text-green-800 px-2 py-0.5 rounded">Newer</span>}
              </div>
              <div className="text-sm text-gray-500 mt-1">
                Modified: {localDate.toLocaleString()}
              </div>
              {conflict.local_size > 0 && (
                <div className="text-sm text-gray-500">
                  Size: {(conflict.local_size / 1024).toFixed(1)} KB
                </div>
              )}
            </div>
            
            <div className={`p-3 rounded border ${!localNewer ? 'border-green-300 bg-green-50' : 'border-gray-200'}`}>
              <div className="flex items-center gap-2 text-sm font-medium text-gray-700">
                <Clock className="w-4 h-4" />
                Remote Version
                {!localNewer && <span className="text-xs bg-green-200 text-green-800 px-2 py-0.5 rounded">Newer</span>}
              </div>
              <div className="text-sm text-gray-500 mt-1">
                Modified: {remoteDate.toLocaleString()}
              </div>
              {conflict.remote_size > 0 && (
                <div className="text-sm text-gray-500">
                  Size: {(conflict.remote_size / 1024).toFixed(1)} KB
                </div>
              )}
            </div>
          </div>
          
          <div className="flex gap-2 mt-4">
            <button
              onClick={() => handleResolve('keep_local')}
              className="px-4 py-2 bg-blue-600 text-white text-sm rounded hover:bg-blue-700"
            >
              Keep Local
            </button>
            <button
              onClick={() => handleResolve('keep_remote')}
              className="px-4 py-2 bg-purple-600 text-white text-sm rounded hover:bg-purple-700"
            >
              Keep Remote
            </button>
            <button
              onClick={() => handleResolve('keep_both')}
              className="px-4 py-2 bg-gray-600 text-white text-sm rounded hover:bg-gray-700"
            >
              Keep Both
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export function ConflictResolver() {
  const { conflicts, loading, refresh } = useConflicts();
  
  if (loading) {
    return <div className="p-4">Loading conflicts...</div>;
  }
  
  if (conflicts.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center p-12 text-gray-500">
        <Check className="w-16 h-16 mb-4 text-green-400" />
        <h2 className="text-xl font-semibold text-gray-700">No Conflicts</h2>
        <p className="mt-2">All documents are synced correctly.</p>
      </div>
    );
  }
  
  return (
    <div className="p-6">
      <div className="flex items-center gap-3 mb-6">
        <AlertTriangle className="w-6 h-6 text-amber-500" />
        <h2 className="text-2xl font-bold">Resolve Conflicts</h2>
        <span className="ml-2 px-2 py-1 bg-amber-100 text-amber-800 text-sm rounded-full">
          {conflicts.length} conflict{conflicts.length !== 1 ? 's' : ''}
        </span>
      </div>
      
      <div className="space-y-4">
        {conflicts.map(conflict => (
          <ConflictCard
            key={conflict.document_id}
            conflict={conflict}
            onResolve={refresh}
          />
        ))}
      </div>
    </div>
  );
}

export default ConflictResolver;
