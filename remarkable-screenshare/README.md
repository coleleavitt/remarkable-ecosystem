# remarkable-screenshare

Screen share viewer for reMarkable tablets with WebRTC and USB framebuffer capture support.

## Features

- **WebRTC Viewer** - Browser-based viewing via cloud signaling (requires Connect subscription)
- **USB Framebuffer Viewer** - Direct capture over USB (no subscription needed)
- **MQTT Signaling** - Integration with reMarkable's cloud infrastructure
- **Low-Latency Display** - Real-time screen updates
- **Recording** - Capture sessions as PNG/JPEG sequences or WebM/MP4 video
- **Web Interface** - Browser-based viewer with WebSocket streaming

## Installation

```bash
# From crates.io (when published)
cargo install remarkable-screenshare

# From source
git clone https://github.com/youruser/remarkable-screenshare
cd remarkable-screenshare
cargo build --release
```

## Usage

### Web Viewer (USB Mode)

```bash
# Connect device via USB and start web viewer
remarkable-screenshare web --usb

# Open http://localhost:8088 in browser
```

### Web Viewer (WebRTC Mode)

```bash
# Requires device tokens from xochitl.conf
remarkable-screenshare web \
    --device-token /path/to/device_token \
    --user-token /path/to/user_token
```

### Single Frame Capture

```bash
remarkable-screenshare capture --output screenshot.png
```

### Recording

```bash
# Record for 30 seconds
remarkable-screenshare record --output ./session --fps 10 --duration 30

# Convert to video
remarkable-screenshare encode \
    --input "session/frame_%06d.png" \
    --output session.webm \
    --fps 10 \
    --format webm
```

### Device Info

```bash
remarkable-screenshare info
```

## Protocol Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    remarkable-screenshare                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐           │
│  │  WebRTC     │   │   MQTT      │   │    USB      │           │
│  │  Viewer     │   │  Signaling  │   │  Capture    │           │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘           │
│         │                 │                 │                   │
│  ┌──────┴─────────────────┴─────────────────┴──────┐           │
│  │              Frame Processor                     │           │
│  │  (RFB decode, grayscale conversion)              │           │
│  └──────────────────────┬───────────────────────────┘           │
│                         │                                       │
│  ┌──────────────────────┴───────────────────────────┐           │
│  │              Output Layer                         │           │
│  │  - Browser (WebSocket → HTML5 Canvas)             │           │
│  │  - Recording (PNG/JPEG sequence → Video)          │           │
│  └───────────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

### WebRTC Flow

1. Client connects to MQTT broker (VerneMQ)
2. Client subscribes to signaling topics
3. Client sends `request-offer` message
4. Device responds with SDP offer
5. Client sends SDP answer
6. ICE candidates exchanged
7. DataChannel established
8. RFB screen data flows over DataChannel

### USB Flow

1. SSH connection to device (10.11.99.1)
2. Read framebuffer device (/dev/fb0)
3. Convert 8-bit grayscale to image
4. Stream to browser via WebSocket

## Device Compatibility

| Device | USB Capture | WebRTC | Resolution |
|--------|-------------|--------|------------|
| reMarkable 2 | ✓ | ✓ | 1872×1404 |
| reMarkable Paper Pro | ✓ | ✓ | 2160×2880 |

## Token Extraction

Tokens are stored in `/home/root/.config/remarkable/xochitl.conf`:

```bash
ssh root@10.11.99.1 cat /home/root/.config/remarkable/xochitl.conf
```

Extract `devicetoken` and `usertoken` lines.

## Dependencies

- **WebRTC**: webrtc-rs (pure Rust WebRTC)
- **MQTT**: rumqttc
- **SSH**: russh
- **Image**: image crate
- **Web**: axum

## Optional Features

```toml
[features]
default = []
recording = ["gstreamer", "gstreamer-app", "gstreamer-video"]
```

## License

MIT

## Acknowledgments

Based on reverse engineering of:
- reMarkable xochitl firmware
- reMarkable iOS/Android mobile apps
- reMarkable Desktop app

Protocol documentation: [reMarkable Research](https://github.com/youruser/remarkable-research)
