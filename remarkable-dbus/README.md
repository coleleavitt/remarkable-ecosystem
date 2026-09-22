# reMarkable D-Bus MDM CLI

Command-line tool for managing reMarkable device policies via the MDM D-Bus interface.

## Overview

The reMarkable MDM (Mobile Device Management) subsystem exposes a D-Bus interface
(`com.remarkable.devicepolicy.MDMAgent1`) without explicit access control. This tool
exploits that to manage device policies directly.

**WARNING**: The device passcode IS the Linux root password.

## Requirements

- SSH access to device (root)
- Device connected via USB (10.11.99.1) or WiFi

## Installation

```bash
# Make executable
chmod +x remarkable-mdm

# Optional: symlink to PATH
ln -s $(pwd)/remarkable-mdm ~/.local/bin/
```

## Commands

### Status & Discovery

```bash
# Check device connectivity and MDM status
./remarkable-mdm status

# Introspect D-Bus interface to discover methods
./remarkable-mdm introspect

# List all MDM policies
./remarkable-mdm get-policies
```

### SSH Control

```bash
# Enable SSH access
./remarkable-mdm enable-ssh

# Disable SSH (DANGEROUS - may lock you out!)
./remarkable-mdm disable-ssh
./remarkable-mdm disable-ssh --force  # Skip confirmation
```

### Passcode Management

```bash
# Set device passcode (also sets root password!)
./remarkable-mdm set-passcode 1234

# Remove passcode requirement
./remarkable-mdm clear-passcode
./remarkable-mdm clear-passcode --force  # Skip confirmation
```

### Generic Policy Control

```bash
# Set any policy by name
./remarkable-mdm set-policy SSHPolicy false
./remarkable-mdm set-policy PincodePolicy true
./remarkable-mdm set-policy DeveloperModePolicy false
```

## Known Policies

| Policy | Description |
|--------|-------------|
| `SSHPolicy` | Enable/disable SSH access |
| `PincodePolicy` | Passcode requirements |
| `RetailPolicy` | Demo mode settings |
| `LogoutPolicy` | Session logout behavior |
| `SoftwareAutoInstallPolicy` | Auto-update control |
| `DeveloperModePolicy` | Developer mode access |
| `AutosleepDelayDevicePolicy` | Power management |

## Options

```
-d, --device IP    Device IP address (default: 10.11.99.1)
-f, --force        Skip confirmation prompts
```

## Security Notes

1. **No D-Bus ACL**: The MDM interface has no explicit D-Bus policy file, unlike other reMarkable services.
2. **Root access**: All operations require root SSH access.
3. **Passcode = Root**: Device passcode is the Linux root password.
4. **Persistence risk**: Disabling SSH can permanently lock you out.

## Reference

See `~/SiteResearch/remarkable/DBUS_MDM_EXPLOIT.md` for full exploit analysis.

## D-Bus Interface

- **Destination**: `com.remarkable.devicepolicy.MDMAgent1`
- **Path**: `/`
- **Methods**: `GetEnrolledState`, `GetPolicies`, `SetPolicyStatus`
- **Properties**: `blockSsh`, `blockDeveloperMode`, `blockRetail`, `mdmEnabled`
