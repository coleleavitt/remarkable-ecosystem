import { useState, useEffect } from 'react';
import { Header } from '../components/layout/Header';
import { ShareList } from '../components/share/ShareList';
import { shares } from '../api/client';
import type { ShareLink } from '../types';
import { Loader2 } from 'lucide-react';

export function SharedPage() {
  const [shareLinks, setShareLinks] = useState<ShareLink[]>([]);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    loadShares();
  }, []);
  
  const loadShares = async () => {
    setLoading(true);
    try {
      const links = await shares.list();
      setShareLinks(links);
    } finally {
      setLoading(false);
    }
  };
  
  const handleDelete = async (id: string) => {
    await shares.delete(id);
    setShareLinks(prev => prev.filter(s => s.id !== id));
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
        title="Shared Links" 
        subtitle={`${shareLinks.length} active links`}
        showCreateFolder={false}
      />
      
      <div className="p-6 max-w-4xl mx-auto">
        <ShareList shares={shareLinks} onDelete={handleDelete} />
      </div>
    </div>
  );
}
