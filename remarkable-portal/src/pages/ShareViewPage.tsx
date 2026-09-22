import { useState, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { FileText, Loader2, AlertCircle } from 'lucide-react';
import { PagePreview } from '../components/preview/PagePreview';
import { shares, exportApi } from '../api/client';
import type { Document } from '../types';

export function ShareViewPage() {
  const { token } = useParams<{ token: string }>();
  const [doc, setDoc] = useState<Document | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [svgContent, setSvgContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [pageLoading, setPageLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    if (!token) return;
    
    const loadShare = async () => {
      setLoading(true);
      try {
        const data = await shares.getByToken(token);
        setDoc(data.document);
      } catch {
        setError('This share link is invalid or has expired');
      } finally {
        setLoading(false);
      }
    };
    
    loadShare();
  }, [token]);
  
  const loadPage = useCallback(async (page: number) => {
    if (!doc?.hash) return;
    
    setPageLoading(true);
    try {
      const svg = await exportApi.toSvg(doc.hash, page);
      setSvgContent(svg);
    } catch {
      console.error('Failed to load page');
    } finally {
      setPageLoading(false);
    }
  }, [doc?.hash]);
  
  useEffect(() => {
    if (doc?.hash) {
      loadPage(currentPage);
    }
  }, [doc?.hash, currentPage, loadPage]);
  
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-surface-dim">
        <Loader2 className="h-8 w-8 animate-spin text-accent" />
      </div>
    );
  }
  
  if (error || !doc) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-surface-dim">
        <div className="text-center">
          <AlertCircle className="h-12 w-12 text-danger mx-auto mb-4" />
          <h1 className="text-xl font-semibold text-text mb-2">Link Expired</h1>
          <p className="text-text-muted">{error}</p>
        </div>
      </div>
    );
  }
  
  return (
    <div className="min-h-screen bg-surface-dim">
      <header className="bg-surface border-b border-border">
        <div className="max-w-5xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-accent to-blue-600 flex items-center justify-center">
              <FileText className="h-5 w-5 text-white" />
            </div>
            <div>
              <h1 className="font-semibold text-text">{doc.visibleName}</h1>
              <p className="text-xs text-text-muted">
                Shared document • {doc.pageCount || 1} pages
              </p>
            </div>
          </div>
        </div>
      </header>
      
      <div className="p-6">
        <div className="max-w-5xl mx-auto">
          <PagePreview
            svgContent={svgContent || undefined}
            pageNumber={currentPage}
            totalPages={doc.pageCount || 1}
            loading={pageLoading}
            onPageChange={setCurrentPage}
          />
        </div>
      </div>
      
      <footer className="fixed bottom-0 inset-x-0 bg-surface border-t border-border py-4">
        <div className="max-w-5xl mx-auto px-6 flex items-center justify-between">
          <p className="text-sm text-text-muted">
            Shared via reMarkable Portal
          </p>
          <span className="text-xs text-text-subtle">
            View-only access
          </span>
        </div>
      </footer>
    </div>
  );
}
