# remarkable-server Deployment Package

Production deployment for reMarkable local sync server.

## Overview

This package provides everything needed to run a self-hosted reMarkable sync server as a drop-in replacement for reMarkable cloud. Your documents stay on your own infrastructure.

```
┌─────────────────┐      HTTPS       ┌─────────────────┐
│   reMarkable    │ ◄──────────────► │     Caddy       │
│     Device      │                  │  (Reverse Proxy)│
└─────────────────┘                  └────────┬────────┘
                                              │
                    ┌─────────────────────────┼─────────────────────────┐
                    │                         │                         │
                    ▼                         ▼                         ▼
            ┌───────────────┐        ┌───────────────┐        ┌───────────────┐
            │  Sync Server  │        │  Web Portal   │        │   Storage     │
            │  (Python)     │        │   (React)     │        │   (SQLite)    │
            └───────────────┘        └───────────────┘        └───────────────┘
```

## Quick Start

### Docker (Recommended)

```bash
# Clone and setup
cd remarkable-deploy
cp .env.example .env
vim .env  # Configure domain and options

# Start all services
docker compose up -d

# View logs
docker compose logs -f
```

### Bare Metal (Systemd)

```bash
# Run interactive setup
sudo ./setup.sh

# Or non-interactive
sudo DOMAIN=sync.example.com ./setup.sh --systemd
```

## Directory Structure

```
remarkable-deploy/
├── Dockerfile              # Multi-stage server image
├── docker-compose.yml      # Full stack with Caddy
├── Caddyfile               # Reverse proxy + auto TLS
├── systemd/                # Systemd unit files
│   ├── remarkable-server.service
│   ├── remarkable-server.socket
│   └── remarkable-server-cleanup.*
├── setup.sh                # Interactive installer
├── .env.example            # Configuration template
├── requirements.txt        # Python dependencies
├── entrypoint.sh           # Container entrypoint
└── portal/                 # Web portal (optional)
```

## Configuration

### Essential Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `DOMAIN` | `localhost` | Domain for TLS cert |
| `RM_PORT` | `8080` | Internal server port |
| `RM_DATA_DIR` | `/data` | Document storage |
| `RM_LOG_LEVEL` | `info` | Logging verbosity |

### TLS Options

1. **Auto (Let's Encrypt)** - Default for public domains
   ```env
   DOMAIN=sync.example.com
   ```

2. **Self-signed** - For local/testing
   ```env
   DOMAIN=localhost
   ```

3. **Custom certificates** - Bring your own
   ```env
   RM_TLS_MODE=manual
   RM_TLS_CERT=/path/to/cert.pem
   RM_TLS_KEY=/path/to/key.pem
   ```

## Device Setup

### 1. Generate Pairing Code

Via web portal or CLI:
```bash
remarkable pair create
```

### 2. Configure Device

**Option A: /etc/hosts modification** (requires SSH access)

```bash
# On device
ssh root@10.11.99.1
echo "<server-ip>  tectonic.remarkable.com" >> /etc/hosts
```

**Option B: mitmproxy** (transparent interception)

```bash
# On local machine
mitmproxy --mode transparent --ssl-insecure
```

### 3. Enter Pairing Code

On device: Settings → Storage → Connect to server → Enter code

## Health Checks

Built-in health endpoint:
```bash
curl https://your-domain/health
# {"status": "ok", "version": "1.0.0", "devices": 1}
```

Docker health check runs every 30 seconds.

## Monitoring

### Logs

```bash
# Docker
docker compose logs -f server

# Systemd
journalctl -u remarkable-server -f
```

### Metrics

Prometheus metrics available at `/metrics` (when enabled):
```env
RM_METRICS_ENABLED=true
```

## Backup

Data is stored in:
- **Docker**: `remarkable-data` volume
- **Bare metal**: `/var/lib/remarkable-server`

```bash
# Docker backup
docker run --rm -v remarkable-data:/data -v $(pwd):/backup \
    alpine tar czf /backup/remarkable-backup.tar.gz /data

# Systemd backup
tar czf remarkable-backup.tar.gz /var/lib/remarkable-server
```

## Security Considerations

1. **TLS Required**: Device will not sync over plain HTTP
2. **Certificate Pinning**: reMarkable pins cloud certs; local server needs device-side trust
3. **Firewall**: Only expose ports 80/443 to trusted networks
4. **Updates**: Keep server updated for security patches

## Troubleshooting

### Device won't connect

1. Verify DNS/hosts resolution:
   ```bash
   # On device
   ping tectonic.remarkable.com
   ```

2. Check server is running:
   ```bash
   curl -k https://your-server/health
   ```

3. Verify certificate trust:
   - Self-signed certs require manual trust on device

### Sync errors

1. Check server logs for errors
2. Verify storage has space
3. Check file permissions on data directory

### Port conflicts

```bash
# Check what's using port 8080
ss -tlnp | grep 8080
```

## API Reference

### Sync v3 API

| Endpoint | Description |
|----------|-------------|
| `GET /sync/v3/root` | Get root hash |
| `GET /sync/v3/files/{hash}` | Download file |
| `PUT /sync/v3/files/{hash}` | Upload file |
| `POST /sync/v3/sync` | Sync operation |

### Device API

| Endpoint | Description |
|----------|-------------|
| `POST /device/register` | Initiate pairing |
| `GET /device/{id}/status` | Device status |
| `DELETE /device/{id}` | Unpair device |

### Portal API

| Endpoint | Description |
|----------|-------------|
| `GET /api/documents` | List documents |
| `GET /api/documents/{id}` | Get document |
| `GET /api/documents/{id}/export` | Export as PDF |

## Development

### Local development

```bash
# Start with hot reload
docker compose -f docker-compose.yml -f docker-compose.dev.yml up
```

### Running tests

```bash
cd remarkable-py
pytest tests/
```

## License

MIT License - See LICENSE file.

## Related Projects

- [remarkable-py](https://github.com/coleleavitt/remarkable-sdk) - Python SDK
- [remarkable-rs](https://github.com/coleleavitt/remarkable-rs) - Rust library
- [rmfakecloud](https://github.com/ddvk/rmfakecloud) - Alternative server
