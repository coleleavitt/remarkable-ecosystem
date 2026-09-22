import { useState, useEffect, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Header } from '../components/layout/Header';
import { DocumentGrid } from '../components/documents/DocumentGrid';
import { Breadcrumb } from '../components/documents/Breadcrumb';
import { ShareDialog } from '../components/share/ShareDialog';
import { Modal } from '../components/ui/Modal';
import { Input } from '../components/ui/Input';
import { Button } from '../components/ui/Button';
import { sync, documents, shares } from '../api/client';
import type { GenTreeItem } from '../types';
import { Loader2 } from 'lucide-react';

export function BrowserPage() {
  const { folderId } = useParams<{ folderId?: string }>();
  const navigate = useNavigate();
  const [allDocuments, setAllDocuments] = useState<GenTreeItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [view, setView] = useState<'grid' | 'list'>('grid');
  
  // Dialogs
  const [renameDialog, setRenameDialog] = useState<{ id: string; name: string } | null>(null);
  const [deleteDialog, setDeleteDialog] = useState<string | null>(null);
  const [shareDialog, setShareDialog] = useState<string | null>(null);
  const [createFolderDialog, setCreateFolderDialog] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [actionLoading, setActionLoading] = useState(false);
  
  useEffect(() => {
    loadDocuments();
  }, []);
  
  const loadDocuments = async () => {
    setLoading(true);
    try {
      const tree = await sync.getGenTree();
      setAllDocuments(tree.items);
    } finally {
      setLoading(false);
    }
  };
  
  // Build breadcrumb path
  const breadcrumbPath = useMemo(() => {
    if (!folderId) return [];
    
    const path: { id: string; name: string }[] = [];
    let currentId: string | undefined = folderId;
    
    while (currentId) {
      const folder = allDocuments.find(d => d.id === currentId);
      if (folder) {
        path.unshift({ id: folder.id, name: folder.visibleName });
        currentId = folder.parent || undefined;
      } else {
        break;
      }
    }
    
    return path;
  }, [folderId, allDocuments]);
  
  // Filter documents for current folder
  const currentDocuments = useMemo(() => {
    const parentId = folderId || '';
    let docs = allDocuments.filter(d => d.parent === parentId);
    
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      docs = docs.filter(d => d.visibleName.toLowerCase().includes(query));
    }
    
    // Sort: folders first, then by name
    return docs.sort((a, b) => {
      if (a.type !== b.type) {
        return a.type === 'CollectionType' ? -1 : 1;
      }
      return a.visibleName.localeCompare(b.visibleName);
    });
  }, [allDocuments, folderId, searchQuery]);
  
  const currentFolder = folderId 
    ? allDocuments.find(d => d.id === folderId)
    : null;
  
  // Handlers
  const handleRename = async () => {
    if (!renameDialog) return;
    setActionLoading(true);
    try {
      await documents.rename(renameDialog.id, renameDialog.name);
      await loadDocuments();
      setRenameDialog(null);
    } finally {
      setActionLoading(false);
    }
  };
  
  const handleDelete = async () => {
    if (!deleteDialog) return;
    setActionLoading(true);
    try {
      await documents.delete(deleteDialog);
      await loadDocuments();
      setDeleteDialog(null);
    } finally {
      setActionLoading(false);
    }
  };
  
  const handleCreateFolder = async () => {
    if (!newFolderName.trim()) return;
    setActionLoading(true);
    try {
      await documents.create({
        type: 'CollectionType',
        visibleName: newFolderName,
        parent: folderId || '',
      });
      await loadDocuments();
      setCreateFolderDialog(false);
      setNewFolderName('');
    } finally {
      setActionLoading(false);
    }
  };
  
  const handleCreateShareLink = async (options: { expiresIn?: number; maxViews?: number }) => {
    if (!shareDialog) throw new Error('No document selected');
    const link = await shares.create(shareDialog, options);
    return { url: `${window.location.origin}/share/${link.token}` };
  };
  
  const handleDownload = async (id: string) => {
    navigate(`/preview/${id}?download=true`);
  };
  
  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="h-8 w-8 animate-spin text-accent" />
      </div>
    );
  }
  
  return (
    <div className="min-h-screen">
      <Header
        title={currentFolder?.visibleName || 'All Documents'}
        subtitle={`${currentDocuments.length} items`}
        onSearch={setSearchQuery}
        onViewChange={setView}
        currentView={view}
        onCreateFolder={() => setCreateFolderDialog(true)}
      />
      
      <div className="px-6 py-4">
        {breadcrumbPath.length > 0 && (
          <div className="mb-4">
            <Breadcrumb items={breadcrumbPath} />
          </div>
        )}
        
        <DocumentGrid
          documents={currentDocuments as any}
          view={view}
          onRename={(id, name) => setRenameDialog({ id, name })}
          onDelete={(id) => setDeleteDialog(id)}
          onShare={(id) => setShareDialog(id)}
          onDownload={handleDownload}
          emptyMessage={searchQuery ? 'No matching documents' : 'This folder is empty'}
        />
      </div>
      
      <Modal 
        isOpen={!!renameDialog} 
        onClose={() => setRenameDialog(null)}
        title="Rename"
      >
        <div className="space-y-4">
          <Input
            label="Name"
            value={renameDialog?.name || ''}
            onChange={(e) => setRenameDialog(prev => prev ? { ...prev, name: e.target.value } : null)}
            autoFocus
          />
          <div className="flex gap-3 justify-end">
            <Button variant="secondary" onClick={() => setRenameDialog(null)}>
              Cancel
            </Button>
            <Button onClick={handleRename} loading={actionLoading}>
              Rename
            </Button>
          </div>
        </div>
      </Modal>
      
      <Modal 
        isOpen={!!deleteDialog} 
        onClose={() => setDeleteDialog(null)}
        title="Delete Document"
      >
        <div className="space-y-4">
          <p className="text-text-muted">
            Are you sure you want to delete this item? This action cannot be undone.
          </p>
          <div className="flex gap-3 justify-end">
            <Button variant="secondary" onClick={() => setDeleteDialog(null)}>
              Cancel
            </Button>
            <Button variant="danger" onClick={handleDelete} loading={actionLoading}>
              Delete
            </Button>
          </div>
        </div>
      </Modal>
      
      <Modal 
        isOpen={createFolderDialog} 
        onClose={() => setCreateFolderDialog(false)}
        title="New Folder"
      >
        <div className="space-y-4">
          <Input
            label="Folder name"
            value={newFolderName}
            onChange={(e) => setNewFolderName(e.target.value)}
            placeholder="Untitled folder"
            autoFocus
          />
          <div className="flex gap-3 justify-end">
            <Button variant="secondary" onClick={() => setCreateFolderDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreateFolder} loading={actionLoading}>
              Create
            </Button>
          </div>
        </div>
      </Modal>
      
      {shareDialog && (
        <ShareDialog
          isOpen={!!shareDialog}
          onClose={() => setShareDialog(null)}
          documentName={allDocuments.find(d => d.id === shareDialog)?.visibleName || 'Document'}
          onCreateLink={handleCreateShareLink}
        />
      )}
    </div>
  );
}
