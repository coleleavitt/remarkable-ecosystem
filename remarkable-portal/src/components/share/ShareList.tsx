import { useState } from 'react';
import { formatDistanceToNow } from 'date-fns';
import { Link2, Trash2, Copy, Check, ExternalLink, Eye } from 'lucide-react';
import { Button } from '../ui/Button';
import type { ShareLink } from '../../types';

interface ShareListProps {
  shares: ShareLink[];
  onDelete: (id: string) => void;
}

export function ShareList({ shares, onDelete }: ShareListProps) {
  const [copiedId, setCopiedId] = useState<string | null>(null);
  
  const handleCopy = async (token: string, id: string) => {
    const url = `${window.location.origin}/share/${token}`;
    await navigator.clipboard.writeText(url);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };
  
  if (shares.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="w-16 h-16 bg-surface-dim rounded-full flex items-center justify-center mx-auto mb-4">
          <Link2 className="h-8 w-8 text-text-subtle" />
        </div>
        <p className="text-text-muted">No shared links yet</p>
        <p className="text-sm text-text-subtle mt-1">
          Share a document to create your first link
        </p>
      </div>
    );
  }
  
  return (
    <div className="space-y-2">
      {shares.map((share) => {
        const isExpired = new Date(share.expiresAt) < new Date();
        
        return (
          <div 
            key={share.id}
            className="flex items-center gap-4 p-4 bg-surface rounded-lg border border-border"
          >
            <div className="w-10 h-10 bg-surface-dim rounded-lg flex items-center justify-center">
              <Link2 className={`h-5 w-5 ${isExpired ? 'text-text-subtle' : 'text-accent'}`} />
            </div>
            
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-text truncate">
                Document: {share.documentId}
              </p>
              <div className="flex items-center gap-3 mt-1 text-xs text-text-muted">
                <span className="flex items-center gap-1">
                  <Eye className="h-3 w-3" />
                  {share.viewCount} views
                  {share.maxViews && ` / ${share.maxViews}`}
                </span>
                <span>
                  {isExpired ? (
                    <span className="text-danger">Expired</span>
                  ) : (
                    `Expires ${formatDistanceToNow(new Date(share.expiresAt), { addSuffix: true })}`
                  )}
                </span>
              </div>
            </div>
            
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleCopy(share.token, share.id)}
                disabled={isExpired}
              >
                {copiedId === share.id ? (
                  <Check className="h-4 w-4 text-success" />
                ) : (
                  <Copy className="h-4 w-4" />
                )}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => window.open(`/share/${share.token}`, '_blank')}
                disabled={isExpired}
              >
                <ExternalLink className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onDelete(share.id)}
                className="text-danger hover:text-danger"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          </div>
        );
      })}
    </div>
  );
}
