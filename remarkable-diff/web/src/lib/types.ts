// CRDT Types matching Rust backend

export interface CrdtTimestamp {
  replica: number
  counter: number
}

export interface DocumentVersion {
  id: string
  document_id: string
  name: string
  timestamp: string
  author_id: number
  pages: PageVersion[]
  operations: CrdtOperationLog[]
}

export interface PageVersion {
  id: string
  index: string
  layers: LayerData[]
  template?: string
}

export interface LayerData {
  id: number
  name?: string
  visible: boolean
  strokes: StrokeData[]
}

export interface StrokeData {
  id: string
  timestamp: CrdtTimestamp
  author_id: number
  pen_type: PenType
  color: StrokeColor
  points: StrokePoint[]
  deleted: boolean
}

export interface StrokePoint {
  x: number
  y: number
  pressure: number
  width: number
  speed: number
}

export type PenType = 
  | 'ball_point' | 'marker' | 'fineliner' | 'sharp_pencil' 
  | 'tilt_pencil' | 'brush' | 'highlighter' | 'eraser'
  | 'erase_area' | 'erase_all' | 'calligraphy' | 'pen'
  | 'selection_brush' | { unknown: number }

export type StrokeColor =
  | 'black' | 'gray' | 'white' | 'yellow' | 'green'
  | 'pink' | 'blue' | 'red' | 'gray_overlap' | { unknown: number }

export interface CrdtOperationLog {
  operation: CrdtOperationType
  timestamp: CrdtTimestamp
  author_id: number
  target_ids: string[]
  data: unknown
  wall_time?: string
}

export type CrdtOperationType =
  | 'add_item' | 'add_items' | 'delete_items' | 'delete_append_items'
  | 'insert_items_after' | 'swap_items' | 'create_group' | 'delete_group'
  | 'add_layer' | 'delete_layer' | 'move_layer' | 'merge_layer_down'
  | 'set_layer_name' | 'set_layer_visible' | 'text_insert' | 'text_remove'

// Diff types
export interface DocumentDiff {
  version_a: VersionInfo
  version_b: VersionInfo
  page_diffs: PageDiff[]
  stats: DiffStats
}

export interface VersionInfo {
  id: string
  name: string
  author_id: number
}

export interface DiffStats {
  strokes_added: number
  strokes_removed: number
  strokes_modified: number
  strokes_unchanged: number
  pages_added: number
  pages_removed: number
  operations_by_type: Record<string, number>
}

export interface PageDiff {
  page_id: string
  status: 'added' | 'removed' | 'modified' | 'unchanged'
  layer_diffs: LayerDiff[]
  stroke_diffs: StrokeDiff[]
}

export interface LayerDiff {
  layer_id: number
  name_a?: string
  name_b?: string
  status: 'added' | 'removed' | 'renamed' | 'unchanged'
}

export interface StrokeDiff {
  stroke_id: string
  layer_id: number
  status: 'added' | 'removed' | 'modified' | 'unchanged'
  stroke_a?: StrokeData
  stroke_b?: StrokeData
  change_author?: number
  change_timestamp?: CrdtTimestamp
}

// Merge types
export interface MergeResult {
  merged: DocumentVersion
  conflicts: MergeConflict[]
  strategy: MergeStrategy
  stats: MergeStats
}

export interface MergeConflict {
  id: string
  conflict_type: 'concurrent_modification' | 'layer_name_conflict' | 'page_order_conflict' | 'group_conflict'
  location: ConflictLocation
  version_a: unknown
  version_b: unknown
  resolution?: ConflictResolution
}

export interface ConflictLocation {
  page_id?: string
  layer_id?: number
  stroke_id?: string
}

export type MergeStrategy = 'last_writer_wins' | 'keep_all' | 'prefer_a' | 'prefer_b' | 'manual'
export type ConflictResolution = 'keep_a' | 'keep_b' | 'keep_both' | 'keep_neither' | 'manual'

export interface MergeStats {
  strokes_from_a: number
  strokes_from_b: number
  strokes_merged: number
  conflicts_auto_resolved: number
  conflicts_manual: number
}

// Timeline types
export interface Timeline {
  events: TimelineEvent[]
  authors: AuthorInfo[]
  time_range: TimeRange
  stats: TimelineStats
}

export interface TimelineEvent {
  index: number
  operation: CrdtOperationLog
  category: string
  change_type: 'addition' | 'deletion' | 'modification' | 'move'
  summary: string
}

export interface AuthorInfo {
  id: number
  color: string
  operation_count: number
  first_seen: CrdtTimestamp
  last_seen: CrdtTimestamp
}

export interface TimeRange {
  start: CrdtTimestamp
  end: CrdtTimestamp
}

export interface TimelineStats {
  total_operations: number
  additions: number
  deletions: number
  modifications: number
  by_category: Record<string, number>
  by_author: Record<number, number>
}

// API response types
export interface DocumentListItem {
  id: string
  name: string
  version_count: number
}

export interface VersionListItem {
  id: string
  name: string
  timestamp: string
  author_id: number
  page_count: number
  operation_count: number
}
