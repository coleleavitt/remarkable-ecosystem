import { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate, useSearchParams } from 'react-router-dom';
import { ArrowLeft, FileText, Download, Share2, Loader2 } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { PagePreview } from '../components/preview/PagePreview';
import { ShareDialog } from '../components/share/ShareDialog';
import { Button } from '../components/ui/Button';
import { sync, exportApi, shares } from '../api/client';
import type { GenTreeItem } from '../types';

export function PreviewPage() {
  const { documentId } = useParams<{ documentId: string }>();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  
  const [doc, setDoc] = useState<GenTreeItem | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [svgContent, setSvgContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [pageLoading, setPageLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [shareDialogOpen, setShareDialogOpen] = useState(false);
  const [exporting, setExporting] = useState(false);
  
  // Load document metadata
  useEffect(() => {
    if (!documentId) return;
    
    const loadDocument = async () => {
      setLoading(true);
      try {
        const tree = await sync.getGenTree();
        const found = tree.items.find(d => d.id === documentId);
        if (found) {
          setDoc(found);
        } else {
          setError('Document not found');
        }
      } catch {
        setError('Failed to load document');
      } finally {
        setLoading(false);
      }
    };
    
    loadDocument();
  }, [documentId]);
  
  // Load page SVG
  const loadPage = useCallback(async (page: number) => {
    if (!doc?.hash) return;
    
    setPageLoading(true);
    setSvgContent(null);
    
    try {
      const svg = await exportApi.toSvg(doc.hash, page);
      setSvgContent(svg);
    } catch (e) {
      console.error('Failed to load page:', e);
    } finally {
      setPageLoading(false);
    }
  }, [doc?.hash]);
  
  useEffect(() => {
    if (doc?.hash) {
      loadPage(currentPage);
    }
  }, [doc?.hash, currentPage, loadPage]);
  
  // Auto-download if ?download=true
  useEffect(() => {
    if (searchParams.get('download') === 'true' && doc) {
      handleExport('pdf');
    }
  }, [searchParams, doc]);
  
  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };
  
  const handleExport = async (format: 'svg' | 'png' | 'pdf') => {
    if (!doc?.hash) return;
    
    setExporting(true);
    try {
      let blob: Blob;
      let filename: string;
      
      if (format === 'svg') {
        const svg = svgContent || await exportApi.toSvg(doc.hash, currentPage);
        blob = new Blob([svg], { type: 'image/svg+xml' });
        filename = `${doc.visibleName}_page${currentPage}.svg`;
      } else if (format === 'png') {
        blob = await exportApi.toPng(doc.hash, currentPage, 300);
        filename = `${doc.visibleName}_page${currentPage}.png`;
      } else {
        blob = await exportApi.toPdf(doc.hash);
        filename = `${doc.visibleName}.pdf`;
      }
      
      const url = URL.createObjectURL(blob);
      const a = window.document.createElement('a');
      a.href = url;
      a.download = filename;
      a.click();
      URL.revokeObjectURL(url);
    } finally {
      setExporting(false);
    }
  };
  
  const handleCreateShareLink = async (options: { expiresIn?: number; maxViews?: number }) => {
    if (!documentId) throw new Error('No document');
    const link = await shares.create(documentId, options);
    return { url: `${window.location.origin}/share/${link.token}` };
  };
  
  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="h-8 w-8 animate-spin text-accent" />
      </div>
    );
  }
  
  if (error || !doc) {
    return (
      <div className="flex flex-col items-center justify-center h-screen gap-4">
        <p className="text-danger">{error || 'Document not found'}</p>
        <Button variant="secondary" onClick={() => navigate('/')}>
          <ArrowLeft className="h-4 w-4" />
          Back to Documents
        </Button>
      </div>
    );
  }
  
  return (
    <div className="min-h-screen bg-surface-dim">
      <header className="sticky top-0 z-40 bg-surface border-b border-border">
        <div className="px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
              <ArrowLeft className="h-4 w-4" />
            </Button>
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-surface-dim rounded-lg flex items-center justify-center">
                <FileText className="h-5 w-5 text-text-muted" />
              </div>
              <div>
                <h1 className="font-semibold text-text">{doc.visibleName}</h1>
                <p className="text-xs text-text-muted">
                  {doc.pageCount || 1} pages • Updated {formatDistanceToNow(new Date(doc.lastModified), { addSuffix: true })}
                </p>
              </div>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <Button variant="secondary" size="sm" onClick={() => setShareDialogOpen(true)}>
              <Share2 className="h-4 w-4" />
              Share
            </Button>
            <Button 
              variant="primary" 
              size="sm" 
              onClick={() => handleExport('pdf')}
              loading={exporting}
            >
              <Download className="h-4 w-4" />
              Download PDF
            </Button>
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
            onPageChange={handlePageChange}
            onExport={handleExport}
          />
        </div>
      </div>
      
      <ShareDialog
        isOpen={shareDialogOpen}
        onClose={() => setShareDialogOpen(false)}
        documentName={doc.visibleName}
        onCreateLink={handleCreateShareLink}
      />
    </div>
  );
}
