import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Upload, File, X, CheckCircle, AlertCircle, Loader2 } from 'lucide-react';
import { Header } from '../components/layout/Header';
import { Button } from '../components/ui/Button';
import { sync } from '../api/client';
import type { UploadProgress } from '../types';

export function UploadPage() {
  const navigate = useNavigate();
  const [uploads, setUploads] = useState<UploadProgress[]>([]);
  const [dragging, setDragging] = useState(false);
  
  const handleFiles = useCallback((files: FileList) => {
    const allowedTypes = ['application/pdf', 'application/epub+zip'];
    const validFiles = Array.from(files).filter(f => 
      allowedTypes.includes(f.type) || f.name.endsWith('.pdf') || f.name.endsWith('.epub')
    );
    
    validFiles.forEach(file => {
      const id = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
      
      setUploads(prev => [...prev, {
        fileId: id,
        fileName: file.name,
        progress: 0,
        status: 'uploading'
      }]);
      
      sync.uploadFile(file, '', (progress) => {
        setUploads(prev => prev.map(u => 
          u.fileId === id ? { ...u, progress } : u
        ));
      }).then(() => {
        setUploads(prev => prev.map(u => 
          u.fileId === id ? { ...u, status: 'complete', progress: 100 } : u
        ));
      }).catch((err) => {
        setUploads(prev => prev.map(u => 
          u.fileId === id ? { ...u, status: 'error', error: err.message } : u
        ));
      });
    });
  }, []);
  
  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
    if (e.dataTransfer.files.length) {
      handleFiles(e.dataTransfer.files);
    }
  }, [handleFiles]);
  
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDragging(true);
  };
  
  const handleDragLeave = () => {
    setDragging(false);
  };
  
  const removeUpload = (id: string) => {
    setUploads(prev => prev.filter(u => u.fileId !== id));
  };
  
  const completedCount = uploads.filter(u => u.status === 'complete').length;
  const hasUploads = uploads.length > 0;
  
  return (
    <div className="min-h-screen">
      <Header 
        title="Upload Documents" 
        subtitle="Add PDFs and EPUBs to your library"
        showCreateFolder={false}
      />
      
      <div className="p-6 max-w-3xl mx-auto">
        {/* Drop zone */}
        <div
          onDrop={handleDrop}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          className={`
            border-2 border-dashed rounded-xl p-12 text-center transition-all
            ${dragging 
              ? 'border-accent bg-accent/5' 
              : 'border-border-strong hover:border-accent hover:bg-surface-dim'
            }
          `}
        >
          <div className="w-16 h-16 bg-surface-dim rounded-full flex items-center justify-center mx-auto mb-4">
            <Upload className={`h-8 w-8 ${dragging ? 'text-accent' : 'text-text-muted'}`} />
          </div>
          <h3 className="text-lg font-semibold text-text mb-2">
            Drop files here
          </h3>
          <p className="text-text-muted mb-4">
            or click to browse
          </p>
          <input
            type="file"
            multiple
            accept=".pdf,.epub"
            onChange={(e) => e.target.files && handleFiles(e.target.files)}
            className="hidden"
            id="file-input"
          />
          <label htmlFor="file-input" className="cursor-pointer">
            <span className="btn-secondary inline-flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium rounded-lg">
              Select Files
            </span>
          </label>
          <p className="text-xs text-text-subtle mt-4">
            Supported formats: PDF, EPUB
          </p>
        </div>
        
        {/* Upload list */}
        {hasUploads && (
          <div className="mt-8 space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium text-text">
                Uploads ({completedCount}/{uploads.length} complete)
              </h3>
              {completedCount === uploads.length && completedCount > 0 && (
                <Button variant="primary" size="sm" onClick={() => navigate('/')}>
                  View Documents
                </Button>
              )}
            </div>
            
            {uploads.map((upload) => (
              <div 
                key={upload.fileId}
                className="flex items-center gap-4 p-4 bg-surface rounded-lg border border-border"
              >
                <div className="w-10 h-10 bg-surface-dim rounded-lg flex items-center justify-center">
                  {upload.status === 'complete' ? (
                    <CheckCircle className="h-5 w-5 text-success" />
                  ) : upload.status === 'error' ? (
                    <AlertCircle className="h-5 w-5 text-danger" />
                  ) : (
                    <File className="h-5 w-5 text-text-muted" />
                  )}
                </div>
                
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-text truncate">
                    {upload.fileName}
                  </p>
                  {upload.status === 'uploading' && (
                    <div className="mt-2">
                      <div className="h-1.5 bg-surface-dim rounded-full overflow-hidden">
                        <div 
                          className="h-full bg-accent transition-all duration-300"
                          style={{ width: `${upload.progress}%` }}
                        />
                      </div>
                    </div>
                  )}
                  {upload.status === 'error' && (
                    <p className="text-xs text-danger mt-1">{upload.error}</p>
                  )}
                </div>
                
                {upload.status === 'uploading' ? (
                  <Loader2 className="h-5 w-5 text-accent animate-spin" />
                ) : (
                  <button
                    onClick={() => removeUpload(upload.fileId)}
                    className="p-1 rounded hover:bg-surface-dim text-text-muted hover:text-text"
                  >
                    <X className="h-4 w-4" />
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
