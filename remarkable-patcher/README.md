# reMarkable Patcher

Binary patcher for reMarkable xochitl firmware modifications.

## Features

- **Analyze** binaries to detect firmware version and patch status
- **Apply patches** to enable hidden features and customize behavior
- **Automatic backups** with rollback capability
- **Pattern scanning** for unknown firmware versions
- **Resource extraction** for QML/image analysis

## Installation

```bash
cargo install --path .
```

## Usage

### Analyze a binary

```bash
rm-patcher analyze /path/to/xochitl
```

Shows:
- Binary information (size, SHA256, architecture)
- Detected firmware version
- Current patch status
- Pattern scan results

### List available patches

```bash
rm-patcher list
```

### Apply patches

```bash
# Apply specific patches
rm-patcher patch /path/to/xochitl -p insecure-settings -p disable-telemetry

# Apply all patches
rm-patcher patch /path/to/xochitl --all

# Output to different file (keeps original intact)
rm-patcher patch /path/to/xochitl -o /path/to/xochitl.patched --all

# Skip backup (not recommended)
rm-patcher patch /path/to/xochitl --all --no-backup
```

### Verify a binary

```bash
rm-patcher verify /path/to/xochitl
```

### Restore from backup

```bash
rm-patcher restore /path/to/backup
```

### Extract resources

```bash
rm-patcher extract /path/to/xochitl -o extracted/
```

## Available Patches

| Patch | Category | Risk | Description |
|-------|----------|------|-------------|
| `insecure-settings` | Developer | Low | Enable hidden InsecureSettings menu |
| `disable-telemetry` | Telemetry | Low | Disable analytics/telemetry collection |
| `disable-memfault` | Telemetry | Low | Disable Memfault crash reporting |
| `bypass-subscription` | Subscription | Medium | Allow third-party integrations without Connect |
| `enable-beta` | Beta | Medium | Enable experimental features |
| `skip-dev-password` | Developer | Low | Skip developer mode password |
| `redirect-cloud` | Subscription | High | Redirect sync to local server |
| `enable-debug-grid` | Developer | Low | Enable UI debug grid overlay |
| `disable-ota` | Subscription | Medium | Disable automatic OTA updates |

## Safety

- **Always test on a development device first**
- Backups are created automatically (stored in `~/.local/share/remarkable-patcher/backups/`)
- Patches are pattern-based and version-agnostic where possible
- Unknown firmware versions are detected and warned about

## Technical Details

### Binary Analysis

The patcher uses pattern matching to locate patch sites:

1. **String patterns** - Find configuration/module names
2. **ARM instruction patterns** - Locate function prologues and checks
3. **Cross-references** - Follow pointers to related code

### Patch Application

Patches work by:

1. Finding the "before" pattern in the binary
2. Replacing it with the "after" pattern (same length)
3. Verifying the patch was applied correctly

### Supported Firmware

Tested on firmware versions:
- 3.27.x - 3.29.x (RM2)
- 3.26.x - 3.28.x (RM1, Ferrari, Chiappa)
- Earlier versions may work but are untested

## Building from Source

```bash
git clone https://github.com/remarkable-rs/remarkable-patcher
cd remarkable-patcher
cargo build --release
```

## References

- [reMarkable Firmware Archive](https://github.com/remarkable-rs/remarkable-research)
- [FIRMWARE_MODDING.md](https://github.com/remarkable-rs/remarkable-research/blob/main/FIRMWARE_MODDING.md)
- IDA databases for reverse engineering

## License

MIT OR Apache-2.0

## Disclaimer

This tool is for educational and research purposes. Modifying firmware may:
- Void your warranty
- Cause device instability
- Prevent future OTA updates
- Be against reMarkable's Terms of Service

Use at your own risk. Always keep backups of your original firmware.
