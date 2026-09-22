# reMarkable Mobile Sync

A cross-platform mobile app for syncing documents with reMarkable tablets via remarkable-server.

## Features

- **Document Browser**: Navigate folders and view documents
- **SVG Preview**: View reMarkable notebook pages as SVG
- **PDF/EPUB Upload**: Upload documents from your device
- **Offline Mode**: SQLite-based cache for offline access
- **Sync Status**: Real-time sync progress and status
- **Push Notifications**: Get notified about sync events
- **Share Extension (iOS)**: Send documents from other apps
- **Intent Filters (Android)**: Open PDFs/EPUBs directly

## Requirements

- Node.js 18+
- Expo CLI
- iOS: Xcode 15+ (for native build)
- Android: Android Studio (for native build)
- remarkable-server running locally or accessible

## Setup

```bash
# Install dependencies
npm install

# Start development server
npm start

# Run on iOS simulator
npm run ios

# Run on Android emulator
npm run android
```

## Configuration

### Connecting to Server

1. Launch the app
2. Select a preset server or enter a custom URL
3. The app will connect and sync your documents

### Server Presets

- **reMarkable (USB)**: `http://10.11.99.1:8080` - Direct USB connection
- **Local Server (WiFi)**: `http://remarkable.local:8080` - Local network

## Architecture

```
remarkable-mobile/
├── app/                    # Expo Router screens
│   ├── (tabs)/            # Tab-based navigation
│   │   ├── index.tsx      # Documents tab
│   │   ├── recent.tsx     # Recent documents
│   │   └── favorites.tsx  # Favorites/pinned
│   ├── preview/[id].tsx   # Document preview
│   ├── settings.tsx       # Settings modal
│   ├── connect.tsx        # Server connection
│   └── share.tsx          # Share intent handler
├── src/
│   ├── components/        # React components
│   │   ├── documents/     # Document browser
│   │   ├── preview/       # SVG preview
│   │   ├── sync/          # Sync status
│   │   ├── upload/        # Upload UI
│   │   └── ui/            # Common UI
│   ├── hooks/             # React hooks
│   ├── services/          # API, cache, notifications
│   ├── stores/            # Zustand state
│   └── types/             # TypeScript types
└── plugins/               # Expo config plugins
    └── ShareExtension/    # iOS Share Extension
```

## API Integration

The app integrates with remarkable-server via REST API:

| Endpoint | Purpose |
|----------|---------|
| `/sync/v3/root` | Get sync root hash |
| `/gentree/v1/` | Get document tree |
| `/sync/v3/files` | CRUD documents |
| `/sync/v3/files/:id/download` | Download document |
| `/export/svg/:id` | Export page as SVG |
| `/export/png/:id` | Export page as PNG |

## Building for Production

```bash
# Create native builds
npx expo prebuild

# Build iOS
cd ios && xcodebuild

# Build Android
cd android && ./gradlew assembleRelease
```

## iOS Share Extension

The Share Extension allows sending PDFs and EPUBs from other apps:

1. Share a PDF/EPUB from any app
2. Select "Send to reMarkable"
3. Choose a folder (optional)
4. Tap Post to upload

## Offline Mode

Documents are cached locally in SQLite for offline access:
- Document metadata is stored in SQLite
- File content is cached in the file system
- Pending uploads are queued and retried
- Sync state is preserved across sessions

## Tech Stack

- **React Native** with Expo SDK 52
- **TypeScript** for type safety
- **Expo Router** for navigation
- **TanStack Query** for data fetching
- **Zustand** for state management
- **SQLite** for offline cache
- **react-native-svg** for SVG rendering

## License

MIT
