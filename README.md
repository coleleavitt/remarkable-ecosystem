# reMarkable Ecosystem

A complete self-hosted alternative to reMarkable's cloud infrastructure, plus features they don't offer.

## 🎯 What's Included

| Tool | Description | Tech |
|------|-------------|------|
| **remarkable-portal** | Web UI for document browsing, preview, export | React 19, Vite |
| **remarkable-desktop** | Desktop sync app with system tray | Tauri, React |
| **remarkable-mobile** | iOS/Android sync app | React Native, Expo |
| **remarkable-screenshare** | Screen viewer (USB + WebRTC) | Rust |
| **remarkable-collab** | Real-time multi-user editing | Rust, WebSocket |
| **remarkable-diff** | Visual CRDT merge/conflict resolution | Rust, React |
| **remarkable-templates** | Custom template builder | React, Vite |
| **remarkable-notes-sync** | Obsidian/Notion bidirectional sync | Rust, Python |
| **remarkable-patcher** | Firmware binary patcher (9 patches) | Rust |
| **remarkable-dbus** | D-Bus MDM control CLI | Python |
| **remarkable-deploy** | Docker, systemd, Caddy deployment | Shell, Docker |

## 📦 Related Repositories

- [remarkable-rs](https://github.com/coleleavitt/remarkable-rs) - Core Rust library (87K LOC, 15 crates)
- [remarkable-server](https://github.com/coleleavitt/remarkable-server) - Local sync server (50K LOC)
- [remarkable-research](https://github.com/coleleavitt/remarkable-research) - Security research & documentation

## 🚀 Quick Start

### Web Portal
```bash
cd remarkable-portal
npm install && npm run dev
# Open http://localhost:5173
```

### Desktop App
```bash
cd remarkable-desktop
npm install && npm run tauri:dev
```

### Mobile App
```bash
cd remarkable-mobile
npm install && npm start
# Scan QR with Expo Go
```

### Screen Share
```bash
cd remarkable-screenshare
cargo build --release
./target/release/remarkable-screenshare web --usb
```

### Real-time Collaboration
```bash
cd remarkable-collab
cargo run -- --port 8090
# Open http://localhost:8090
```

### Template Builder
```bash
cd remarkable-templates
npm install && npm run dev
```

### Firmware Patcher
```bash
cd remarkable-patcher
cargo build --release
./target/release/rm-patcher analyze /path/to/xochitl
./target/release/rm-patcher patch /path/to/xochitl --all
```

### D-Bus MDM Control
```bash
cd remarkable-dbus
./remarkable-mdm enable-ssh
./remarkable-mdm get-policies
```

### Notes Sync
```bash
cd remarkable-notes-sync
cargo build --release
./target/release/remarkable-sync obsidian push --vault ~/Notes
./target/release/remarkable-sync notion push --token $NOTION_TOKEN
```

## 📊 Statistics

- **Total LOC**: ~175,000+
- **Languages**: Rust, TypeScript, Python
- **Tests**: 200+
- **Firmware versions analyzed**: 440
- **SDK versions analyzed**: 58

## 🔧 Features

### Sync & Storage
- Local sync server (drop-in cloud replacement)
- Cloud storage integration (Google Drive, Dropbox, OneDrive)
- Email-to-device (SMTP server)
- RSS/Newsletter to EPUB
- Read-it-later (Pocket, Instapaper, Wallabag, Omnivore)
- Obsidian/Notion sync

### Document Management
- Full .rm format support (v3/v5/v6)
- SVG/PNG/PDF export
- Version history with diff
- Full-text search (FTS5)
- Encrypted backups (AES-256-GCM, Age, GPG)

### Device Control
- D-Bus MDM (SSH enable, passcode control)
- Firmware patching (9 patches: telemetry, subscription, beta features)
- Custom templates

### Collaboration
- Real-time multi-user editing
- CRDT conflict resolution
- Visual stroke diff

### Local OCR
- TrOCR/PaddleOCR handwriting recognition
- MyScript replacement via mitmproxy

## 📝 License

MIT License - see individual project directories for details.

## 🙏 Acknowledgments

Built on top of extensive reverse engineering of reMarkable protocols, firmware analysis of 440 versions, and official SDK documentation from developer.remarkable.com.
