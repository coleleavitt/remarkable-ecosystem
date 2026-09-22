import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import clsx from 'clsx'
import { 
  GitMerge, AlertTriangle, CheckCircle, Download, 
  ChevronRight, ArrowLeft, ArrowRight 
} from 'lucide-react'
import * as api from '../lib/api'
import type { MergeResult, MergeConflict, ConflictResolution } from '../lib/types'

interface Props {
  mergeId: string
  result: MergeResult
  onUpdate: (result: MergeResult) => void
}

export default function MergePanel({ mergeId, result, onUpdate }: Props) {
  const [selectedConflict, setSelectedConflict] = useState<string | null>(null)
  
  const resolveMutation = useMutation({
    mutationFn: ({ conflictId, resolution }: { conflictId: string; resolution: ConflictResolution }) =>
      api.resolveConflict(mergeId, conflictId, resolution),
    onSuccess: onUpdate,
  })
  
  const unresolvedConflicts = result.conflicts.filter((c) => !c.resolution)
  const resolvedConflicts = result.conflicts.filter((c) => c.resolution)
  
  const handleResolve = (conflictId: string, resolution: ConflictResolution) => {
    resolveMutation.mutate({ conflictId, resolution })
  }
  
  const handleExport = async () => {
    try {
      const merged = await api.exportMerged(mergeId)
      const blob = new Blob([JSON.stringify(merged, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${result.merged.name}.json`
      a.click()
      URL.revokeObjectURL(url)
    } catch (error) {
      console.error('Export failed:', error)
    }
  }
  
  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <header className="p-4 border-b border-gray-800 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <GitMerge className="w-6 h-6 text-green-400" />
          <div>
            <h2 className="text-xl font-bold">{result.merged.name}</h2>
            <p className="text-sm text-gray-400">
              Strategy: {result.strategy.replace(/_/g, ' ')}
            </p>
          </div>
        </div>
        
        <div className="flex items-center gap-4">
          {/* Stats */}
          <div className="flex items-center gap-6 text-sm">
            <div className="text-center">
              <div className="font-bold text-lg">{result.stats.strokes_from_a}</div>
              <div className="text-gray-400">From A</div>
            </div>
            <div className="text-center">
              <div className="font-bold text-lg">{result.stats.strokes_from_b}</div>
              <div className="text-gray-400">From B</div>
            </div>
            <div className="text-center">
              <div className="font-bold text-lg">{result.stats.strokes_merged}</div>
              <div className="text-gray-400">Merged</div>
            </div>
          </div>
          
          {/* Export button */}
          <button
            onClick={handleExport}
            disabled={unresolvedConflicts.length > 0}
            className={clsx(
              'px-4 py-2 rounded-lg flex items-center gap-2',
              unresolvedConflicts.length > 0 
                ? 'bg-gray-700 text-gray-400 cursor-not-allowed'
                : 'bg-green-600 hover:bg-green-700'
            )}
          >
            <Download className="w-4 h-4" />
            Export Merged
          </button>
        </div>
      </header>
      
      <div className="flex-1 flex overflow-hidden">
        {/* Conflicts list */}
        <aside className="w-80 border-r border-gray-800 overflow-auto">
          <div className="p-4">
            {unresolvedConflicts.length > 0 && (
              <div className="mb-6">
                <h3 className="font-semibold flex items-center gap-2 mb-3 text-amber-400">
                  <AlertTriangle className="w-4 h-4" />
                  Unresolved ({unresolvedConflicts.length})
                </h3>
                <div className="space-y-2">
                  {unresolvedConflicts.map((conflict) => (
                    <ConflictCard
                      key={conflict.id}
                      conflict={conflict}
                      selected={selectedConflict === conflict.id}
                      onClick={() => setSelectedConflict(conflict.id)}
                    />
                  ))}
                </div>
              </div>
            )}
            
            {resolvedConflicts.length > 0 && (
              <div>
                <h3 className="font-semibold flex items-center gap-2 mb-3 text-green-400">
                  <CheckCircle className="w-4 h-4" />
                  Resolved ({resolvedConflicts.length})
                </h3>
                <div className="space-y-2">
                  {resolvedConflicts.map((conflict) => (
                    <ConflictCard
                      key={conflict.id}
                      conflict={conflict}
                      selected={selectedConflict === conflict.id}
                      onClick={() => setSelectedConflict(conflict.id)}
                    />
                  ))}
                </div>
              </div>
            )}
            
            {result.conflicts.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                <CheckCircle className="w-12 h-12 mx-auto mb-3 text-green-400" />
                <p>No conflicts!</p>
                <p className="text-sm">All changes merged automatically.</p>
              </div>
            )}
          </div>
        </aside>
        
        {/* Conflict detail */}
        <main className="flex-1 overflow-auto p-6">
          {selectedConflict ? (
            <ConflictDetail
              conflict={result.conflicts.find((c) => c.id === selectedConflict)!}
              onResolve={handleResolve}
              isResolving={resolveMutation.isPending}
            />
          ) : (
            <div className="h-full flex items-center justify-center text-gray-500">
              Select a conflict to view details
            </div>
          )}
        </main>
      </div>
    </div>
  )
}

function ConflictCard({
  conflict,
  selected,
  onClick,
}: {
  conflict: MergeConflict
  selected: boolean
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className={clsx(
        'w-full px-3 py-2 rounded-lg text-left flex items-center gap-2',
        selected ? 'bg-blue-600' : 'bg-gray-800 hover:bg-gray-700'
      )}
    >
      {conflict.resolution ? (
        <CheckCircle className="w-4 h-4 text-green-400 flex-shrink-0" />
      ) : (
        <AlertTriangle className="w-4 h-4 text-amber-400 flex-shrink-0" />
      )}
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium truncate">
          {conflict.conflict_type.replace(/_/g, ' ')}
        </div>
        {conflict.location.stroke_id && (
          <div className="text-xs text-gray-400 truncate">
            Stroke: {conflict.location.stroke_id}
          </div>
        )}
      </div>
      <ChevronRight className="w-4 h-4 text-gray-500" />
    </button>
  )
}

function ConflictDetail({
  conflict,
  onResolve,
  isResolving,
}: {
  conflict: MergeConflict
  onResolve: (conflictId: string, resolution: ConflictResolution) => void
  isResolving: boolean
}) {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-2">
          {conflict.conflict_type.replace(/_/g, ' ')}
        </h3>
        <p className="text-gray-400 text-sm">
          {conflict.resolution 
            ? `Resolved: ${conflict.resolution.replace(/_/g, ' ')}`
            : 'Choose how to resolve this conflict'}
        </p>
      </div>
      
      {/* Side by side comparison */}
      <div className="grid grid-cols-2 gap-6">
        <div className="bg-gray-800 rounded-lg p-4">
          <h4 className="font-medium mb-2 flex items-center gap-2">
            <ArrowLeft className="w-4 h-4" />
            Version A
          </h4>
          <pre className="text-sm font-mono bg-gray-900 p-3 rounded overflow-auto max-h-64">
            {JSON.stringify(conflict.version_a, null, 2)}
          </pre>
        </div>
        
        <div className="bg-gray-800 rounded-lg p-4">
          <h4 className="font-medium mb-2 flex items-center gap-2">
            <ArrowRight className="w-4 h-4" />
            Version B
          </h4>
          <pre className="text-sm font-mono bg-gray-900 p-3 rounded overflow-auto max-h-64">
            {JSON.stringify(conflict.version_b, null, 2)}
          </pre>
        </div>
      </div>
      
      {/* Resolution buttons */}
      {!conflict.resolution && (
        <div className="flex gap-3">
          <button
            onClick={() => onResolve(conflict.id, 'keep_a')}
            disabled={isResolving}
            className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg disabled:opacity-50"
          >
            Keep A
          </button>
          <button
            onClick={() => onResolve(conflict.id, 'keep_b')}
            disabled={isResolving}
            className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg disabled:opacity-50"
          >
            Keep B
          </button>
          <button
            onClick={() => onResolve(conflict.id, 'keep_both')}
            disabled={isResolving}
            className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg disabled:opacity-50"
          >
            Keep Both
          </button>
          <button
            onClick={() => onResolve(conflict.id, 'keep_neither')}
            disabled={isResolving}
            className="flex-1 px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg disabled:opacity-50"
          >
            Keep Neither
          </button>
        </div>
      )}
    </div>
  )
}
