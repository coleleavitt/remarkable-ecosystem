import { useState } from 'react';
import { Copy, Check, Calendar, Link2 } from 'lucide-react';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';

interface ShareDialogProps {
  isOpen: boolean;
  onClose: () => void;
  documentName: string;
  onCreateLink: (options: { expiresIn?: number; maxViews?: number }) => Promise<{ url: string }>;
}

export function ShareDialog({ isOpen, onClose, documentName, onCreateLink }: ShareDialogProps) {
  const [shareUrl, setShareUrl] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(false);
  const [expiresIn, setExpiresIn] = useState('24');
  const [maxViews, setMaxViews] = useState('');
  
  const handleCreate = async () => {
    setLoading(true);
    try {
      const result = await onCreateLink({
        expiresIn: parseInt(expiresIn) || undefined,
        maxViews: maxViews ? parseInt(maxViews) : undefined,
      });
      setShareUrl(result.url);
    } finally {
      setLoading(false);
    }
  };
  
  const handleCopy = async () => {
    if (shareUrl) {
      await navigator.clipboard.writeText(shareUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };
  
  const handleClose = () => {
    setShareUrl(null);
    setCopied(false);
    onClose();
  };
  
  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Share Document" size="md">
      <div className="space-y-6">
        <div className="flex items-center gap-3 p-3 bg-surface-dim rounded-lg">
          <div className="w-10 h-10 bg-surface rounded-lg flex items-center justify-center">
            <Link2 className="h-5 w-5 text-text-muted" />
          </div>
          <div>
            <p className="font-medium text-text">{documentName}</p>
            <p className="text-xs text-text-muted">Create a shareable link</p>
          </div>
        </div>
        
        {!shareUrl ? (
          <>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Input
                  label="Expires in (hours)"
                  type="number"
                  value={expiresIn}
                  onChange={(e) => setExpiresIn(e.target.value)}
                  placeholder="24"
                />
              </div>
              <div>
                <Input
                  label="Max views (optional)"
                  type="number"
                  value={maxViews}
                  onChange={(e) => setMaxViews(e.target.value)}
                  placeholder="Unlimited"
                />
              </div>
            </div>
            
            <Button 
              onClick={handleCreate} 
              loading={loading}
              className="w-full"
            >
              Create Share Link
            </Button>
          </>
        ) : (
          <div className="space-y-4">
            <div className="flex gap-2">
              <input
                type="text"
                value={shareUrl}
                readOnly
                className="flex-1 input text-sm font-mono"
              />
              <Button onClick={handleCopy} variant="secondary">
                {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
              </Button>
            </div>
            
            <p className="text-xs text-text-muted flex items-center gap-1">
              <Calendar className="h-3 w-3" />
              Expires in {expiresIn} hours
              {maxViews && ` • Limited to ${maxViews} views`}
            </p>
            
            <Button onClick={handleClose} variant="secondary" className="w-full">
              Done
            </Button>
          </div>
        )}
      </div>
    </Modal>
  );
}
