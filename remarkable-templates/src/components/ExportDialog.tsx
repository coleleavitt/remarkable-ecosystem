import { useState } from 'react';
import type { TemplateConfig } from '../types';
import { exportToPNG, exportToSVG, downloadBlob, downloadText, generateInstallScript } from '../lib/export';

interface ExportDialogProps {
  config: TemplateConfig;
  isOpen: boolean;
  onClose: () => void;
}

export function ExportDialog({ config, isOpen, onClose }: ExportDialogProps) {
  const [filename, setFilename] = useState('custom_template');
  const [exportPNG, setExportPNG] = useState(true);
  const [exportSVG, setExportSVG] = useState(true);
  const [exportScript, setExportScript] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [exportStatus, setExportStatus] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleExport = async () => {
    setIsExporting(true);
    setExportStatus('Generating files...');

    try {
      if (exportPNG) {
        const png = await exportToPNG(config, filename);
        downloadBlob(png, `${filename}.png`);
      }

      if (exportSVG) {
        const svg = exportToSVG(config);
        downloadText(svg, `${filename}.svg`, 'image/svg+xml');
      }

      if (exportScript) {
        const script = generateInstallScript(filename, filename, ['Creative']);
        downloadText(script, `install_${filename}.sh`, 'text/x-shellscript');
      }

      setExportStatus('Export complete!');
      setTimeout(() => {
        setExportStatus(null);
        onClose();
      }, 1500);
    } catch (error) {
      setExportStatus(`Error: ${error}`);
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-neutral-900 border border-neutral-700 rounded-xl shadow-2xl w-full max-w-md mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-neutral-700">
          <h2 className="text-lg font-semibold text-neutral-100">Export Template</h2>
          <button
            onClick={onClose}
            className="p-1 text-neutral-400 hover:text-neutral-200 transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4">
          {/* Filename */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-neutral-300">
              Filename
            </label>
            <input
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value.replace(/[^a-zA-Z0-9_-]/g, '_'))}
              className="w-full px-3 py-2 bg-neutral-800 border border-neutral-600 rounded-lg text-neutral-200 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="template_name"
            />
            <p className="text-xs text-neutral-500">
              Use letters, numbers, underscores, and hyphens only
            </p>
          </div>

          {/* Export options */}
          <div className="space-y-3">
            <label className="block text-sm font-medium text-neutral-300">
              Export Formats
            </label>

            <label className="flex items-center gap-3 p-3 bg-neutral-800 rounded-lg cursor-pointer hover:bg-neutral-750 transition-colors">
              <input
                type="checkbox"
                checked={exportPNG}
                onChange={(e) => setExportPNG(e.target.checked)}
                className="w-4 h-4 rounded border-neutral-600 bg-neutral-700 text-blue-500 focus:ring-blue-500"
              />
              <div>
                <div className="text-sm font-medium text-neutral-200">PNG Image</div>
                <div className="text-xs text-neutral-400">
                  1404×1872 pixels (required for reMarkable)
                </div>
              </div>
            </label>

            <label className="flex items-center gap-3 p-3 bg-neutral-800 rounded-lg cursor-pointer hover:bg-neutral-750 transition-colors">
              <input
                type="checkbox"
                checked={exportSVG}
                onChange={(e) => setExportSVG(e.target.checked)}
                className="w-4 h-4 rounded border-neutral-600 bg-neutral-700 text-blue-500 focus:ring-blue-500"
              />
              <div>
                <div className="text-sm font-medium text-neutral-200">SVG Vector</div>
                <div className="text-xs text-neutral-400">
                  Scalable for printing and editing
                </div>
              </div>
            </label>

            <label className="flex items-center gap-3 p-3 bg-neutral-800 rounded-lg cursor-pointer hover:bg-neutral-750 transition-colors">
              <input
                type="checkbox"
                checked={exportScript}
                onChange={(e) => setExportScript(e.target.checked)}
                className="w-4 h-4 rounded border-neutral-600 bg-neutral-700 text-blue-500 focus:ring-blue-500"
              />
              <div>
                <div className="text-sm font-medium text-neutral-200">Install Script</div>
                <div className="text-xs text-neutral-400">
                  SSH script to install on reMarkable device
                </div>
              </div>
            </label>
          </div>

          {/* SSH Info */}
          {exportScript && (
            <div className="p-3 bg-neutral-800 rounded-lg text-xs text-neutral-400 space-y-1">
              <p className="font-medium text-neutral-300">SSH Installation</p>
              <p>Run the script with your device IP:</p>
              <code className="block p-2 bg-neutral-900 rounded mt-1 text-neutral-300">
                ./install_{filename}.sh 10.11.99.1
              </code>
            </div>
          )}

          {/* Status */}
          {exportStatus && (
            <div className={`p-3 rounded-lg text-sm ${
              exportStatus.startsWith('Error')
                ? 'bg-red-900/50 text-red-200'
                : 'bg-blue-900/50 text-blue-200'
            }`}>
              {exportStatus}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 p-4 border-t border-neutral-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-neutral-300 hover:text-neutral-100 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleExport}
            disabled={isExporting || (!exportPNG && !exportSVG && !exportScript)}
            className="px-4 py-2 text-sm font-medium bg-blue-600 hover:bg-blue-500 disabled:bg-neutral-700 disabled:text-neutral-500 text-white rounded-lg transition-colors"
          >
            {isExporting ? 'Exporting...' : 'Export'}
          </button>
        </div>
      </div>
    </div>
  );
}
