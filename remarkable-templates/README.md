# reMarkable Template Builder

A web-based template builder for creating custom templates for reMarkable devices.

## Features

- **Template Types**: Blank, Lined, Grid, Dot Grid, Cornell Notes, Calendar, Music Staff
- **Customizable Settings**:
  - Line spacing, thickness, color, and style
  - Grid cell size and minor divisions
  - Margin controls
  - Background color
- **Live Preview**: See your template at device resolution (1404×1872)
- **Export Options**:
  - PNG (required for reMarkable)
  - SVG (scalable vector format)
  - SSH install script for direct device deployment
- **Template Gallery**: 25+ built-in presets across categories

## Usage

### Development

```bash
npm install
npm run dev
```

### Build

```bash
npm run build
```

### Export to Device

1. Create or customize your template
2. Click "Export" and enable "Install Script"
3. Download all files
4. Run the install script:

```bash
chmod +x install_your_template.sh
./install_your_template.sh 10.11.99.1
```

## Device Specifications

- **Display Resolution**: 1404 × 1872 pixels
- **PPI**: 226 (reMarkable 2)
- **File Formats**: PNG (required) + SVG (optional)
- **Template Location**: `/usr/share/remarkable/templates/`

## Template Types

| Type | Description |
|------|-------------|
| Blank | Empty template |
| Lined | Horizontal ruled lines with configurable spacing |
| Grid | Square grid with optional minor divisions |
| Dot Grid | Dot pattern with configurable spacing |
| Cornell | Cornell note-taking layout with cue/notes/summary sections |
| Calendar | Monthly calendar or blank grid |
| Music | 5-line music staves with optional clefs |

## Categories

- **Creative**: Blank, Isometric, Hexagon, Music, Storyboard
- **Grids**: Grid, Dot Grid, Engineering Grid
- **Lines**: College, Legal, Small/Medium/Large ruled
- **Life/organize**: Cornell, Calendar, Checklist, Day Planner

## License

MIT
