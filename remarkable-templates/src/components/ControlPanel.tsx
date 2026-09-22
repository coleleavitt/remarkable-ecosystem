import type { TemplateConfig, TemplateType } from '../types';

interface ControlPanelProps {
  config: TemplateConfig;
  onChange: (config: TemplateConfig) => void;
}

function Slider({
  label,
  value,
  min,
  max,
  step = 1,
  onChange,
  unit = 'px',
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (value: number) => void;
  unit?: string;
}) {
  return (
    <div className="space-y-1">
      <div className="flex justify-between text-sm">
        <span className="text-neutral-300">{label}</span>
        <span className="text-neutral-400">{value}{unit}</span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full h-2 bg-neutral-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
      />
    </div>
  );
}

function ColorInput({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm text-neutral-300">{label}</span>
      <div className="flex items-center gap-2">
        <input
          type="color"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="w-8 h-8 rounded border border-neutral-600 bg-transparent cursor-pointer"
        />
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="w-20 px-2 py-1 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-200"
        />
      </div>
    </div>
  );
}

function Select({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm text-neutral-300">{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="px-3 py-1.5 text-sm bg-neutral-800 border border-neutral-600 rounded text-neutral-200 cursor-pointer"
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  );
}

function Checkbox({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex items-center gap-2 cursor-pointer">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="w-4 h-4 rounded border-neutral-600 bg-neutral-800 text-blue-500 focus:ring-blue-500 focus:ring-offset-0"
      />
      <span className="text-sm text-neutral-300">{label}</span>
    </label>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <h3 className="text-sm font-semibold text-neutral-200 uppercase tracking-wide">{title}</h3>
      <div className="space-y-3">{children}</div>
    </div>
  );
}

export function ControlPanel({ config, onChange }: ControlPanelProps) {
  const update = (partial: Partial<TemplateConfig>) => {
    onChange({ ...config, ...partial });
  };

  const updateLine = (partial: Partial<NonNullable<TemplateConfig['line']>>) => {
    update({ line: { ...config.line!, ...partial } });
  };

  const updateGrid = (partial: Partial<NonNullable<TemplateConfig['grid']>>) => {
    update({ grid: { ...config.grid!, ...partial } });
  };

  const updateDotGrid = (partial: Partial<NonNullable<TemplateConfig['dotGrid']>>) => {
    update({ dotGrid: { ...config.dotGrid!, ...partial } });
  };

  const updateMargin = (partial: Partial<NonNullable<TemplateConfig['margin']>>) => {
    update({ margin: { ...config.margin!, ...partial } });
  };

  const updateCornell = (partial: Partial<NonNullable<TemplateConfig['cornell']>>) => {
    update({ cornell: { ...config.cornell!, ...partial } });
  };

  const updateCalendar = (partial: Partial<NonNullable<TemplateConfig['calendar']>>) => {
    update({ calendar: { ...config.calendar!, ...partial } });
  };

  const updateMusic = (partial: Partial<NonNullable<TemplateConfig['music']>>) => {
    update({ music: { ...config.music!, ...partial } });
  };

  const templateTypes: { value: TemplateType; label: string }[] = [
    { value: 'blank', label: 'Blank' },
    { value: 'lined', label: 'Lined' },
    { value: 'grid', label: 'Grid' },
    { value: 'dot-grid', label: 'Dot Grid' },
    { value: 'cornell', label: 'Cornell Notes' },
    { value: 'calendar', label: 'Calendar' },
    { value: 'music', label: 'Music Staff' },
  ];

  return (
    <div className="space-y-6 p-4 overflow-y-auto h-full">
      {/* Basic Settings */}
      <Section title="Template Type">
        <Select
          label="Type"
          value={config.type}
          options={templateTypes}
          onChange={(v) => update({ type: v as TemplateType })}
        />
        <Select
          label="Orientation"
          value={config.orientation}
          options={[
            { value: 'portrait', label: 'Portrait' },
            { value: 'landscape', label: 'Landscape' },
          ]}
          onChange={(v) => update({ orientation: v as 'portrait' | 'landscape' })}
        />
        <ColorInput
          label="Background"
          value={config.backgroundColor}
          onChange={(v) => update({ backgroundColor: v })}
        />
      </Section>

      {/* Line-specific controls */}
      {config.type === 'lined' && config.line && (
        <Section title="Line Settings">
          <Slider
            label="Line Spacing"
            value={config.line.spacing}
            min={16}
            max={80}
            onChange={(v) => updateLine({ spacing: v })}
          />
          <Slider
            label="Start Position"
            value={config.line.startY}
            min={40}
            max={200}
            onChange={(v) => updateLine({ startY: v })}
          />
          <Slider
            label="Line Thickness"
            value={config.line.thickness}
            min={0.5}
            max={3}
            step={0.5}
            onChange={(v) => updateLine({ thickness: v })}
          />
          <ColorInput
            label="Line Color"
            value={config.line.color}
            onChange={(v) => updateLine({ color: v })}
          />
          <Select
            label="Line Style"
            value={config.line.style}
            options={[
              { value: 'solid', label: 'Solid' },
              { value: 'dashed', label: 'Dashed' },
              { value: 'dotted', label: 'Dotted' },
            ]}
            onChange={(v) => updateLine({ style: v as 'solid' | 'dashed' | 'dotted' })}
          />
        </Section>
      )}

      {/* Grid-specific controls */}
      {config.type === 'grid' && config.grid && (
        <Section title="Grid Settings">
          <Slider
            label="Cell Size"
            value={config.grid.cellSize}
            min={10}
            max={80}
            onChange={(v) => updateGrid({ cellSize: v })}
          />
          <Slider
            label="Line Thickness"
            value={config.grid.thickness}
            min={0.5}
            max={3}
            step={0.5}
            onChange={(v) => updateGrid({ thickness: v })}
          />
          <ColorInput
            label="Line Color"
            value={config.grid.color}
            onChange={(v) => updateGrid({ color: v })}
          />
          <Select
            label="Line Style"
            value={config.grid.style}
            options={[
              { value: 'solid', label: 'Solid' },
              { value: 'dashed', label: 'Dashed' },
            ]}
            onChange={(v) => updateGrid({ style: v as 'solid' | 'dashed' })}
          />
          <Checkbox
            label="Show Minor Lines"
            checked={config.grid.showMinorLines}
            onChange={(v) => updateGrid({ showMinorLines: v })}
          />
          {config.grid.showMinorLines && (
            <>
              <Slider
                label="Minor Divisions"
                value={config.grid.minorDivisions}
                min={2}
                max={10}
                onChange={(v) => updateGrid({ minorDivisions: v })}
              />
              <ColorInput
                label="Minor Color"
                value={config.grid.minorColor}
                onChange={(v) => updateGrid({ minorColor: v })}
              />
            </>
          )}
        </Section>
      )}

      {/* Dot Grid controls */}
      {config.type === 'dot-grid' && config.dotGrid && (
        <Section title="Dot Grid Settings">
          <Slider
            label="Dot Spacing"
            value={config.dotGrid.spacing}
            min={10}
            max={60}
            onChange={(v) => updateDotGrid({ spacing: v })}
          />
          <Slider
            label="Dot Size"
            value={config.dotGrid.dotSize}
            min={1}
            max={5}
            step={0.5}
            onChange={(v) => updateDotGrid({ dotSize: v })}
          />
          <ColorInput
            label="Dot Color"
            value={config.dotGrid.color}
            onChange={(v) => updateDotGrid({ color: v })}
          />
        </Section>
      )}

      {/* Cornell Notes controls */}
      {config.type === 'cornell' && config.cornell && (
        <Section title="Cornell Notes Settings">
          <Slider
            label="Cue Column Width"
            value={config.cornell.cueWidth}
            min={200}
            max={500}
            onChange={(v) => updateCornell({ cueWidth: v })}
          />
          <Slider
            label="Summary Height"
            value={config.cornell.summaryHeight}
            min={150}
            max={400}
            onChange={(v) => updateCornell({ summaryHeight: v })}
          />
          <Slider
            label="Line Spacing"
            value={config.cornell.lineSpacing}
            min={24}
            max={60}
            onChange={(v) => updateCornell({ lineSpacing: v })}
          />
          <ColorInput
            label="Line Color"
            value={config.cornell.color}
            onChange={(v) => updateCornell({ color: v })}
          />
        </Section>
      )}

      {/* Calendar controls */}
      {config.type === 'calendar' && config.calendar && (
        <Section title="Calendar Settings">
          <Select
            label="Month"
            value={String(config.calendar.month)}
            options={[
              { value: '0', label: 'Blank Grid' },
              { value: '1', label: 'January' },
              { value: '2', label: 'February' },
              { value: '3', label: 'March' },
              { value: '4', label: 'April' },
              { value: '5', label: 'May' },
              { value: '6', label: 'June' },
              { value: '7', label: 'July' },
              { value: '8', label: 'August' },
              { value: '9', label: 'September' },
              { value: '10', label: 'October' },
              { value: '11', label: 'November' },
              { value: '12', label: 'December' },
            ]}
            onChange={(v) => updateCalendar({ month: Number(v) })}
          />
          {config.calendar.month > 0 && (
            <Slider
              label="Year"
              value={config.calendar.year}
              min={2020}
              max={2030}
              unit=""
              onChange={(v) => updateCalendar({ year: v })}
            />
          )}
          <Checkbox
            label="Show Week Numbers"
            checked={config.calendar.showWeekNumbers}
            onChange={(v) => updateCalendar({ showWeekNumbers: v })}
          />
          <Checkbox
            label="Start on Sunday"
            checked={config.calendar.startOnSunday}
            onChange={(v) => updateCalendar({ startOnSunday: v })}
          />
          <ColorInput
            label="Color"
            value={config.calendar.color}
            onChange={(v) => updateCalendar({ color: v })}
          />
        </Section>
      )}

      {/* Music Staff controls */}
      {config.type === 'music' && config.music && (
        <Section title="Music Staff Settings">
          <Slider
            label="Staff Count"
            value={config.music.staffCount}
            min={1}
            max={12}
            unit=""
            onChange={(v) => updateMusic({ staffCount: v })}
          />
          <Slider
            label="Staff Spacing"
            value={config.music.staffSpacing}
            min={100}
            max={300}
            onChange={(v) => updateMusic({ staffSpacing: v })}
          />
          <ColorInput
            label="Line Color"
            value={config.music.color}
            onChange={(v) => updateMusic({ color: v })}
          />
          <Checkbox
            label="Show Clef"
            checked={config.music.showClef}
            onChange={(v) => updateMusic({ showClef: v })}
          />
          <Checkbox
            label="Show Time Signature"
            checked={config.music.showTimeSignature}
            onChange={(v) => updateMusic({ showTimeSignature: v })}
          />
        </Section>
      )}

      {/* Margin controls (for lined, grid, dot-grid) */}
      {['lined', 'grid', 'dot-grid'].includes(config.type) && config.margin && (
        <Section title="Margins">
          <Slider
            label="Left Margin"
            value={config.margin.left}
            min={0}
            max={200}
            onChange={(v) => updateMargin({ left: v })}
          />
          <Slider
            label="Right Margin"
            value={config.margin.right}
            min={0}
            max={200}
            onChange={(v) => updateMargin({ right: v })}
          />
          <Slider
            label="Top Margin"
            value={config.margin.top}
            min={0}
            max={200}
            onChange={(v) => updateMargin({ top: v })}
          />
          <Slider
            label="Bottom Margin"
            value={config.margin.bottom}
            min={0}
            max={200}
            onChange={(v) => updateMargin({ bottom: v })}
          />
          <Checkbox
            label="Show Margin Lines"
            checked={config.margin.showLines}
            onChange={(v) => updateMargin({ showLines: v })}
          />
          {config.margin.showLines && (
            <ColorInput
              label="Margin Color"
              value={config.margin.color}
              onChange={(v) => updateMargin({ color: v })}
            />
          )}
        </Section>
      )}
    </div>
  );
}
