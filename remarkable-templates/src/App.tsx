import { useState, useCallback } from 'react';
import type { TemplateConfig } from './types';
import { TemplatePreview } from './components/TemplatePreview';
import { ControlPanel } from './components/ControlPanel';
import { TemplateGallery } from './components/TemplateGallery';
import { ExportDialog } from './components/ExportDialog';
// Presets used in TemplateGallery

const DEFAULT_CONFIG: TemplateConfig = {
  type: 'lined',
  orientation: 'portrait',
  backgroundColor: '#ffffff',
  line: {
    spacing: 40,
    startY: 120,
    color: '#aaaaaa',
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
  },
};

type ViewMode = 'editor' | 'gallery';

function App() {
  const [config, setConfig] = useState<TemplateConfig>(DEFAULT_CONFIG);
  const [viewMode, setViewMode] = useState<ViewMode>('editor');
  const [showExport, setShowExport] = useState(false);
  const [previewScale, setPreviewScale] = useState(0.4);

  const handleConfigChange = useCallback((newConfig: TemplateConfig) => {
    // Ensure required sub-configs exist when type changes
    let configWithDefaults = { ...newConfig };

    if (newConfig.type === 'lined' && !newConfig.line) {
      configWithDefaults.line = { ...DEFAULT_CONFIG.line! };
      configWithDefaults.margin = { ...DEFAULT_CONFIG.margin! };
    } else if (newConfig.type === 'grid' && !newConfig.grid) {
      configWithDefaults.grid = {
        cellSize: 30,
        color: '#cccccc',
        thickness: 1,
        style: 'solid',
        showMinorLines: false,
        minorDivisions: 5,
        minorColor: '#eeeeee',
      };
      configWithDefaults.margin = { left: 0, right: 0, top: 0, bottom: 0, showLines: false, color: '#cccccc' };
    } else if (newConfig.type === 'dot-grid' && !newConfig.dotGrid) {
      configWithDefaults.dotGrid = {
        spacing: 30,
        dotSize: 2.5,
        color: '#999999',
      };
      configWithDefaults.margin = { left: 0, right: 0, top: 0, bottom: 0, showLines: false, color: '#cccccc' };
    } else if (newConfig.type === 'cornell' && !newConfig.cornell) {
      configWithDefaults.cornell = {
        cueWidth: 350,
        summaryHeight: 280,
        lineSpacing: 36,
        color: '#aaaaaa',
      };
      configWithDefaults.margin = { left: 40, right: 40, top: 100, bottom: 40, showLines: false, color: '#aaaaaa' };
    } else if (newConfig.type === 'calendar' && !newConfig.calendar) {
      configWithDefaults.calendar = {
        month: new Date().getMonth() + 1,
        year: new Date().getFullYear(),
        showWeekNumbers: false,
        startOnSunday: true,
        color: '#666666',
      };
    } else if (newConfig.type === 'music' && !newConfig.music) {
      configWithDefaults.music = {
        staffCount: 10,
        staffSpacing: 160,
        color: '#666666',
        showClef: true,
        showTimeSignature: false,
      };
      configWithDefaults.margin = { left: 60, right: 40, top: 100, bottom: 100, showLines: false, color: '#666666' };
    }

    setConfig(configWithDefaults);
  }, []);

  const handlePresetSelect = useCallback((presetConfig: TemplateConfig) => {
    setConfig({ ...presetConfig });
    setViewMode('editor');
  }, []);

  return (
    <div className="h-screen flex flex-col bg-neutral-950 text-neutral-100">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-neutral-800 bg-neutral-900">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold tracking-tight">
            <span className="text-neutral-400">re</span>
            <span className="text-white">Markable</span>
            <span className="text-neutral-500 font-normal ml-2">Template Builder</span>
          </h1>
        </div>

        <div className="flex items-center gap-3">
          {/* View toggle */}
          <div className="flex bg-neutral-800 rounded-lg p-1">
            <button
              onClick={() => setViewMode('editor')}
              className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
                viewMode === 'editor'
                  ? 'bg-neutral-700 text-white'
                  : 'text-neutral-400 hover:text-white'
              }`}
            >
              Editor
            </button>
            <button
              onClick={() => setViewMode('gallery')}
              className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
                viewMode === 'gallery'
                  ? 'bg-neutral-700 text-white'
                  : 'text-neutral-400 hover:text-white'
              }`}
            >
              Gallery
            </button>
          </div>

          {/* Export button */}
          <button
            onClick={() => setShowExport(true)}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg transition-colors"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Export
          </button>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 flex overflow-hidden">
        {viewMode === 'editor' ? (
          <>
            {/* Left sidebar - Controls */}
            <aside className="w-80 border-r border-neutral-800 bg-neutral-900 overflow-hidden flex flex-col">
              <div className="p-3 border-b border-neutral-800">
                <h2 className="text-sm font-semibold text-neutral-300 uppercase tracking-wide">
                  Template Settings
                </h2>
              </div>
              <div className="flex-1 overflow-y-auto">
                <ControlPanel config={config} onChange={handleConfigChange} />
              </div>
            </aside>

            {/* Center - Preview */}
            <div className="flex-1 flex flex-col items-center justify-center p-8 bg-neutral-950">
              {/* Scale controls */}
              <div className="flex items-center gap-4 mb-6">
                <span className="text-sm text-neutral-500">Zoom:</span>
                <div className="flex gap-2">
                  {[0.25, 0.4, 0.5, 0.75, 1].map((scale) => (
                    <button
                      key={scale}
                      onClick={() => setPreviewScale(scale)}
                      className={`px-2 py-1 text-xs rounded transition-colors ${
                        previewScale === scale
                          ? 'bg-neutral-700 text-white'
                          : 'text-neutral-500 hover:text-white hover:bg-neutral-800'
                      }`}
                    >
                      {Math.round(scale * 100)}%
                    </button>
                  ))}
                </div>
              </div>

              <TemplatePreview config={config} scale={previewScale} />
            </div>
          </>
        ) : (
          /* Gallery view */
          <div className="flex-1 overflow-hidden bg-neutral-900">
            <TemplateGallery
              onSelect={handlePresetSelect}
            />
          </div>
        )}
      </main>

      {/* Export dialog */}
      <ExportDialog
        config={config}
        isOpen={showExport}
        onClose={() => setShowExport(false)}
      />
    </div>
  );
}

export default App;
