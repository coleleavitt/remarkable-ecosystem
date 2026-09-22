import { useMemo } from 'react'
import clsx from 'clsx'
import type { PageDiff, StrokeDiff, StrokeData } from '../lib/types'

interface Props {
  pageDiff: PageDiff
}

const PAGE_WIDTH = 1404
const PAGE_HEIGHT = 1872
const SCALE = 0.4

export default function StrokeDiffView({ pageDiff }: Props) {
  const { added, removed, modified, unchanged } = useMemo(() => {
    return pageDiff.stroke_diffs.reduce(
      (acc, diff) => {
        acc[diff.status].push(diff)
        return acc
      },
      { added: [], removed: [], modified: [], unchanged: [] } as Record<string, StrokeDiff[]>
    )
  }, [pageDiff.stroke_diffs])
  
  return (
    <div className="flex gap-6">
      {/* Version A (left) */}
      <div className="flex-1">
        <h3 className="font-semibold mb-2 text-gray-400">Version A (Base)</h3>
        <div 
          className="bg-white rounded-lg overflow-hidden"
          style={{
            width: PAGE_WIDTH * SCALE,
            height: PAGE_HEIGHT * SCALE,
          }}
        >
          <svg 
            viewBox={`0 0 ${PAGE_WIDTH} ${PAGE_HEIGHT}`}
            className="w-full h-full"
          >
            {/* Unchanged strokes (dimmed) */}
            {unchanged.map((diff) => diff.stroke_a && (
              <StrokePath 
                key={diff.stroke_id} 
                stroke={diff.stroke_a} 
                className="stroke-unchanged"
              />
            ))}
            
            {/* Removed strokes (highlighted in red) */}
            {removed.map((diff) => diff.stroke_a && (
              <StrokePath 
                key={diff.stroke_id} 
                stroke={diff.stroke_a} 
                className="stroke-removed"
              />
            ))}
            
            {/* Modified strokes (original in amber) */}
            {modified.map((diff) => diff.stroke_a && (
              <StrokePath 
                key={diff.stroke_id} 
                stroke={diff.stroke_a} 
                className="stroke-modified"
              />
            ))}
          </svg>
        </div>
      </div>
      
      {/* Version B (right) */}
      <div className="flex-1">
        <h3 className="font-semibold mb-2 text-gray-400">Version B (Changed)</h3>
        <div 
          className="bg-white rounded-lg overflow-hidden"
          style={{
            width: PAGE_WIDTH * SCALE,
            height: PAGE_HEIGHT * SCALE,
          }}
        >
          <svg 
            viewBox={`0 0 ${PAGE_WIDTH} ${PAGE_HEIGHT}`}
            className="w-full h-full"
          >
            {/* Unchanged strokes (dimmed) */}
            {unchanged.map((diff) => diff.stroke_b && (
              <StrokePath 
                key={diff.stroke_id} 
                stroke={diff.stroke_b} 
                className="stroke-unchanged"
              />
            ))}
            
            {/* Added strokes (highlighted in green) */}
            {added.map((diff) => diff.stroke_b && (
              <StrokePath 
                key={diff.stroke_id} 
                stroke={diff.stroke_b} 
                className="stroke-added"
              />
            ))}
            
            {/* Modified strokes (new version in amber) */}
            {modified.map((diff) => diff.stroke_b && (
              <StrokePath 
                key={diff.stroke_id} 
                stroke={diff.stroke_b} 
                className="stroke-modified"
              />
            ))}
          </svg>
        </div>
      </div>
      
      {/* Legend */}
      <div className="w-48 space-y-4">
        <h3 className="font-semibold">Legend</h3>
        
        <div className="space-y-2 text-sm">
          <div className="flex items-center gap-2">
            <div className="w-4 h-1 bg-green-500 rounded"></div>
            <span>Added ({added.length})</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-1 bg-red-500 rounded"></div>
            <span>Removed ({removed.length})</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-1 bg-amber-500 rounded"></div>
            <span>Modified ({modified.length})</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-1 bg-gray-500 rounded"></div>
            <span>Unchanged ({unchanged.length})</span>
          </div>
        </div>
        
        {/* Layer info */}
        <div>
          <h4 className="font-medium mb-2">Layers</h4>
          <div className="space-y-1 text-sm">
            {pageDiff.layer_diffs.map((layer) => (
              <div 
                key={layer.layer_id}
                className={clsx(
                  'px-2 py-1 rounded',
                  layer.status === 'added' && 'bg-green-900/50',
                  layer.status === 'removed' && 'bg-red-900/50',
                  layer.status === 'renamed' && 'bg-amber-900/50',
                  layer.status === 'unchanged' && 'bg-gray-800'
                )}
              >
                Layer {layer.layer_id}: {layer.name_b || layer.name_a || 'Unnamed'}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

function StrokePath({ stroke, className }: { stroke: StrokeData; className: string }) {
  const pathData = useMemo(() => {
    if (stroke.points.length < 2) return ''
    
    const points = stroke.points
    let d = `M ${points[0].x} ${points[0].y}`
    
    for (let i = 1; i < points.length; i++) {
      d += ` L ${points[i].x} ${points[i].y}`
    }
    
    return d
  }, [stroke.points])
  
  const strokeWidth = stroke.points[0]?.width || 2
  
  return (
    <path
      d={pathData}
      fill="none"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    />
  )
}
