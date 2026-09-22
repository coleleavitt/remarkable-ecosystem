import { useState, useRef, useEffect, type ReactNode } from 'react';
import { clsx } from 'clsx';

interface DropdownItem {
  label: string;
  icon?: ReactNode;
  onClick: () => void;
  danger?: boolean;
  disabled?: boolean;
}

interface DropdownProps {
  trigger: ReactNode;
  items: DropdownItem[];
  align?: 'left' | 'right';
}

export function Dropdown({ trigger, items, align = 'right' }: DropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);
  
  return (
    <div ref={ref} className="relative">
      <div onClick={() => setIsOpen(!isOpen)}>
        {trigger}
      </div>
      
      {isOpen && (
        <div className={clsx(
          'absolute z-50 mt-1 w-48 py-1 bg-surface rounded-lg border border-border shadow-lg',
          'animate-in fade-in slide-in-from-top-1 duration-150',
          align === 'right' ? 'right-0' : 'left-0'
        )}>
          {items.map((item, index) => (
            <button
              key={index}
              onClick={() => {
                if (!item.disabled) {
                  item.onClick();
                  setIsOpen(false);
                }
              }}
              disabled={item.disabled}
              className={clsx(
                'w-full flex items-center gap-2 px-3 py-2 text-sm text-left',
                'transition-colors',
                item.disabled
                  ? 'text-text-subtle cursor-not-allowed'
                  : item.danger
                    ? 'text-danger hover:bg-red-50'
                    : 'text-text hover:bg-surface-dim'
              )}
            >
              {item.icon}
              {item.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
