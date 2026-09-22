import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { 
  FileUp, GitCompare, GitMerge, Clock, Layers, 
  Plus, Minus, RefreshCw
} from 'lucide-react'
import clsx from 'clsx'
import * as api from './lib/api'
import type { 
  DocumentListItem, VersionListItem, DocumentDiff,
  MergeResult, MergeStrategy
} from './lib/types'

import StrokeDiffView from './components/StrokeDiffView'
import TimelineView from './components/TimelineView'
import MergePanel from './components/MergePanel'
import DiffStats from './components/DiffStats'
import UploadPanel from './components/UploadPanel'

type View = 'documents' | 'diff' | 'timeline' | 'merge'

export default function App() {
  const [view, setView] = useState<View>('documents')
  const [selectedDoc, setSelectedDoc] = useState<string | null>(null)
  const [selectedVersions, setSelectedVersions] = useState<[string, string] | null>(null)
  const [currentDiff, setCurrentDiff] = useState<{ id: string; diff: DocumentDiff } | null>(null)
  const [currentMerge, setCurrentMerge] = useState<{ id: string; result: MergeResult } | null>(null)
  const [showUpload, setShowUpload] = useState(false)
  
  const queryClient = useQueryClient()
  
  const { data: documents = [] } = useQuery({
    queryKey: ['documents'],
    queryFn: api.listDocuments,
  })
  
  const { data: versions = [] } = useQuery({
    queryKey: ['versions', selectedDoc],
    queryFn: () => selectedDoc ? api.listVersions(selectedDoc) : Promise.resolve([]),
    enabled: !!selectedDoc,
  })
  
  const { data: timeline } = useQuery({
    queryKey: ['timeline', selectedDoc],
    queryFn: () => selectedDoc ? api.getTimeline(selectedDoc) : Promise.resolve(null),
    enabled: !!selectedDoc && view === 'timeline',
  })
  
  const computeDiffMutation = useMutation({
    mutationFn: (args: { versionA: string; versionB: string }) =>
      api.computeDiff(selectedDoc!, args.versionA, args.versionB),
    onSuccess: (data) => {
      setCurrentDiff(data)
      setView('diff')
    },
  })
  
  const mergeMutation = useMutation({
    mutationFn: (args: { versionA: string; versionB: string; strategy: MergeStrategy }) =>
      api.mergeVersions(selectedDoc!, args.versionA, args.versionB, args.strategy),
    onSuccess: (data) => {
      setCurrentMerge(data)
      setView('merge')
    },
  })
  
  const handleVersionSelect = (versionId: string) => {
    if (!selectedVersions) {
      setSelectedVersions([versionId, ''])
    } else if (selectedVersions[1] === '') {
      setSelectedVersions([selectedVersions[0], versionId])
    } else {
      setSelectedVersions([versionId, ''])
    }
  }
  
  const handleComputeDiff = () => {
    if (selectedVersions && selectedVersions[0] && selectedVersions[1]) {
      computeDiffMutation.mutate({
        versionA: selectedVersions[0],
        versionB: selectedVersions[1],
      })
    }
  }
  
  const handleMerge = (strategy: MergeStrategy) => {
    if (selectedVersions && selectedVersions[0] && selectedVersions[1]) {
      mergeMutation.mutate({
        versionA: selectedVersions[0],
        versionB: selectedVersions[1],
        strategy,
      })
    }
  }
  
  return (
    <div className="min-h-screen flex">
      {/* Sidebar */}
      <aside className="w-64 bg-gray-900 border-r border-gray-800 flex flex-col">
        <header className="p-4 border-b border-gray-800">
          <h1 className="text-xl font-bold text-white flex items-center gap-2">
            <GitCompare className="w-5 h-5" />
            reMarkable Diff
          </h1>
        </header>
        
        <nav className="flex-1 p-4 space-y-2">
          <button
            onClick={() => setView('documents')}
            className={clsx(
              'w-full px-3 py-2 rounded-lg text-left flex items-center gap-2',
              view === 'documents' ? 'bg-blue-600' : 'hover:bg-gray-800'
            )}
          >
            <Layers className="w-4 h-4" />
            Documents
          </button>
          
          <button
            onClick={() => setView('diff')}
            disabled={!currentDiff}
            className={clsx(
              'w-full px-3 py-2 rounded-lg text-left flex items-center gap-2',
              view === 'diff' ? 'bg-blue-600' : 'hover:bg-gray-800',
              !currentDiff && 'opacity-50 cursor-not-allowed'
            )}
          >
            <GitCompare className="w-4 h-4" />
            Diff View
          </button>
          
          <button
            onClick={() => setView('timeline')}
            disabled={!selectedDoc}
            className={clsx(
              'w-full px-3 py-2 rounded-lg text-left flex items-center gap-2',
              view === 'timeline' ? 'bg-blue-600' : 'hover:bg-gray-800',
              !selectedDoc && 'opacity-50 cursor-not-allowed'
            )}
          >
            <Clock className="w-4 h-4" />
            Timeline
          </button>
          
          <button
            onClick={() => setView('merge')}
            disabled={!currentMerge}
            className={clsx(
              'w-full px-3 py-2 rounded-lg text-left flex items-center gap-2',
              view === 'merge' ? 'bg-blue-600' : 'hover:bg-gray-800',
              !currentMerge && 'opacity-50 cursor-not-allowed'
            )}
          >
            <GitMerge className="w-4 h-4" />
            Merge
          </button>
        </nav>
        
        <footer className="p-4 border-t border-gray-800">
          <button
            onClick={() => setShowUpload(true)}
            className="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg flex items-center justify-center gap-2"
          >
            <FileUp className="w-4 h-4" />
            Upload Document
          </button>
        </footer>
      </aside>
      
      {/* Main content */}
      <main className="flex-1 overflow-auto">
        {view === 'documents' && (
          <DocumentsView
            documents={documents}
            selectedDoc={selectedDoc}
            versions={versions}
            selectedVersions={selectedVersions}
            onSelectDoc={setSelectedDoc}
            onSelectVersion={handleVersionSelect}
            onComputeDiff={handleComputeDiff}
            onMerge={handleMerge}
            isComputing={computeDiffMutation.isPending}
            isMerging={mergeMutation.isPending}
          />
        )}
        
        {view === 'diff' && currentDiff && (
          <DiffView diff={currentDiff.diff} />
        )}
        
        {view === 'timeline' && timeline && (
          <TimelineView timeline={timeline} />
        )}
        
        {view === 'merge' && currentMerge && (
          <MergePanel 
            mergeId={currentMerge.id}
            result={currentMerge.result}
            onUpdate={(result) => setCurrentMerge({ ...currentMerge, result })}
          />
        )}
      </main>
      
      {/* Upload modal */}
      {showUpload && (
        <UploadPanel
          onClose={() => setShowUpload(false)}
          onSuccess={() => {
            setShowUpload(false)
            queryClient.invalidateQueries({ queryKey: ['documents'] })
          }}
        />
      )}
    </div>
  )
}

function DocumentsView({
  documents,
  selectedDoc,
  versions,
  selectedVersions,
  onSelectDoc,
  onSelectVersion,
  onComputeDiff,
  onMerge,
  isComputing,
  isMerging,
}: {
  documents: DocumentListItem[]
  selectedDoc: string | null
  versions: VersionListItem[]
  selectedVersions: [string, string] | null
  onSelectDoc: (id: string) => void
  onSelectVersion: (id: string) => void
  onComputeDiff: () => void
  onMerge: (strategy: MergeStrategy) => void
  isComputing: boolean
  isMerging: boolean
}) {
  const canDiff = selectedVersions && selectedVersions[0] && selectedVersions[1]
  
  return (
    <div className="p-6">
      <h2 className="text-2xl font-bold mb-6">Documents</h2>
      
      <div className="grid grid-cols-2 gap-6">
        {/* Document list */}
        <div className="bg-gray-900 rounded-lg p-4">
          <h3 className="font-semibold mb-4">Available Documents</h3>
          
          {documents.length === 0 ? (
            <p className="text-gray-500 text-center py-8">
              No documents uploaded yet
            </p>
          ) : (
            <div className="space-y-2">
              {documents.map((doc) => (
                <button
                  key={doc.id}
                  onClick={() => onSelectDoc(doc.id)}
                  className={clsx(
                    'w-full px-4 py-3 rounded-lg text-left flex items-center justify-between',
                    selectedDoc === doc.id ? 'bg-blue-600' : 'bg-gray-800 hover:bg-gray-700'
                  )}
                >
                  <span>{doc.name}</span>
                  <span className="text-sm text-gray-400">
                    {doc.version_count} version{doc.version_count !== 1 ? 's' : ''}
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>
        
        {/* Version selection */}
        <div className="bg-gray-900 rounded-lg p-4">
          <h3 className="font-semibold mb-4">Select Versions to Compare</h3>
          
          {versions.length === 0 ? (
            <p className="text-gray-500 text-center py-8">
              {selectedDoc ? 'No versions found' : 'Select a document first'}
            </p>
          ) : (
            <>
              <div className="space-y-2 mb-6">
                {versions.map((v) => {
                  const isSelected = selectedVersions?.includes(v.id)
                  const position = selectedVersions?.[0] === v.id ? 'A' : 
                                   selectedVersions?.[1] === v.id ? 'B' : null
                  
                  return (
                    <button
                      key={v.id}
                      onClick={() => onSelectVersion(v.id)}
                      className={clsx(
                        'w-full px-4 py-3 rounded-lg text-left flex items-center gap-3',
                        isSelected ? 'bg-blue-600' : 'bg-gray-800 hover:bg-gray-700'
                      )}
                    >
                      {position && (
                        <span className="w-6 h-6 rounded-full bg-white/20 flex items-center justify-center text-sm font-bold">
                          {position}
                        </span>
                      )}
                      <div className="flex-1">
                        <div className="font-medium">{v.name}</div>
                        <div className="text-sm text-gray-400">
                          {new Date(v.timestamp).toLocaleString()} · 
                          {v.page_count} pages · 
                          {v.operation_count} ops
                        </div>
                      </div>
                    </button>
                  )
                })}
              </div>
              
              {canDiff && (
                <div className="flex gap-3">
                  <button
                    onClick={onComputeDiff}
                    disabled={isComputing}
                    className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg flex items-center justify-center gap-2 disabled:opacity-50"
                  >
                    {isComputing ? (
                      <RefreshCw className="w-4 h-4 animate-spin" />
                    ) : (
                      <GitCompare className="w-4 h-4" />
                    )}
                    Compute Diff
                  </button>
                  
                  <button
                    onClick={() => onMerge('last_writer_wins')}
                    disabled={isMerging}
                    className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg flex items-center justify-center gap-2 disabled:opacity-50"
                  >
                    {isMerging ? (
                      <RefreshCw className="w-4 h-4 animate-spin" />
                    ) : (
                      <GitMerge className="w-4 h-4" />
                    )}
                    Merge
                  </button>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  )
}

function DiffView({ diff }: { diff: DocumentDiff }) {
  const [selectedPage, setSelectedPage] = useState(0)
  
  const pageDiff = diff.page_diffs[selectedPage]
  
  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <header className="p-4 border-b border-gray-800 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-xl font-bold">Diff: {diff.version_a.name} → {diff.version_b.name}</h2>
        </div>
        <DiffStats stats={diff.stats} />
      </header>
      
      {/* Page tabs */}
      <div className="border-b border-gray-800 px-4">
        <div className="flex gap-2 overflow-x-auto py-2">
          {diff.page_diffs.map((page, idx) => (
            <button
              key={page.page_id}
              onClick={() => setSelectedPage(idx)}
              className={clsx(
                'px-3 py-1.5 rounded-lg text-sm whitespace-nowrap flex items-center gap-2',
                selectedPage === idx ? 'bg-blue-600' : 'bg-gray-800 hover:bg-gray-700',
                page.status === 'added' && 'ring-2 ring-green-500',
                page.status === 'removed' && 'ring-2 ring-red-500',
                page.status === 'modified' && 'ring-2 ring-amber-500'
              )}
            >
              Page {idx + 1}
              {page.status === 'added' && <Plus className="w-3 h-3 text-green-400" />}
              {page.status === 'removed' && <Minus className="w-3 h-3 text-red-400" />}
            </button>
          ))}
        </div>
      </div>
      
      {/* Diff content */}
      {pageDiff && (
        <div className="flex-1 overflow-auto p-4">
          <StrokeDiffView pageDiff={pageDiff} />
        </div>
      )}
    </div>
  )
}
