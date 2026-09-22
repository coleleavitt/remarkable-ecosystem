import { Link } from 'react-router-dom';
import { ChevronRight, Home } from 'lucide-react';

interface BreadcrumbItem {
  id: string;
  name: string;
}

interface BreadcrumbProps {
  items: BreadcrumbItem[];
}

export function Breadcrumb({ items }: BreadcrumbProps) {
  return (
    <nav className="flex items-center gap-1 text-sm">
      <Link 
        to="/" 
        className="p-1 rounded hover:bg-surface-dim text-text-muted hover:text-text transition-colors"
      >
        <Home className="h-4 w-4" />
      </Link>
      
      {items.map((item, index) => (
        <div key={item.id} className="flex items-center gap-1">
          <ChevronRight className="h-4 w-4 text-text-subtle" />
          {index === items.length - 1 ? (
            <span className="px-1 text-text font-medium">{item.name}</span>
          ) : (
            <Link
              to={`/folder/${item.id}`}
              className="px-1 text-text-muted hover:text-text transition-colors"
            >
              {item.name}
            </Link>
          )}
        </div>
      ))}
    </nav>
  );
}
