import { useEffect, useRef, useCallback } from 'react';
import type { TemplateConfig } from '../types';
import { TemplateRenderer, getCanvasDimensions } from '../lib/renderer';

interface TemplatePreviewProps {
  config: TemplateConfig;
  scale?: number;
  className?: string;
}

export function TemplatePreview({ config, scale = 0.4, className = '' }: TemplatePreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const render = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const dims = getCanvasDimensions(config.orientation);
    canvas.width = dims.width;
    canvas.height = dims.height;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const renderer = new TemplateRenderer(ctx, dims.width, dims.height);
    renderer.render(config);
  }, [config]);

  useEffect(() => {
    render();
  }, [render]);

  const dims = getCanvasDimensions(config.orientation);

  return (
    <div className={`relative ${className}`}>
      <div 
        className="relative bg-neutral-800 rounded-lg overflow-hidden shadow-2xl"
        style={{
          width: dims.width * scale,
          height: dims.height * scale,
        }}
      >
        <canvas
          ref={canvasRef}
          className="absolute top-0 left-0"
          style={{
            width: dims.width * scale,
            height: dims.height * scale,
            imageRendering: 'crisp-edges',
          }}
        />
        {/* Device frame overlay */}
        <div className="absolute inset-0 pointer-events-none border border-neutral-600 rounded-lg" />
      </div>
      <div className="mt-2 text-center text-neutral-400 text-sm">
        {dims.width} × {dims.height} px @ 226 PPI
      </div>
    </div>
  );
}

// Thumbnail version for gallery
export function TemplateThumbnail({ 
  config, 
  onClick,
  selected,
  label 
}: { 
  config: TemplateConfig; 
  onClick?: () => void;
  selected?: boolean;
  label?: string;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const dims = getCanvasDimensions(config.orientation);
    canvas.width = dims.width;
    canvas.height = dims.height;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const renderer = new TemplateRenderer(ctx, dims.width, dims.height);
    renderer.render(config);
  }, [config]);

  const dims = getCanvasDimensions(config.orientation);
  const scale = 0.12;

  return (
    <button
      onClick={onClick}
      className={`
        group relative flex flex-col items-center p-2 rounded-lg transition-all
        ${selected 
          ? 'bg-blue-600/30 ring-2 ring-blue-500' 
          : 'hover:bg-neutral-800 hover:ring-1 hover:ring-neutral-600'
        }
      `}
    >
      <div 
        className="bg-white rounded shadow-md overflow-hidden"
        style={{
          width: dims.width * scale,
          height: dims.height * scale,
        }}
      >
        <canvas
          ref={canvasRef}
          style={{
            width: dims.width * scale,
            height: dims.height * scale,
          }}
        />
      </div>
      {label && (
        <span className="mt-2 text-xs text-neutral-300 truncate max-w-full">
          {label}
        </span>
      )}
    </button>
  );
}
