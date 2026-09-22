import { useEffect, useRef, useState, useCallback } from 'react';
import { clsx } from 'clsx';
import { 
  ChevronLeft, 
  ChevronRight, 
  ZoomIn, 
  ZoomOut, 
  Download,
  Maximize2,
  RotateCcw
} from 'lucide-react';
import { Button } from '../ui/Button';

interface PagePreviewProps {
  svgContent?: string;
  pageNumber: number;
  totalPages: number;
  loading?: boolean;
  error?: string;
  onPageChange: (page: number) => void;
  onExport?: (format: 'svg' | 'png' | 'pdf') => void;
}

export function PagePreview({
  svgContent,
  pageNumber,
  totalPages,
  loading,
  error,
  onPageChange,
  onExport
}: PagePreviewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState(1);
  const [isFullscreen, setIsFullscreen] = useState(false);
  
  const handleZoomIn = () => setScale((s) => Math.min(s + 0.25, 3));
  const handleZoomOut = () => setScale((s) => Math.max(s - 0.25, 0.5));
  const handleResetZoom = () => setScale(1);
  
  const handleFullscreen = useCallback(() => {
    if (!document.fullscreenElement) {
      containerRef.current?.requestFullscreen();
      setIsFullscreen(true);
    } else {
      document.exitFullscreen();
      setIsFullscreen(false);
    }
  }, []);
  
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'ArrowLeft' && pageNumber > 1) {
      onPageChange(pageNumber - 1);
    } else if (e.key === 'ArrowRight' && pageNumber < totalPages) {
      onPageChange(pageNumber + 1);
    } else if (e.key === '=' || e.key === '+') {
      handleZoomIn();
    } else if (e.key === '-') {
      handleZoomOut();
    }
  }, [pageNumber, totalPages, onPageChange]);
  
  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
  
  return (
    <div 
      ref={containerRef}
      className={clsx(
        'flex flex-col bg-surface rounded-xl border border-border overflow-hidden',
        isFullscreen && 'fixed inset-0 z-50 rounded-none'
      )}
    >
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border bg-surface-dim">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onPageChange(pageNumber - 1)}
            disabled={pageNumber <= 1}
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <span className="text-sm font-medium text-text">
            Page {pageNumber} of {totalPages}
          </span>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onPageChange(pageNumber + 1)}
            disabled={pageNumber >= totalPages}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
        
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" onClick={handleZoomOut}>
            <ZoomOut className="h-4 w-4" />
          </Button>
          <span className="w-16 text-center text-sm text-text-muted">
            {Math.round(scale * 100)}%
          </span>
          <Button variant="ghost" size="sm" onClick={handleZoomIn}>
            <ZoomIn className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="sm" onClick={handleResetZoom}>
            <RotateCcw className="h-4 w-4" />
          </Button>
          <div className="w-px h-6 bg-border mx-2" />
          <Button variant="ghost" size="sm" onClick={handleFullscreen}>
            <Maximize2 className="h-4 w-4" />
          </Button>
          {onExport && (
            <>
              <div className="w-px h-6 bg-border mx-2" />
              <Button variant="secondary" size="sm" onClick={() => onExport('svg')}>
                <Download className="h-4 w-4" />
                SVG
              </Button>
              <Button variant="secondary" size="sm" onClick={() => onExport('png')}>
                PNG
              </Button>
              <Button variant="secondary" size="sm" onClick={() => onExport('pdf')}>
                PDF
              </Button>
            </>
          )}
        </div>
      </div>
      
      {/* Preview area */}
      <div className="flex-1 overflow-auto p-4 flex items-center justify-center bg-neutral-100">
        {loading ? (
          <div className="flex flex-col items-center gap-3">
            <div className="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin" />
            <span className="text-sm text-text-muted">Loading page...</span>
          </div>
        ) : error ? (
          <div className="text-center text-danger">
            <p className="font-medium">Failed to load page</p>
            <p className="text-sm mt-1">{error}</p>
          </div>
        ) : svgContent ? (
          <div 
            className="bg-white shadow-lg rounded-lg overflow-hidden transition-transform duration-200"
            style={{ transform: `scale(${scale})`, transformOrigin: 'center' }}
          >
            <div 
              dangerouslySetInnerHTML={{ __html: svgContent }}
              className="[&>svg]:w-full [&>svg]:h-auto"
            />
          </div>
        ) : (
          <div className="text-text-muted">No preview available</div>
        )}
      </div>
      
      {/* Page thumbnails */}
      {totalPages > 1 && (
        <div className="flex items-center gap-2 px-4 py-3 border-t border-border bg-surface-dim overflow-x-auto">
          {Array.from({ length: Math.min(totalPages, 10) }, (_, i) => i + 1).map((page) => (
            <button
              key={page}
              onClick={() => onPageChange(page)}
              className={clsx(
                'w-12 h-16 rounded border-2 flex-shrink-0 transition-all',
                page === pageNumber
                  ? 'border-accent bg-accent/10'
                  : 'border-border hover:border-border-strong bg-surface'
              )}
            >
              <span className="text-xs text-text-muted">{page}</span>
            </button>
          ))}
          {totalPages > 10 && (
            <span className="text-xs text-text-muted px-2">
              +{totalPages - 10} more
            </span>
          )}
        </div>
      )}
    </div>
  );
}
