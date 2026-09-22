import type {
  TemplateConfig,
  LineConfig,
  GridConfig,
  DotGridConfig,
  MarginConfig,
  CornellConfig,
  CalendarConfig,
  MusicConfig,
  CustomPatternConfig,
} from '../types';
import { DEVICE_WIDTH, DEVICE_HEIGHT } from '../types';

export function getCanvasDimensions(orientation: 'portrait' | 'landscape') {
  return orientation === 'portrait'
    ? { width: DEVICE_WIDTH, height: DEVICE_HEIGHT }
    : { width: DEVICE_HEIGHT, height: DEVICE_WIDTH };
}

export class TemplateRenderer {
  private ctx: CanvasRenderingContext2D;
  private width: number;
  private height: number;

  constructor(ctx: CanvasRenderingContext2D, width: number, height: number) {
    this.ctx = ctx;
    this.width = width;
    this.height = height;
  }

  clear(color: string = '#ffffff') {
    this.ctx.fillStyle = color;
    this.ctx.fillRect(0, 0, this.width, this.height);
  }

  render(config: TemplateConfig) {
    const dims = getCanvasDimensions(config.orientation);
    this.width = dims.width;
    this.height = dims.height;

    // Background
    this.clear(config.backgroundColor);

    // Margins first (if applicable)
    if (config.margin?.showLines) {
      this.drawMargins(config.margin);
    }

    // Main pattern
    switch (config.type) {
      case 'blank':
        // Nothing to draw
        break;
      case 'lined':
        if (config.line) this.drawLines(config.line, config.margin);
        break;
      case 'grid':
        if (config.grid) this.drawGrid(config.grid, config.margin);
        break;
      case 'dot-grid':
        if (config.dotGrid) this.drawDotGrid(config.dotGrid, config.margin);
        break;
      case 'cornell':
        if (config.cornell) this.drawCornell(config.cornell, config.margin);
        break;
      case 'calendar':
        if (config.calendar) this.drawCalendar(config.calendar);
        break;
      case 'music':
        if (config.music) this.drawMusic(config.music, config.margin);
        break;
      case 'custom':
        if (config.custom) this.drawCustomPattern(config.custom);
        break;
    }
  }

  private drawMargins(margin: MarginConfig) {
    this.ctx.strokeStyle = margin.color;
    this.ctx.lineWidth = 1;
    this.ctx.setLineDash([]);

    // Left margin line
    if (margin.left > 0) {
      this.ctx.beginPath();
      this.ctx.moveTo(margin.left, 0);
      this.ctx.lineTo(margin.left, this.height);
      this.ctx.stroke();
    }

    // Top margin line
    if (margin.top > 0) {
      this.ctx.beginPath();
      this.ctx.moveTo(0, margin.top);
      this.ctx.lineTo(this.width, margin.top);
      this.ctx.stroke();
    }
  }

  private drawLines(line: LineConfig, margin?: MarginConfig) {
    const startX = margin?.left ?? 0;
    const endX = this.width - (margin?.right ?? 0);
    const startY = line.startY;
    const endY = this.height - (margin?.bottom ?? 0);

    this.ctx.strokeStyle = line.color;
    this.ctx.lineWidth = line.thickness;

    switch (line.style) {
      case 'dashed':
        this.ctx.setLineDash([8, 4]);
        break;
      case 'dotted':
        this.ctx.setLineDash([2, 4]);
        break;
      default:
        this.ctx.setLineDash([]);
    }

    for (let y = startY; y <= endY; y += line.spacing) {
      this.ctx.beginPath();
      this.ctx.moveTo(startX, y);
      this.ctx.lineTo(endX, y);
      this.ctx.stroke();
    }

    this.ctx.setLineDash([]);
  }

  private drawGrid(grid: GridConfig, margin?: MarginConfig) {
    const startX = margin?.left ?? 0;
    const startY = margin?.top ?? 0;
    const endX = this.width - (margin?.right ?? 0);
    const endY = this.height - (margin?.bottom ?? 0);

    // Minor lines first (if enabled)
    if (grid.showMinorLines && grid.minorDivisions > 1) {
      const minorSize = grid.cellSize / grid.minorDivisions;
      this.ctx.strokeStyle = grid.minorColor;
      this.ctx.lineWidth = 0.5;
      this.ctx.setLineDash([]);

      // Vertical minor lines
      for (let x = startX; x <= endX; x += minorSize) {
        if ((x - startX) % grid.cellSize !== 0) {
          this.ctx.beginPath();
          this.ctx.moveTo(x, startY);
          this.ctx.lineTo(x, endY);
          this.ctx.stroke();
        }
      }

      // Horizontal minor lines
      for (let y = startY; y <= endY; y += minorSize) {
        if ((y - startY) % grid.cellSize !== 0) {
          this.ctx.beginPath();
          this.ctx.moveTo(startX, y);
          this.ctx.lineTo(endX, y);
          this.ctx.stroke();
        }
      }
    }

    // Major grid lines
    this.ctx.strokeStyle = grid.color;
    this.ctx.lineWidth = grid.thickness;
    this.ctx.setLineDash(grid.style === 'dashed' ? [6, 3] : []);

    // Vertical lines
    for (let x = startX; x <= endX; x += grid.cellSize) {
      this.ctx.beginPath();
      this.ctx.moveTo(x, startY);
      this.ctx.lineTo(x, endY);
      this.ctx.stroke();
    }

    // Horizontal lines
    for (let y = startY; y <= endY; y += grid.cellSize) {
      this.ctx.beginPath();
      this.ctx.moveTo(startX, y);
      this.ctx.lineTo(endX, y);
      this.ctx.stroke();
    }

    this.ctx.setLineDash([]);
  }

  private drawDotGrid(dotGrid: DotGridConfig, margin?: MarginConfig) {
    const startX = (margin?.left ?? 0) + dotGrid.spacing / 2;
    const startY = (margin?.top ?? 0) + dotGrid.spacing / 2;
    const endX = this.width - (margin?.right ?? 0);
    const endY = this.height - (margin?.bottom ?? 0);

    this.ctx.fillStyle = dotGrid.color;

    for (let y = startY; y < endY; y += dotGrid.spacing) {
      for (let x = startX; x < endX; x += dotGrid.spacing) {
        this.ctx.beginPath();
        this.ctx.arc(x, y, dotGrid.dotSize, 0, Math.PI * 2);
        this.ctx.fill();
      }
    }
  }

  private drawCornell(cornell: CornellConfig, margin?: MarginConfig) {
    const left = margin?.left ?? 40;
    const right = margin?.right ?? 40;
    const top = margin?.top ?? 100;
    const bottom = margin?.bottom ?? 40;

    this.ctx.strokeStyle = cornell.color;
    this.ctx.lineWidth = 2;
    this.ctx.setLineDash([]);

    // Title area (top)
    this.ctx.beginPath();
    this.ctx.moveTo(left, top);
    this.ctx.lineTo(this.width - right, top);
    this.ctx.stroke();

    // Cue column divider
    this.ctx.beginPath();
    this.ctx.moveTo(left + cornell.cueWidth, top);
    this.ctx.lineTo(left + cornell.cueWidth, this.height - bottom - cornell.summaryHeight);
    this.ctx.stroke();

    // Summary area divider
    this.ctx.beginPath();
    this.ctx.moveTo(left, this.height - bottom - cornell.summaryHeight);
    this.ctx.lineTo(this.width - right, this.height - bottom - cornell.summaryHeight);
    this.ctx.stroke();

    // Lines in notes area
    this.ctx.strokeStyle = cornell.color;
    this.ctx.lineWidth = 1;
    const notesStartX = left + cornell.cueWidth + 10;
    const notesEndX = this.width - right;
    const notesStartY = top + cornell.lineSpacing;
    const notesEndY = this.height - bottom - cornell.summaryHeight - 10;

    for (let y = notesStartY; y < notesEndY; y += cornell.lineSpacing) {
      this.ctx.beginPath();
      this.ctx.moveTo(notesStartX, y);
      this.ctx.lineTo(notesEndX, y);
      this.ctx.stroke();
    }

    // Lines in summary area
    const summaryStartY = this.height - bottom - cornell.summaryHeight + cornell.lineSpacing;
    for (let y = summaryStartY; y < this.height - bottom; y += cornell.lineSpacing) {
      this.ctx.beginPath();
      this.ctx.moveTo(left, y);
      this.ctx.lineTo(this.width - right, y);
      this.ctx.stroke();
    }

    // Labels
    this.ctx.fillStyle = '#888888';
    this.ctx.font = '20px system-ui, sans-serif';
    this.ctx.fillText('TITLE', left + 10, top - 20);
    this.ctx.fillText('CUES', left + 10, top + 40);
    this.ctx.fillText('NOTES', notesStartX + 10, top + 40);
    this.ctx.fillText('SUMMARY', left + 10, this.height - bottom - cornell.summaryHeight + 30);
  }

  private drawCalendar(calendar: CalendarConfig) {
    const margin = 60;
    const headerHeight = 100;
    const cellWidth = (this.width - margin * 2) / 7;
    const cellHeight = (this.height - margin * 2 - headerHeight) / 6;

    this.ctx.strokeStyle = calendar.color;
    this.ctx.fillStyle = calendar.color;
    this.ctx.lineWidth = 1;
    this.ctx.setLineDash([]);

    // Week header
    const days = calendar.startOnSunday
      ? ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
      : ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

    this.ctx.font = 'bold 28px system-ui, sans-serif';
    this.ctx.textAlign = 'center';

    // Month/Year header if specified
    if (calendar.month > 0) {
      const monthNames = [
        '', 'January', 'February', 'March', 'April', 'May', 'June',
        'July', 'August', 'September', 'October', 'November', 'December'
      ];
      this.ctx.font = 'bold 48px system-ui, sans-serif';
      this.ctx.fillText(
        `${monthNames[calendar.month]} ${calendar.year}`,
        this.width / 2,
        margin + 50
      );
    }

    // Day headers
    this.ctx.font = 'bold 28px system-ui, sans-serif';
    days.forEach((day, i) => {
      this.ctx.fillText(
        day,
        margin + i * cellWidth + cellWidth / 2,
        margin + headerHeight - 20
      );
    });

    // Grid lines
    for (let row = 0; row <= 6; row++) {
      this.ctx.beginPath();
      this.ctx.moveTo(margin, margin + headerHeight + row * cellHeight);
      this.ctx.lineTo(this.width - margin, margin + headerHeight + row * cellHeight);
      this.ctx.stroke();
    }

    for (let col = 0; col <= 7; col++) {
      this.ctx.beginPath();
      this.ctx.moveTo(margin + col * cellWidth, margin + headerHeight);
      this.ctx.lineTo(margin + col * cellWidth, margin + headerHeight + 6 * cellHeight);
      this.ctx.stroke();
    }

    // Fill in dates if month specified
    if (calendar.month > 0) {
      const firstDay = new Date(calendar.year, calendar.month - 1, 1);
      const lastDay = new Date(calendar.year, calendar.month, 0);
      const startDayOfWeek = calendar.startOnSunday
        ? firstDay.getDay()
        : (firstDay.getDay() + 6) % 7;

      this.ctx.font = '32px system-ui, sans-serif';
      this.ctx.textAlign = 'left';

      for (let date = 1; date <= lastDay.getDate(); date++) {
        const dayIndex = startDayOfWeek + date - 1;
        const row = Math.floor(dayIndex / 7);
        const col = dayIndex % 7;

        this.ctx.fillText(
          String(date),
          margin + col * cellWidth + 10,
          margin + headerHeight + row * cellHeight + 36
        );
      }
    }

    this.ctx.textAlign = 'start';
  }

  private drawMusic(music: MusicConfig, margin?: MarginConfig) {
    const left = margin?.left ?? 60;
    const right = margin?.right ?? 40;
    const top = margin?.top ?? 100;

    this.ctx.strokeStyle = music.color;
    this.ctx.lineWidth = 1;
    this.ctx.setLineDash([]);

    const staffLineSpacing = 16; // Standard music staff line spacing
    const staffHeight = 4 * staffLineSpacing; // 5 lines = 4 spaces

    for (let staff = 0; staff < music.staffCount; staff++) {
      const staffY = top + staff * music.staffSpacing;

      // Draw 5 lines per staff
      for (let line = 0; line < 5; line++) {
        const y = staffY + line * staffLineSpacing;
        this.ctx.beginPath();
        this.ctx.moveTo(left, y);
        this.ctx.lineTo(this.width - right, y);
        this.ctx.stroke();
      }

      // Clef placeholder
      if (music.showClef) {
        this.ctx.font = 'bold 48px serif';
        this.ctx.fillStyle = music.color;
        // Treble clef symbol (simplified)
        this.ctx.fillText('𝄞', left + 5, staffY + staffHeight - 10);
      }

      // Barlines at end
      this.ctx.lineWidth = 2;
      this.ctx.beginPath();
      this.ctx.moveTo(this.width - right, staffY);
      this.ctx.lineTo(this.width - right, staffY + staffHeight);
      this.ctx.stroke();
      this.ctx.lineWidth = 1;
    }
  }

  private drawCustomPattern(custom: CustomPatternConfig) {
    switch (custom.svg) {
      case 'isometric':
        this.drawIsometric(custom.scale, custom.color);
        break;
      case 'hexagon':
        this.drawHexagonGrid(custom.scale, custom.color);
        break;
      case 'storyboard-2':
        this.drawStoryboard(2, custom.color);
        break;
      case 'storyboard-4':
        this.drawStoryboard(4, custom.color);
        break;
      case 'checklist':
        this.drawChecklist(custom.color);
        break;
      case 'dayplanner':
        this.drawDayPlanner(custom.color);
        break;
      case 'calligraphy':
        this.drawCalligraphyGuide(custom.color);
        break;
    }
  }

  private drawIsometric(scale: number, color: string) {
    this.ctx.strokeStyle = color;
    this.ctx.lineWidth = 0.5;

    // Vertical lines
    for (let x = 0; x < this.width + scale; x += scale) {
      this.ctx.beginPath();
      this.ctx.moveTo(x, 0);
      this.ctx.lineTo(x, this.height);
      this.ctx.stroke();
    }

    // Diagonal lines (60 degrees)
    for (let start = -this.height; start < this.width + this.height; start += scale) {
      // Right-leaning
      this.ctx.beginPath();
      this.ctx.moveTo(start, 0);
      this.ctx.lineTo(start + this.height / Math.tan(Math.PI / 3), this.height);
      this.ctx.stroke();

      // Left-leaning
      this.ctx.beginPath();
      this.ctx.moveTo(start, 0);
      this.ctx.lineTo(start - this.height / Math.tan(Math.PI / 3), this.height);
      this.ctx.stroke();
    }
  }

  private drawHexagonGrid(size: number, color: string) {
    this.ctx.strokeStyle = color;
    this.ctx.lineWidth = 1;

    const h = size * Math.sqrt(3);
    const w = size * 2;

    for (let row = 0; row < this.height / h + 1; row++) {
      for (let col = 0; col < this.width / (w * 0.75) + 1; col++) {
        const x = col * w * 0.75;
        const y = row * h + (col % 2 === 1 ? h / 2 : 0);

        this.drawHexagon(x, y, size);
      }
    }
  }

  private drawHexagon(cx: number, cy: number, size: number) {
    this.ctx.beginPath();
    for (let i = 0; i < 6; i++) {
      const angle = (Math.PI / 3) * i - Math.PI / 6;
      const x = cx + size * Math.cos(angle);
      const y = cy + size * Math.sin(angle);
      if (i === 0) {
        this.ctx.moveTo(x, y);
      } else {
        this.ctx.lineTo(x, y);
      }
    }
    this.ctx.closePath();
    this.ctx.stroke();
  }

  private drawStoryboard(panels: number, color: string) {
    const margin = 60;
    const gutter = 40;
    const labelHeight = 60;

    this.ctx.strokeStyle = color;
    this.ctx.lineWidth = 2;

    if (panels === 2) {
      // 2 panels side by side (landscape)
      const panelWidth = (this.width - margin * 2 - gutter) / 2;
      const panelHeight = this.height - margin * 2 - labelHeight;

      for (let i = 0; i < 2; i++) {
        const x = margin + i * (panelWidth + gutter);
        const y = margin;
        this.ctx.strokeRect(x, y, panelWidth, panelHeight);

        // Label line
        this.ctx.beginPath();
        this.ctx.moveTo(x, y + panelHeight + 10);
        this.ctx.lineTo(x + panelWidth, y + panelHeight + 10);
        this.ctx.stroke();
      }
    } else {
      // 4 panels in 2x2 grid (portrait)
      const panelWidth = (this.width - margin * 2 - gutter) / 2;
      const panelHeight = (this.height - margin * 2 - gutter - labelHeight * 2) / 2;

      for (let row = 0; row < 2; row++) {
        for (let col = 0; col < 2; col++) {
          const x = margin + col * (panelWidth + gutter);
          const y = margin + row * (panelHeight + gutter + labelHeight);
          this.ctx.strokeRect(x, y, panelWidth, panelHeight);

          // Label line
          this.ctx.beginPath();
          this.ctx.moveTo(x, y + panelHeight + 10);
          this.ctx.lineTo(x + panelWidth, y + panelHeight + 10);
          this.ctx.stroke();
        }
      }
    }
  }

  private drawChecklist(color: string) {
    const margin = 80;
    const rowHeight = 55;
    const checkboxSize = 30;
    const rows = Math.floor((this.height - margin * 2) / rowHeight);

    this.ctx.strokeStyle = color;
    this.ctx.lineWidth = 1;

    for (let i = 0; i < rows; i++) {
      const y = margin + i * rowHeight;

      // Checkbox
      this.ctx.strokeRect(margin, y, checkboxSize, checkboxSize);

      // Line for text
      this.ctx.beginPath();
      this.ctx.moveTo(margin + checkboxSize + 20, y + checkboxSize);
      this.ctx.lineTo(this.width - margin, y + checkboxSize);
      this.ctx.stroke();
    }
  }

  private drawDayPlanner(color: string) {
    const margin = 60;
    const headerHeight = 120;
    const timeColumnWidth = 100;
    const hourHeight = 80;

    this.ctx.strokeStyle = color;
    this.ctx.lineWidth = 1;
    this.ctx.fillStyle = color;
    this.ctx.font = 'bold 32px system-ui, sans-serif';

    // Date header
    this.ctx.strokeRect(margin, margin, this.width - margin * 2, headerHeight);
    this.ctx.fillText('DATE:', margin + 20, margin + 50);

    // Time slots
    const startHour = 6;
    const endHour = 22;

    for (let hour = startHour; hour <= endHour; hour++) {
      const y = margin + headerHeight + (hour - startHour) * hourHeight;

      // Hour label
      this.ctx.font = '24px system-ui, sans-serif';
      const hourLabel = hour > 12 ? `${hour - 12} PM` : hour === 12 ? '12 PM' : `${hour} AM`;
      this.ctx.fillText(hourLabel, margin + 10, y + 30);

      // Horizontal line
      this.ctx.beginPath();
      this.ctx.moveTo(margin + timeColumnWidth, y);
      this.ctx.lineTo(this.width - margin, y);
      this.ctx.stroke();
    }

    // Vertical divider
    this.ctx.beginPath();
    this.ctx.moveTo(margin + timeColumnWidth, margin + headerHeight);
    this.ctx.lineTo(margin + timeColumnWidth, margin + headerHeight + (endHour - startHour) * hourHeight);
    this.ctx.stroke();
  }

  private drawCalligraphyGuide(color: string) {
    const margin = 60;
    const setHeight = 100;
    const sets = Math.floor((this.height - margin * 2) / setHeight);

    this.ctx.strokeStyle = color;
    this.ctx.lineWidth = 1;

    for (let set = 0; set < sets; set++) {
      const baseY = margin + set * setHeight;

      // Ascender line (dashed)
      this.ctx.setLineDash([4, 4]);
      this.ctx.beginPath();
      this.ctx.moveTo(margin, baseY);
      this.ctx.lineTo(this.width - margin, baseY);
      this.ctx.stroke();

      // Cap height line
      this.ctx.setLineDash([]);
      this.ctx.beginPath();
      this.ctx.moveTo(margin, baseY + 25);
      this.ctx.lineTo(this.width - margin, baseY + 25);
      this.ctx.stroke();

      // X-height line
      this.ctx.beginPath();
      this.ctx.moveTo(margin, baseY + 50);
      this.ctx.lineTo(this.width - margin, baseY + 50);
      this.ctx.stroke();

      // Baseline (thicker)
      this.ctx.lineWidth = 2;
      this.ctx.beginPath();
      this.ctx.moveTo(margin, baseY + 70);
      this.ctx.lineTo(this.width - margin, baseY + 70);
      this.ctx.stroke();
      this.ctx.lineWidth = 1;

      // Descender line (dashed)
      this.ctx.setLineDash([4, 4]);
      this.ctx.beginPath();
      this.ctx.moveTo(margin, baseY + 95);
      this.ctx.lineTo(this.width - margin, baseY + 95);
      this.ctx.stroke();
      this.ctx.setLineDash([]);
    }
  }
}

export function generateSVG(config: TemplateConfig): string {
  const dims = getCanvasDimensions(config.orientation);
  const { width, height } = dims;

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}">\n`;
  svg += `  <rect width="${width}" height="${height}" fill="${config.backgroundColor}"/>\n`;

  // Generate pattern-specific SVG elements
  switch (config.type) {
    case 'lined':
      if (config.line) {
        const margin = config.margin;
        const startX = margin?.left ?? 0;
        const endX = width - (margin?.right ?? 0);
        const endY = height - (margin?.bottom ?? 0);

        svg += `  <g stroke="${config.line.color}" stroke-width="${config.line.thickness}">\n`;

        if (config.line.style === 'dashed') {
          svg += `    <g stroke-dasharray="8,4">\n`;
        } else if (config.line.style === 'dotted') {
          svg += `    <g stroke-dasharray="2,4">\n`;
        }

        for (let y = config.line.startY; y <= endY; y += config.line.spacing) {
          svg += `      <line x1="${startX}" y1="${y}" x2="${endX}" y2="${y}"/>\n`;
        }

        if (config.line.style !== 'solid') {
          svg += `    </g>\n`;
        }
        svg += `  </g>\n`;

        // Margin line
        if (margin?.showLines && margin.left > 0) {
          svg += `  <line x1="${margin.left}" y1="0" x2="${margin.left}" y2="${height}" stroke="${margin.color}" stroke-width="1"/>\n`;
        }
      }
      break;

    case 'grid':
      if (config.grid) {
        const margin = config.margin;
        const startX = margin?.left ?? 0;
        const startY = margin?.top ?? 0;
        const endX = width - (margin?.right ?? 0);
        const endY = height - (margin?.bottom ?? 0);

        svg += `  <g stroke="${config.grid.color}" stroke-width="${config.grid.thickness}">\n`;
        for (let x = startX; x <= endX; x += config.grid.cellSize) {
          svg += `    <line x1="${x}" y1="${startY}" x2="${x}" y2="${endY}"/>\n`;
        }
        for (let y = startY; y <= endY; y += config.grid.cellSize) {
          svg += `    <line x1="${startX}" y1="${y}" x2="${endX}" y2="${y}"/>\n`;
        }
        svg += `  </g>\n`;
      }
      break;

    case 'dot-grid':
      if (config.dotGrid) {
        const margin = config.margin;
        const startX = (margin?.left ?? 0) + config.dotGrid.spacing / 2;
        const startY = (margin?.top ?? 0) + config.dotGrid.spacing / 2;
        const endX = width - (margin?.right ?? 0);
        const endY = height - (margin?.bottom ?? 0);

        svg += `  <g fill="${config.dotGrid.color}">\n`;
        for (let y = startY; y < endY; y += config.dotGrid.spacing) {
          for (let x = startX; x < endX; x += config.dotGrid.spacing) {
            svg += `    <circle cx="${x}" cy="${y}" r="${config.dotGrid.dotSize}"/>\n`;
          }
        }
        svg += `  </g>\n`;
      }
      break;
  }

  svg += '</svg>';
  return svg;
}
