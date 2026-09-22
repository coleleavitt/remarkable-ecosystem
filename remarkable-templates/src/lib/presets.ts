import type { TemplatePreset } from '../types';

const LIGHT_GRAY = '#cccccc';
const MEDIUM_GRAY = '#999999';
const DARK_GRAY = '#666666';
const LINE_GRAY = '#aaaaaa';

export const TEMPLATE_PRESETS: TemplatePreset[] = [
  // Blank templates
  {
    id: 'blank-portrait',
    name: 'Blank',
    category: 'Creative',
    config: {
      type: 'blank',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
    }
  },
  {
    id: 'blank-landscape',
    name: 'Blank (L)',
    category: 'Creative',
    config: {
      type: 'blank',
      orientation: 'landscape',
      backgroundColor: '#ffffff',
    }
  },

  // Lined templates
  {
    id: 'lined-medium',
    name: 'Lined Medium',
    category: 'Lines',
    config: {
      type: 'lined',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      line: {
        spacing: 40,
        startY: 120,
        color: LINE_GRAY,
        thickness: 1,
        style: 'solid',
      },
      margin: {
        left: 100,
        right: 50,
        top: 100,
        bottom: 100,
        showLines: true,
        color: '#ff6b6b',
      }
    }
  },
  {
    id: 'lined-small',
    name: 'Lined Small',
    category: 'Lines',
    config: {
      type: 'lined',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      line: {
        spacing: 28,
        startY: 100,
        color: LINE_GRAY,
        thickness: 1,
        style: 'solid',
      },
      margin: {
        left: 80,
        right: 40,
        top: 80,
        bottom: 80,
        showLines: true,
        color: '#ff6b6b',
      }
    }
  },
  {
    id: 'lined-large',
    name: 'Lined Large',
    category: 'Lines',
    config: {
      type: 'lined',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      line: {
        spacing: 56,
        startY: 140,
        color: LINE_GRAY,
        thickness: 1,
        style: 'solid',
      },
      margin: {
        left: 120,
        right: 60,
        top: 120,
        bottom: 120,
        showLines: true,
        color: '#ff6b6b',
      }
    }
  },
  {
    id: 'us-college',
    name: 'US College Ruled',
    category: 'Lines',
    config: {
      type: 'lined',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      line: {
        spacing: 32, // 7.1mm at 226 PPI ≈ 32px
        startY: 100,
        color: '#6699cc',
        thickness: 1,
        style: 'solid',
      },
      margin: {
        left: 90,
        right: 50,
        top: 100,
        bottom: 80,
        showLines: true,
        color: '#ff6666',
      }
    }
  },
  {
    id: 'us-legal',
    name: 'US Legal Ruled',
    category: 'Lines',
    config: {
      type: 'lined',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      line: {
        spacing: 40, // 8.7mm ≈ 40px at 226 PPI
        startY: 120,
        color: LINE_GRAY,
        thickness: 1,
        style: 'solid',
      },
      margin: {
        left: 90,
        right: 50,
        top: 100,
        bottom: 80,
        showLines: true,
        color: '#ff6666',
      }
    }
  },

  // Grid templates
  {
    id: 'grid-small',
    name: 'Grid Small',
    category: 'Grids',
    config: {
      type: 'grid',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      grid: {
        cellSize: 20,
        color: LIGHT_GRAY,
        thickness: 1,
        style: 'solid',
        showMinorLines: false,
        minorDivisions: 5,
        minorColor: '#eeeeee',
      }
    }
  },
  {
    id: 'grid-medium',
    name: 'Grid Medium',
    category: 'Grids',
    config: {
      type: 'grid',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      grid: {
        cellSize: 30,
        color: LIGHT_GRAY,
        thickness: 1,
        style: 'solid',
        showMinorLines: false,
        minorDivisions: 5,
        minorColor: '#eeeeee',
      }
    }
  },
  {
    id: 'grid-large',
    name: 'Grid Large',
    category: 'Grids',
    config: {
      type: 'grid',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      grid: {
        cellSize: 45,
        color: LIGHT_GRAY,
        thickness: 1,
        style: 'solid',
        showMinorLines: false,
        minorDivisions: 5,
        minorColor: '#eeeeee',
      }
    }
  },
  {
    id: 'grid-engineering',
    name: 'Engineering Grid',
    category: 'Grids',
    config: {
      type: 'grid',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      grid: {
        cellSize: 50,
        color: MEDIUM_GRAY,
        thickness: 1,
        style: 'solid',
        showMinorLines: true,
        minorDivisions: 5,
        minorColor: '#dddddd',
      }
    }
  },

  // Dot grid templates
  {
    id: 'dots-small',
    name: 'Dots Small',
    category: 'Grids',
    config: {
      type: 'dot-grid',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      dotGrid: {
        spacing: 20,
        dotSize: 2,
        color: MEDIUM_GRAY,
      }
    }
  },
  {
    id: 'dots-medium',
    name: 'Dots Medium',
    category: 'Grids',
    config: {
      type: 'dot-grid',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      dotGrid: {
        spacing: 30,
        dotSize: 2.5,
        color: MEDIUM_GRAY,
      }
    }
  },
  {
    id: 'dots-large',
    name: 'Dots Large',
    category: 'Grids',
    config: {
      type: 'dot-grid',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      dotGrid: {
        spacing: 45,
        dotSize: 3,
        color: MEDIUM_GRAY,
      }
    }
  },

  // Cornell notes
  {
    id: 'cornell',
    name: 'Cornell Notes',
    category: 'Life/organize',
    config: {
      type: 'cornell',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      cornell: {
        cueWidth: 350,
        summaryHeight: 280,
        lineSpacing: 36,
        color: LINE_GRAY,
      },
      margin: {
        left: 40,
        right: 40,
        top: 100,
        bottom: 40,
        showLines: false,
        color: LINE_GRAY,
      }
    }
  },

  // Calendar
  {
    id: 'calendar-blank',
    name: 'Calendar Grid',
    category: 'Life/organize',
    config: {
      type: 'calendar',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      calendar: {
        month: 0, // 0 = blank grid
        year: new Date().getFullYear(),
        showWeekNumbers: false,
        startOnSunday: true,
        color: DARK_GRAY,
      }
    }
  },
  {
    id: 'calendar-monthly',
    name: 'Monthly Calendar',
    category: 'Life/organize',
    config: {
      type: 'calendar',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      calendar: {
        month: new Date().getMonth() + 1,
        year: new Date().getFullYear(),
        showWeekNumbers: true,
        startOnSunday: true,
        color: DARK_GRAY,
      }
    }
  },
  {
    id: 'weekly-planner',
    name: 'Weekly Planner',
    category: 'Life/organize',
    config: {
      type: 'calendar',
      orientation: 'landscape',
      backgroundColor: '#ffffff',
      calendar: {
        month: 0,
        year: 0,
        showWeekNumbers: false,
        startOnSunday: true,
        color: DARK_GRAY,
      }
    }
  },

  // Music
  {
    id: 'music-standard',
    name: 'Music Staff',
    category: 'Creative',
    config: {
      type: 'music',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      music: {
        staffCount: 10,
        staffSpacing: 160,
        color: DARK_GRAY,
        showClef: true,
        showTimeSignature: false,
      }
    }
  },
  {
    id: 'music-piano',
    name: 'Piano Staff',
    category: 'Creative',
    config: {
      type: 'music',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      music: {
        staffCount: 6, // 6 grand staves
        staffSpacing: 280,
        color: DARK_GRAY,
        showClef: true,
        showTimeSignature: true,
      }
    }
  },
  {
    id: 'guitar-tab',
    name: 'Guitar Tablature',
    category: 'Creative',
    config: {
      type: 'music',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      music: {
        staffCount: 8,
        staffSpacing: 200,
        color: DARK_GRAY,
        showClef: false,
        showTimeSignature: false,
      }
    }
  },

  // Isometric
  {
    id: 'isometric',
    name: 'Isometric Grid',
    category: 'Creative',
    config: {
      type: 'custom',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      custom: {
        svg: 'isometric',
        scale: 40,
        color: LIGHT_GRAY,
      }
    }
  },

  // Hexagon
  {
    id: 'hexagon',
    name: 'Hexagon Grid',
    category: 'Grids',
    config: {
      type: 'custom',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      custom: {
        svg: 'hexagon',
        scale: 35,
        color: LIGHT_GRAY,
      }
    }
  },

  // Storyboard
  {
    id: 'storyboard-2',
    name: 'Storyboard (2)',
    category: 'Creative',
    config: {
      type: 'custom',
      orientation: 'landscape',
      backgroundColor: '#ffffff',
      custom: {
        svg: 'storyboard-2',
        scale: 1,
        color: DARK_GRAY,
      }
    }
  },
  {
    id: 'storyboard-4',
    name: 'Storyboard (4)',
    category: 'Creative',
    config: {
      type: 'custom',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      custom: {
        svg: 'storyboard-4',
        scale: 1,
        color: DARK_GRAY,
      }
    }
  },

  // Checklist
  {
    id: 'checklist',
    name: 'Checklist',
    category: 'Life/organize',
    config: {
      type: 'custom',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      custom: {
        svg: 'checklist',
        scale: 1,
        color: LINE_GRAY,
      }
    }
  },

  // Day planner
  {
    id: 'dayplanner',
    name: 'Day Planner',
    category: 'Life/organize',
    config: {
      type: 'custom',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      custom: {
        svg: 'dayplanner',
        scale: 1,
        color: LINE_GRAY,
      }
    }
  },

  // Calligraphy
  {
    id: 'calligraphy',
    name: 'Calligraphy Guide',
    category: 'Creative',
    config: {
      type: 'custom',
      orientation: 'portrait',
      backgroundColor: '#ffffff',
      custom: {
        svg: 'calligraphy',
        scale: 1,
        color: LINE_GRAY,
      }
    }
  },
];

export const CATEGORIES = ['Creative', 'Grids', 'Lines', 'Life/organize'];

export function getPresetsByCategory(category: string): TemplatePreset[] {
  return TEMPLATE_PRESETS.filter(p => p.category === category);
}

export function getPresetById(id: string): TemplatePreset | undefined {
  return TEMPLATE_PRESETS.find(p => p.id === id);
}
