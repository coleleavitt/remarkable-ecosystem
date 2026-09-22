import { useState } from 'react';
import type { TemplateConfig } from '../types';
import { TEMPLATE_PRESETS, CATEGORIES } from '../lib/presets';
import { TemplateThumbnail } from './TemplatePreview';

interface TemplateGalleryProps {
  onSelect: (config: TemplateConfig) => void;
  selectedId?: string;
}

export function TemplateGallery({ onSelect, selectedId }: TemplateGalleryProps) {
  const [activeCategory, setActiveCategory] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');

  const filteredPresets = TEMPLATE_PRESETS.filter((preset) => {
    const matchesCategory = activeCategory === 'all' || preset.category === activeCategory;
    const matchesSearch = preset.name.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  return (
    <div className="flex flex-col h-full">
      {/* Search */}
      <div className="p-3 border-b border-neutral-700">
        <input
          type="text"
          placeholder="Search templates..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full px-3 py-2 text-sm bg-neutral-800 border border-neutral-600 rounded-lg text-neutral-200 placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      {/* Category tabs */}
      <div className="flex gap-1 p-2 border-b border-neutral-700 overflow-x-auto">
        <CategoryTab
          label="All"
          active={activeCategory === 'all'}
          onClick={() => setActiveCategory('all')}
        />
        {CATEGORIES.map((cat) => (
          <CategoryTab
            key={cat}
            label={cat}
            active={activeCategory === cat}
            onClick={() => setActiveCategory(cat)}
          />
        ))}
      </div>

      {/* Template grid */}
      <div className="flex-1 overflow-y-auto p-3">
        <div className="grid grid-cols-3 gap-2">
          {filteredPresets.map((preset) => (
            <TemplateThumbnail
              key={preset.id}
              config={preset.config}
              label={preset.name}
              selected={selectedId === preset.id}
              onClick={() => onSelect(preset.config)}
            />
          ))}
        </div>

        {filteredPresets.length === 0 && (
          <div className="text-center text-neutral-500 py-8">
            No templates found
          </div>
        )}
      </div>
    </div>
  );
}

function CategoryTab({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`
        px-3 py-1.5 text-xs font-medium rounded-md whitespace-nowrap transition-colors
        ${active
          ? 'bg-blue-600 text-white'
          : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-700'
        }
      `}
    >
      {label}
    </button>
  );
}
