# reMarkable Desktop Sync

A desktop application for syncing reMarkable documents with the cloud or a local server.

Built with Tauri + React + TypeScript.

## Features

- **System Tray**: Runs in background with system tray icon
- **Background Sync**: Automatic periodic synchronization
- **Sync Status Notifications**: Visual feedback for sync operations
- **Folder Selection**: Sync specific folders only
- **Offline Mode**: Continue working without network connectivity
- **Conflict Resolution UI**: Visual interface for resolving sync conflicts
- **Configurable Settings**: Server URL, sync interval, and more

## Requirements

### Development

- Node.js 18+
- Rust 1.77+
- System dependencies for Tauri:

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Linux (Fedora):**
```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file libxdo-devel
sudo dnf group install "C Development Tools and Libraries"
```

**Linux (Arch):**
```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module libappindicator-gtk3 librsvg
```

## Installation

```bash
# Clone the repository
git clone https://github.com/coleleavitt/remarkable-desktop.git
cd remarkable-desktop

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Configuration

Settings can be configured in the Settings panel:

- **Server URL**: Connect to reMarkable cloud (`https://tectonic.remarkable.com`) or a local sync server
- **Device Token**: Optional authentication token from your device
- **Sync Interval**: Time between automatic syncs (default: 5 minutes)
- **Local Path**: Where to store synced documents locally
- **Conflict Resolution**: Choose how to handle sync conflicts
- **Folder Selection**: Select specific folders to sync

## Usage

### System Tray

The application runs in the system tray. Click the tray icon to:
- Show/hide the main window
- Trigger manual sync
- Toggle offline mode
- Quit the application

### Sync Status

The status bar shows:
- Current sync state (syncing, connected, offline)
- Number of documents synced
- Number of conflicts (if any)

### Conflict Resolution

When documents are modified both locally and remotely, you can:
- **Keep Local**: Upload your local version
- **Keep Remote**: Download the server version
- **Keep Both**: Save both versions (local backup + download remote)

## Building for Distribution

### Linux (AppImage)

```bash
npm run tauri:build
# Output: src-tauri/target/release/bundle/appimage/
```

### Linux (Debian/Ubuntu .deb)

```bash
npm run tauri:build
# Output: src-tauri/target/release/bundle/deb/
```

## Cross-Compilation

### macOS

1. Install Rust target:
   ```bash
   rustup target add x86_64-apple-darwin
   rustup target add aarch64-apple-darwin
   ```

2. Set up cross-compilation (requires macOS SDK):
   ```bash
   # Using osxcross or similar
   export CC_x86_64_apple_darwin=x86_64-apple-darwin-clang
   export CXX_x86_64_apple_darwin=x86_64-apple-darwin-clang++
   ```

3. Build:
   ```bash
   npm run tauri:build -- --target x86_64-apple-darwin
   npm run tauri:build -- --target aarch64-apple-darwin
   ```

### Windows

1. Install Rust target:
   ```bash
   rustup target add x86_64-pc-windows-msvc
   ```

2. Set up cross-compilation (requires Wine + MSVC):
   ```bash
   # Using cargo-xwin
   cargo install cargo-xwin
   ```

3. Build:
   ```bash
   cargo xwin build --release --target x86_64-pc-windows-msvc
   # Then run tauri bundler manually
   ```

**Note**: Cross-compilation for Windows/macOS from Linux is complex. Building natively on each platform is recommended.

## Project Structure

```
remarkable-desktop/
├── src/                    # React frontend
│   ├── api/               # Tauri API layer
│   ├── components/        # React components
│   ├── hooks/             # Custom React hooks
│   ├── types/             # TypeScript types
│   ├── App.tsx            # Main app component
│   └── main.tsx           # Entry point
├── src-tauri/             # Tauri backend (Rust)
│   ├── src/
│   │   ├── lib.rs         # Main library entry
│   │   ├── commands.rs    # Tauri command handlers
│   │   ├── sync.rs        # Sync logic
│   │   ├── state.rs       # App state management
│   │   ├── tray.rs        # System tray
│   │   ├── config.rs      # Configuration types
│   │   └── error.rs       # Error handling
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri config
└── package.json           # Node dependencies
```

## Connecting to a Local Server

To sync with a local server instead of reMarkable cloud:

1. Start your local sync server (e.g., remarkable-server)
2. Open Settings in the app
3. Change Server URL to your local server (e.g., `http://localhost:8080`)
4. Test connection
5. Save settings

## License

MIT
