import { useState, useMemo } from 'react'
import clsx from 'clsx'
import { Filter, User, Layers, Type, ChevronDown } from 'lucide-react'
import type { Timeline, TimelineEvent, AuthorInfo } from '../lib/types'

interface Props {
  timeline: Timeline
}

export default function TimelineView({ timeline }: Props) {
  const [filterAuthor, setFilterAuthor] = useState<number | null>(null)
  const [filterCategory, setFilterCategory] = useState<string | null>(null)
  const [expandedEvents, setExpandedEvents] = useState<Set<number>>(new Set())
  
  const filteredEvents = useMemo(() => {
    return timeline.events.filter((event) => {
      if (filterAuthor !== null && event.operation.author_id !== filterAuthor) {
        return false
      }
      if (filterCategory && event.category !== filterCategory) {
        return false
      }
      return true
    })
  }, [timeline.events, filterAuthor, filterCategory])
  
  const categories = useMemo(() => {
    return [...new Set(timeline.events.map((e) => e.category))]
  }, [timeline.events])
  
  const toggleEvent = (index: number) => {
    const next = new Set(expandedEvents)
    if (next.has(index)) {
      next.delete(index)
    } else {
      next.add(index)
    }
    setExpandedEvents(next)
  }
  
  return (
    <div className="h-full flex">
      {/* Sidebar filters */}
      <aside className="w-64 border-r border-gray-800 p-4 space-y-6">
        <div>
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <Filter className="w-4 h-4" />
            Filters
          </h3>
          
          {/* Author filter */}
          <div className="mb-4">
            <label className="text-sm text-gray-400 mb-1 block">Author</label>
            <select
              value={filterAuthor ?? ''}
              onChange={(e) => setFilterAuthor(e.target.value ? Number(e.target.value) : null)}
              className="w-full bg-gray-800 rounded-lg px-3 py-2 text-sm"
            >
              <option value="">All authors</option>
              {timeline.authors.map((author) => (
                <option key={author.id} value={author.id}>
                  Author {author.id} ({author.operation_count} ops)
                </option>
              ))}
            </select>
          </div>
          
          {/* Category filter */}
          <div>
            <label className="text-sm text-gray-400 mb-1 block">Category</label>
            <select
              value={filterCategory ?? ''}
              onChange={(e) => setFilterCategory(e.target.value || null)}
              className="w-full bg-gray-800 rounded-lg px-3 py-2 text-sm"
            >
              <option value="">All categories</option>
              {categories.map((cat) => (
                <option key={cat} value={cat}>
                  {cat}
                </option>
              ))}
            </select>
          </div>
        </div>
        
        {/* Statistics */}
        <div>
          <h3 className="font-semibold mb-3">Statistics</h3>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Total</span>
              <span>{timeline.stats.total_operations}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-green-400">Additions</span>
              <span>{timeline.stats.additions}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-red-400">Deletions</span>
              <span>{timeline.stats.deletions}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-amber-400">Modifications</span>
              <span>{timeline.stats.modifications}</span>
            </div>
          </div>
        </div>
        
        {/* Authors */}
        <div>
          <h3 className="font-semibold mb-3">Authors</h3>
          <div className="space-y-2">
            {timeline.authors.map((author) => (
              <button
                key={author.id}
                onClick={() => setFilterAuthor(filterAuthor === author.id ? null : author.id)}
                className={clsx(
                  'w-full flex items-center gap-2 px-2 py-1.5 rounded text-sm',
                  filterAuthor === author.id ? 'bg-blue-600' : 'bg-gray-800 hover:bg-gray-700'
                )}
              >
                <div 
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: author.color }}
                />
                <span>Author {author.id}</span>
                <span className="ml-auto text-gray-400">{author.operation_count}</span>
              </button>
            ))}
          </div>
        </div>
      </aside>
      
      {/* Timeline */}
      <main className="flex-1 overflow-auto p-6">
        <h2 className="text-2xl font-bold mb-6">
          CRDT Operation Timeline
          <span className="text-gray-400 text-lg ml-2">
            ({filteredEvents.length} of {timeline.events.length})
          </span>
        </h2>
        
        <div className="space-y-1">
          {filteredEvents.map((event) => (
            <TimelineEventRow
              key={event.index}
              event={event}
              author={timeline.authors.find((a) => a.id === event.operation.author_id)}
              expanded={expandedEvents.has(event.index)}
              onToggle={() => toggleEvent(event.index)}
            />
          ))}
        </div>
      </main>
    </div>
  )
}

function TimelineEventRow({
  event,
  author,
  expanded,
  onToggle,
}: {
  event: TimelineEvent
  author?: AuthorInfo
  expanded: boolean
  onToggle: () => void
}) {
  const icon = event.category === 'layer' ? Layers :
               event.category === 'text' ? Type :
               User
  const Icon = icon
  
  return (
    <div 
      className={clsx(
        'timeline-event cursor-pointer hover:bg-gray-900/50 rounded-lg',
        event.change_type
      )}
      onClick={onToggle}
    >
      <div className="flex items-center gap-3">
        <div 
          className="w-2 h-2 rounded-full flex-shrink-0"
          style={{ backgroundColor: author?.color || '#6b7280' }}
        />
        
        <span className="text-gray-500 text-sm font-mono w-16">
          {event.operation.timestamp.replica}:{event.operation.timestamp.counter}
        </span>
        
        <span className={clsx(
          'px-2 py-0.5 rounded text-xs font-medium',
          event.change_type === 'addition' && 'bg-green-900 text-green-300',
          event.change_type === 'deletion' && 'bg-red-900 text-red-300',
          event.change_type === 'modification' && 'bg-amber-900 text-amber-300',
          event.change_type === 'move' && 'bg-blue-900 text-blue-300'
        )}>
          {event.change_type}
        </span>
        
        <Icon className="w-4 h-4 text-gray-500" />
        
        <span className="flex-1">{event.summary}</span>
        
        <ChevronDown className={clsx(
          'w-4 h-4 transition-transform',
          expanded && 'rotate-180'
        )} />
      </div>
      
      {expanded && (
        <div className="mt-2 ml-6 p-3 bg-gray-800 rounded text-sm font-mono overflow-x-auto">
          <pre>{JSON.stringify(event.operation, null, 2)}</pre>
        </div>
      )}
    </div>
  )
}
