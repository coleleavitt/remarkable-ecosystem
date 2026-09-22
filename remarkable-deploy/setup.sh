#!/bin/bash
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_MODE="${1:-}"

# Default values
DOMAIN="${DOMAIN:-localhost}"
DATA_DIR="${DATA_DIR:-/var/lib/remarkable-server}"
CERTS_DIR="${CERTS_DIR:-/etc/remarkable-server/certs}"
INSTALL_DIR="${INSTALL_DIR:-/opt/remarkable-server}"
USER="${RM_USER:-remarkable}"
PORT="${RM_PORT:-8080}"

banner() {
    echo ""
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}          reMarkable Local Sync Server Setup              ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root (use sudo)"
        exit 1
    fi
}

check_deps() {
    info "Checking dependencies..."
    local deps=(python3 pip3 openssl curl)
    local missing=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing+=("$dep")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing dependencies: ${missing[*]}"
        info "Install with: apt install python3 python3-pip python3-venv openssl curl"
        exit 1
    fi
    success "All dependencies present"
}

prompt_config() {
    echo ""
    info "Configuration"
    echo "─────────────────────────────────────────────────────"
    
    # Domain
    read -rp "Domain (or localhost for local dev) [$DOMAIN]: " input
    DOMAIN="${input:-$DOMAIN}"
    
    # Port
    read -rp "Server port [$PORT]: " input
    PORT="${input:-$PORT}"
    
    # Data directory
    read -rp "Data directory [$DATA_DIR]: " input
    DATA_DIR="${input:-$DATA_DIR}"
    
    # Install method
    echo ""
    echo "Installation methods:"
    echo "  1) Docker Compose (recommended)"
    echo "  2) Systemd (bare metal)"
    echo "  3) Docker only (no reverse proxy)"
    read -rp "Choose method [1]: " method
    INSTALL_METHOD="${method:-1}"
    
    # TLS for bare metal
    if [[ "$INSTALL_METHOD" == "2" ]]; then
        echo ""
        echo "TLS options:"
        echo "  1) Let's Encrypt (requires public domain)"
        echo "  2) Self-signed certificates"
        echo "  3) Provide own certificates"
        read -rp "Choose TLS method [2]: " tls_method
        TLS_METHOD="${tls_method:-2}"
    fi
    
    echo ""
    echo "─────────────────────────────────────────────────────"
    info "Configuration summary:"
    echo "  Domain:     $DOMAIN"
    echo "  Port:       $PORT"
    echo "  Data dir:   $DATA_DIR"
    echo "  Method:     $(case $INSTALL_METHOD in 1) echo "Docker Compose";; 2) echo "Systemd";; 3) echo "Docker";; esac)"
    echo ""
    read -rp "Proceed with installation? [Y/n]: " confirm
    if [[ "${confirm,,}" == "n" ]]; then
        info "Installation cancelled"
        exit 0
    fi
}

create_user() {
    info "Creating system user..."
    if id "$USER" &>/dev/null; then
        success "User $USER already exists"
    else
        useradd --system --create-home --shell /sbin/nologin "$USER"
        success "Created user $USER"
    fi
}

create_dirs() {
    info "Creating directories..."
    mkdir -p "$DATA_DIR" "$CERTS_DIR" "$INSTALL_DIR"
    chown -R "$USER:$USER" "$DATA_DIR"
    chmod 700 "$DATA_DIR" "$CERTS_DIR"
    success "Directories created"
}

generate_certs() {
    info "Generating certificates..."
    
    case "${TLS_METHOD:-2}" in
        1)  # Let's Encrypt (handled by Caddy in docker mode)
            warn "Let's Encrypt will be configured via Caddy"
            ;;
        2)  # Self-signed
            if [[ ! -f "$CERTS_DIR/server.crt" ]]; then
                openssl req -x509 -nodes -days 3650 \
                    -newkey rsa:4096 \
                    -keyout "$CERTS_DIR/server.key" \
                    -out "$CERTS_DIR/server.crt" \
                    -subj "/CN=$DOMAIN" \
                    -addext "subjectAltName=DNS:$DOMAIN,DNS:*.remarkable.com,DNS:tectonic.remarkable.com" \
                    2>/dev/null
                chmod 600 "$CERTS_DIR/server.key"
                chown -R "$USER:$USER" "$CERTS_DIR"
                success "Self-signed certificates generated"
            else
                success "Certificates already exist"
            fi
            ;;
        3)  # User-provided
            read -rp "Path to certificate file: " cert_path
            read -rp "Path to key file: " key_path
            cp "$cert_path" "$CERTS_DIR/server.crt"
            cp "$key_path" "$CERTS_DIR/server.key"
            chmod 600 "$CERTS_DIR/server.key"
            chown -R "$USER:$USER" "$CERTS_DIR"
            success "Certificates installed"
            ;;
    esac
}

install_python() {
    info "Setting up Python environment..."
    python3 -m venv "$INSTALL_DIR/venv"
    "$INSTALL_DIR/venv/bin/pip" install --upgrade pip
    "$INSTALL_DIR/venv/bin/pip" install -r "$SCRIPT_DIR/requirements.txt"
    
    # Copy remarkable package
    cp -r "$SCRIPT_DIR/remarkable" "$INSTALL_DIR/"
    chown -R "$USER:$USER" "$INSTALL_DIR"
    success "Python environment ready"
}

install_systemd() {
    info "Installing systemd services..."
    cp "$SCRIPT_DIR/systemd/"*.service /etc/systemd/system/
    cp "$SCRIPT_DIR/systemd/"*.timer /etc/systemd/system/
    
    # Create env file
    cat > /etc/remarkable-server/env << EOF
RM_HOST=0.0.0.0
RM_PORT=$PORT
RM_DATA_DIR=$DATA_DIR
RM_CERTS_DIR=$CERTS_DIR
RM_LOG_LEVEL=info
EOF
    chmod 600 /etc/remarkable-server/env
    
    systemctl daemon-reload
    systemctl enable remarkable-server.service
    systemctl enable remarkable-server-cleanup.timer
    success "Systemd services installed"
}

install_docker() {
    info "Setting up Docker deployment..."
    
    # Create env file
    cat > "$SCRIPT_DIR/.env" << EOF
DOMAIN=$DOMAIN
RM_PORT=$PORT
RM_LOG_LEVEL=info
EOF
    
    # Build and start
    cd "$SCRIPT_DIR"
    if [[ "$INSTALL_METHOD" == "1" ]]; then
        docker compose build
        docker compose up -d
    else
        docker build -t remarkable-server:latest .
        docker run -d \
            --name remarkable-server \
            --restart unless-stopped \
            -p "$PORT:8080" \
            -v remarkable-data:/data \
            -v remarkable-certs:/certs \
            remarkable-server:latest
    fi
    success "Docker containers started"
}

start_service() {
    info "Starting service..."
    case "$INSTALL_METHOD" in
        1|3)
            cd "$SCRIPT_DIR"
            docker compose ps
            ;;
        2)
            systemctl start remarkable-server
            systemctl start remarkable-server-cleanup.timer
            systemctl status remarkable-server --no-pager
            ;;
    esac
}

print_next_steps() {
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║${NC}                 Installation Complete!                   ${GREEN}║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    info "Next steps:"
    echo ""
    echo "  1. Access web portal:"
    if [[ "$DOMAIN" == "localhost" ]]; then
        echo "     https://localhost:$PORT"
    else
        echo "     https://$DOMAIN"
    fi
    echo ""
    echo "  2. Pair your reMarkable device:"
    echo "     - Generate pairing code via web portal or CLI"
    echo "     - Enter code on device: Settings > Storage > Connect to server"
    echo ""
    echo "  3. Configure device to use local server:"
    echo "     Option A: Add to /etc/hosts on device:"
    echo "       <server-ip>  tectonic.remarkable.com"
    echo ""
    echo "     Option B: Use mitmproxy for HTTPS interception"
    echo ""
    if [[ "$INSTALL_METHOD" == "2" ]]; then
        echo "  Service commands:"
        echo "     systemctl status remarkable-server"
        echo "     journalctl -u remarkable-server -f"
    else
        echo "  Docker commands:"
        echo "     docker compose logs -f"
        echo "     docker compose ps"
    fi
    echo ""
}

# Main
banner

case "$INSTALL_MODE" in
    --uninstall)
        check_root
        info "Uninstalling remarkable-server..."
        systemctl stop remarkable-server 2>/dev/null || true
        systemctl disable remarkable-server 2>/dev/null || true
        rm -f /etc/systemd/system/remarkable-server.*
        systemctl daemon-reload
        success "Systemd services removed"
        info "Data preserved in $DATA_DIR (delete manually if needed)"
        exit 0
        ;;
    --docker-only)
        INSTALL_METHOD=3
        ;;
    --systemd)
        INSTALL_METHOD=2
        ;;
    "")
        # Interactive mode
        ;;
    *)
        echo "Usage: $0 [--docker-only|--systemd|--uninstall]"
        exit 1
        ;;
esac

check_root
check_deps
prompt_config
create_user
create_dirs

case "$INSTALL_METHOD" in
    1|3)
        generate_certs
        install_docker
        ;;
    2)
        generate_certs
        install_python
        install_systemd
        start_service
        ;;
esac

print_next_steps
