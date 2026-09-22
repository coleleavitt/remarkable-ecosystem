// reMarkable template dimensions
export const DEVICE_WIDTH = 1404;
export const DEVICE_HEIGHT = 1872;
export const DEVICE_PPI = 226;

export type TemplateType = 
  | 'blank'
  | 'lined'
  | 'grid'
  | 'dot-grid'
  | 'cornell'
  | 'calendar'
  | 'music'
  | 'custom';

export type Orientation = 'portrait' | 'landscape';

export interface LineConfig {
  spacing: number;        // pixels between lines
  startY: number;         // first line Y position
  color: string;
  thickness: number;
  style: 'solid' | 'dashed' | 'dotted';
}

export interface GridConfig {
  cellSize: number;       // pixels per cell
  color: string;
  thickness: number;
  style: 'solid' | 'dashed';
  showMinorLines: boolean;
  minorDivisions: number;
  minorColor: string;
}

export interface DotGridConfig {
  spacing: number;        // pixels between dots
  dotSize: number;
  color: string;
}

export interface MarginConfig {
  left: number;
  right: number;
  top: number;
  bottom: number;
  showLines: boolean;
  color: string;
}

export interface CornellConfig {
  cueWidth: number;       // left column width
  summaryHeight: number;  // bottom summary area height
  lineSpacing: number;
  color: string;
}

export interface CalendarConfig {
  month: number;          // 1-12, 0 for blank
  year: number;
  showWeekNumbers: boolean;
  startOnSunday: boolean;
  color: string;
}

export interface MusicConfig {
  staffCount: number;     // number of 5-line staves
  staffSpacing: number;   // pixels between staves
  color: string;
  showClef: boolean;
  showTimeSignature: boolean;
}

export interface CustomPatternConfig {
  svg: string;            // SVG pattern definition
  scale: number;
  color: string;
}

export interface TemplateConfig {
  type: TemplateType;
  orientation: Orientation;
  backgroundColor: string;
  line?: LineConfig;
  grid?: GridConfig;
  dotGrid?: DotGridConfig;
  margin?: MarginConfig;
  cornell?: CornellConfig;
  calendar?: CalendarConfig;
  music?: MusicConfig;
  custom?: CustomPatternConfig;
}

export interface TemplatePreset {
  id: string;
  name: string;
  category: string;
  config: TemplateConfig;
  thumbnail?: string;
}

export interface ExportOptions {
  format: 'png' | 'svg' | 'both';
  filename: string;
  installToDevice: boolean;
  deviceHost: string;
  devicePassword: string;
}
