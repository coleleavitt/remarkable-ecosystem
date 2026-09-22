import type { DiffStats as DiffStatsType } from '../lib/types'
import { Plus, Minus, RefreshCw, FileText } from 'lucide-react'

interface Props {
  stats: DiffStatsType
}

export default function DiffStats({ stats }: Props) {
  return (
    <div className="flex items-center gap-4 text-sm">
      <div className="flex items-center gap-1 text-green-400">
        <Plus className="w-4 h-4" />
        <span>{stats.strokes_added}</span>
      </div>
      
      <div className="flex items-center gap-1 text-red-400">
        <Minus className="w-4 h-4" />
        <span>{stats.strokes_removed}</span>
      </div>
      
      <div className="flex items-center gap-1 text-amber-400">
        <RefreshCw className="w-4 h-4" />
        <span>{stats.strokes_modified}</span>
      </div>
      
      <div className="flex items-center gap-1 text-gray-400">
        <FileText className="w-4 h-4" />
        <span>{stats.strokes_unchanged}</span>
      </div>
    </div>
  )
}
