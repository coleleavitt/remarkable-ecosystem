"""
Local sync server for reMarkable.

Provides a local alternative to reMarkable cloud sync,
compatible with the sync v3 API, including device pairing.
"""

from __future__ import annotations

import asyncio
import hashlib
import base64
import secrets
import string
import json
import logging
import os
import secrets
import string
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any, Callable
from uuid import uuid4

logger = logging.getLogger(__name__)

# Server version
SERVER_VERSION = "1.0.0"

# Pairing code config
PAIRING_CODE_LENGTH = 8
PAIRING_CODE_EXPIRY_SECONDS = 300  # 5 minutes


@dataclass
class FileEntry:
    """A file in the local sync store."""
    hash: str
    document_id: str
    filename: str
    size: int
    data: bytes | None = None
    
    @classmethod
    def from_path(cls, path: Path, doc_id: str) -> FileEntry:
        """Create entry from file path."""
        data = path.read_bytes()
        file_hash = hashlib.sha256(data).hexdigest()
        return cls(
            hash=file_hash,
            document_id=doc_id,
            filename=path.name,
            size=len(data),
            data=data,
        )


@dataclass
class PairingRequest:
    """A pending pairing request."""
    code: str
    device_id: str
    device_type: str
    device_desc: str
    created_at: datetime
    expires_at: datetime
    approved: bool = False
    token: str | None = None
    
    @property
    def is_expired(self) -> bool:
        return datetime.now() > self.expires_at


@dataclass
class RegisteredDevice:
    """A paired/registered device."""
    device_id: str
    device_type: str
    device_desc: str
    token: str
    registered_at: datetime
    last_seen: datetime | None = None
    
    def to_dict(self) -> dict[str, Any]:
        return {
            "device_id": self.device_id,
            "device_type": self.device_type,
            "device_desc": self.device_desc,
            "registered_at": self.registered_at.isoformat(),
            "last_seen": self.last_seen.isoformat() if self.last_seen else None,
        }
    
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> RegisteredDevice:
        return cls(
            device_id=data["device_id"],
            device_type=data["device_type"],
            device_desc=data["device_desc"],
            token=data.get("token", ""),
            registered_at=datetime.fromisoformat(data["registered_at"]),
            last_seen=datetime.fromisoformat(data["last_seen"]) if data.get("last_seen") else None,
        )


@dataclass
class SyncState:
    """Local sync state."""
    root_hash: str = ""
    generation: int = 0
    files: dict[str, FileEntry] = field(default_factory=dict)
    documents: dict[str, dict[str, Any]] = field(default_factory=dict)
    
    def add_file(self, entry: FileEntry) -> None:
        """Add or update a file."""
        self.files[entry.hash] = entry
    
    def get_file(self, file_hash: str) -> FileEntry | None:
        """Get file by hash."""
        return self.files.get(file_hash)
    
    def compute_root(self) -> str:
        """Compute root hash from current state."""
        # Combine all document hashes
        doc_hashes = sorted(
            f.hash for f in self.files.values() 
            if f.filename.endswith(".docSchema")
        )
        combined = "".join(doc_hashes).encode()
        self.root_hash = hashlib.sha256(combined).hexdigest()
        self.generation += 1
        return self.root_hash
    
    def to_root_index(self) -> dict[str, Any]:
        """Generate root.docSchema content."""
        file_list = [
            {
                "hash": f.hash,
                "documentId": f.document_id,
                "filename": f.filename,
                "size": f.size,
            }
            for f in self.files.values()
        ]
        
        return {
            "files": file_list,
            "schemaVersion": 3,
        }


def generate_pairing_code() -> str:
    """Generate a random pairing code."""
    chars = string.ascii_lowercase + string.digits
    return "".join(secrets.choice(chars) for _ in range(PAIRING_CODE_LENGTH))


def generate_token() -> str:
    """Generate a secure token for device authentication."""
    return secrets.token_urlsafe(32)




@dataclass
class PairingCode:
    """Active pairing code."""
    code: str
    created_at: float
    expires_at: float
    device_info: dict | None = None
    completed: bool = False
    device_id: str | None = None
    device_token: str | None = None
    user_token: str | None = None


class PairingManager:
    """Manage device pairing codes and tokens."""
    
    def __init__(self, secret_key: str | None = None, code_expiry_seconds: int = 300):
        self.secret_key = secret_key or secrets.token_hex(32)
        self.code_expiry = code_expiry_seconds
        self.active_codes: dict[str, PairingCode] = {}
        self.registered_devices: dict[str, dict] = {}
    
    def generate_code(self) -> str:
        """Generate a new 8-character pairing code."""
        self._cleanup_expired()
        
        chars = string.ascii_lowercase
        while True:
            code = ''.join(secrets.choice(chars) for _ in range(8))
            if code not in self.active_codes:
                break
        
        now = time.time()
        self.active_codes[code] = PairingCode(
            code=code,
            created_at=now,
            expires_at=now + self.code_expiry,
        )
        
        return code
    
    def get_code_status(self, code: str) -> dict:
        """Get status of a pairing code."""
        if code not in self.active_codes:
            return {"status": "not_found"}
        
        pc = self.active_codes[code]
        
        if time.time() > pc.expires_at:
            return {"status": "expired"}
        
        if pc.completed:
            return {
                "status": "completed",
                "device_id": pc.device_id,
            }
        
        return {
            "status": "pending",
            "expires_in": int(pc.expires_at - time.time()),
        }
    
    def complete_pairing(self, code: str, device_info: dict) -> dict:
        """Complete pairing and return tokens."""
        if code not in self.active_codes:
            raise ValueError("Invalid code")
        
        pc = self.active_codes[code]
        
        if time.time() > pc.expires_at:
            raise ValueError("Code expired")
        
        if pc.completed:
            raise ValueError("Code already used")
        
        serial = device_info.get("serial", secrets.token_hex(6))
        device_id = f"RM-{serial[-6:]}"
        
        device_token = self._generate_token(device_id, "device", hours=24*30)
        user_token = self._generate_token(device_id, "user", hours=3)
        
        pc.completed = True
        pc.device_info = device_info
        pc.device_id = device_id
        pc.device_token = device_token
        pc.user_token = user_token
        
        self.registered_devices[device_id] = {
            "device_id": device_id,
            "device_info": device_info,
            "registered_at": time.time(),
        }
        
        return {
            "device_id": device_id,
            "device_token": device_token,
            "user_token": user_token,
        }
    
    def _generate_token(self, device_id: str, token_type: str, hours: int) -> str:
        """Generate a JWT-like token."""
        header = {"alg": "HS256", "typ": "JWT"}
        payload = {
            "device-id": device_id,
            "type": token_type,
            "iat": int(time.time()),
            "exp": int(time.time()) + (hours * 3600),
            "iss": "local-sync",
            "scopes": "sync:full intgr hwc screenshare",
        }
        
        if token_type == "user":
            payload["tectonic"] = {"region": "local"}
        
        header_b64 = base64.urlsafe_b64encode(json.dumps(header).encode()).decode().rstrip("=")
        payload_b64 = base64.urlsafe_b64encode(json.dumps(payload).encode()).decode().rstrip("=")
        
        import hmac
        signature = hmac.new(
            self.secret_key.encode(),
            f"{header_b64}.{payload_b64}".encode(),
            hashlib.sha256
        ).digest()
        sig_b64 = base64.urlsafe_b64encode(signature).decode().rstrip("=")
        
        return f"{header_b64}.{payload_b64}.{sig_b64}"
    
    def _cleanup_expired(self):
        """Remove expired codes."""
        now = time.time()
        expired = [c for c, pc in self.active_codes.items() 
                   if now > pc.expires_at + 60]
        for code in expired:
            del self.active_codes[code]


class LocalSyncServer:
    """
    Local sync server implementing the reMarkable sync v3 API.
    
    Can be used for:
    - Offline sync and backup
    - Development and testing
    - Self-hosted sync alternative
    
    Supports device pairing via one-time codes.
    
    Example:
        ```python
        server = LocalSyncServer(storage_dir="./sync_data")
        await server.start(host="0.0.0.0", port=8080)
        ```
    """
    
    def __init__(
        self,
        storage_dir: Path | str = "./remarkable_sync",
        *,
        require_auth: bool = False,
        auth_token: str | None = None,
        server_name: str = "reMarkable Local Server",
        auto_approve: bool = False,
    ) -> None:
        """
        Initialize the local sync server.
        
        Args:
            storage_dir: Directory for storing sync data
            require_auth: Require Bearer token authentication
            auth_token: Expected auth token (generated if not provided)
            server_name: Human-readable server name
            auto_approve: Automatically approve pairing requests
        """
        self.storage_dir = Path(storage_dir)
        self.storage_dir.mkdir(parents=True, exist_ok=True)
        
        self.require_auth = require_auth
        self.auth_token = auth_token or str(uuid4())
        self.server_name = server_name
        self.auto_approve = auto_approve
        
        self._state = SyncState()
        self._app = None
        self._server = None
        
        # Pairing state
        self._pending_pairings: dict[str, PairingRequest] = {}
        self._registered_devices: dict[str, RegisteredDevice] = {}
        self._device_tokens: dict[str, str] = {}  # token -> device_id
        
        self._load_state()
        self._pairing = PairingManager()
    
    def _load_state(self) -> None:
        """Load sync state from disk."""
        state_path = self.storage_dir / "state.json"
        if state_path.exists():
            try:
                data = json.loads(state_path.read_text())
                self._state.root_hash = data.get("root_hash", "")
                self._state.generation = data.get("generation", 0)
                self._state.documents = data.get("documents", {})
                
                # Load file entries
                for f_data in data.get("files", []):
                    entry = FileEntry(
                        hash=f_data["hash"],
                        document_id=f_data["document_id"],
                        filename=f_data["filename"],
                        size=f_data["size"],
                    )
                    self._state.files[entry.hash] = entry
                
                logger.info(
                    f"Loaded state: {len(self._state.files)} files, "
                    f"generation {self._state.generation}"
                )
            except Exception as e:
                logger.warning(f"Failed to load state: {e}")
        
        # Load registered devices
        devices_path = self.storage_dir / "devices.json"
        if devices_path.exists():
            try:
                data = json.loads(devices_path.read_text())
                for dev_data in data.get("devices", []):
                    device = RegisteredDevice.from_dict(dev_data)
                    device.token = dev_data.get("token", generate_token())
                    self._registered_devices[device.device_id] = device
                    self._device_tokens[device.token] = device.device_id
                
                logger.info(f"Loaded {len(self._registered_devices)} registered devices")
            except Exception as e:
                logger.warning(f"Failed to load devices: {e}")
    
    def _save_state(self) -> None:
        """Save sync state to disk."""
        state_path = self.storage_dir / "state.json"
        
        data = {
            "root_hash": self._state.root_hash,
            "generation": self._state.generation,
            "documents": self._state.documents,
            "files": [
                {
                    "hash": f.hash,
                    "document_id": f.document_id,
                    "filename": f.filename,
                    "size": f.size,
                }
                for f in self._state.files.values()
            ],
        }
        
        state_path.write_text(json.dumps(data, indent=2))
    
    def _save_devices(self) -> None:
        """Save registered devices to disk."""
        devices_path = self.storage_dir / "devices.json"
        
        data = {
            "devices": [
                {**dev.to_dict(), "token": dev.token}
                for dev in self._registered_devices.values()
            ],
        }
        
        devices_path.write_text(json.dumps(data, indent=2))
    
    def _get_file_path(self, file_hash: str) -> Path:
        """Get storage path for a file by hash."""
        # Use hash prefix for directory sharding
        prefix = file_hash[:2]
        return self.storage_dir / "files" / prefix / file_hash
    
    def _store_file(self, data: bytes, doc_id: str, filename: str) -> FileEntry:
        """Store a file and return its entry."""
        file_hash = hashlib.sha256(data).hexdigest()
        
        file_path = self._get_file_path(file_hash)
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.write_bytes(data)
        
        entry = FileEntry(
            hash=file_hash,
            document_id=doc_id,
            filename=filename,
            size=len(data),
        )
        self._state.add_file(entry)
        
        return entry
    
    def _load_file(self, file_hash: str) -> bytes | None:
        """Load file content by hash."""
        file_path = self._get_file_path(file_hash)
        if file_path.exists():
            return file_path.read_bytes()
        return None
    
    # === Pairing ===
    
    def create_pairing_code(
        self,
        device_id: str,
        device_type: str = "desktop-linux",
        device_desc: str = "Unknown",
    ) -> str:
        """
        Create a new pairing code for a device.
        
        Args:
            device_id: Unique device identifier
            device_type: Type of device
            device_desc: Device description
            
        Returns:
            8-character pairing code
        """
        # Clean up expired pairings
        self._cleanup_pairings()
        
        code = generate_pairing_code()
        now = datetime.now()
        
        self._pending_pairings[code] = PairingRequest(
            code=code,
            device_id=device_id,
            device_type=device_type,
            device_desc=device_desc,
            created_at=now,
            expires_at=now + timedelta(seconds=PAIRING_CODE_EXPIRY_SECONDS),
        )
        
        logger.info(f"Created pairing code {code} for device {device_id}")
        return code
    
    def approve_pairing(self, code: str) -> RegisteredDevice | None:
        """
        Approve a pending pairing request.
        
        Args:
            code: Pairing code to approve
            
        Returns:
            RegisteredDevice if approved, None if code invalid/expired
        """
        pairing = self._pending_pairings.get(code.lower())
        if pairing is None or pairing.is_expired:
            return None
        
        # Generate token and register device
        token = generate_token()
        now = datetime.now()
        
        device = RegisteredDevice(
            device_id=pairing.device_id,
            device_type=pairing.device_type,
            device_desc=pairing.device_desc,
            token=token,
            registered_at=now,
            last_seen=now,
        )
        
        self._registered_devices[device.device_id] = device
        self._device_tokens[token] = device.device_id
        
        # Mark pairing as approved
        pairing.approved = True
        pairing.token = token
        
        # Save devices
        self._save_devices()
        
        logger.info(f"Approved pairing for device {device.device_id}")
        return device
    
    def _cleanup_pairings(self) -> None:
        """Remove expired pairing requests."""
        expired = [
            code for code, p in self._pending_pairings.items()
            if p.is_expired
        ]
        for code in expired:
            del self._pending_pairings[code]
    
    def get_pending_pairings(self) -> list[PairingRequest]:
        """Get list of pending pairing requests."""
        self._cleanup_pairings()
        return [p for p in self._pending_pairings.values() if not p.approved]
    
    def get_registered_devices(self) -> list[RegisteredDevice]:
        """Get list of registered devices."""
        return list(self._registered_devices.values())
    
    def revoke_device(self, device_id: str) -> bool:
        """Revoke a device's access."""
        device = self._registered_devices.pop(device_id, None)
        if device:
            self._device_tokens.pop(device.token, None)
            self._save_devices()
            logger.info(f"Revoked device {device_id}")
            return True
        return False
    
    # === Import/Export ===
    
    def import_document(self, doc_path: Path) -> str:
        """
        Import a document directory into the sync store.
        
        Args:
            doc_path: Path to document directory
            
        Returns:
            Document ID
        """
        doc_path = Path(doc_path)
        doc_id = doc_path.name
        
        # Import all files
        for file_path in doc_path.rglob("*"):
            if file_path.is_file():
                rel_path = file_path.relative_to(doc_path)
                filename = f"{doc_id}/{rel_path}" if "/" in str(rel_path) else str(rel_path)
                
                data = file_path.read_bytes()
                self._store_file(data, doc_id, filename)
        
        # Update root
        self._state.compute_root()
        self._save_state()
        
        logger.info(f"Imported document {doc_id}")
        return doc_id
    
    def export_document(self, doc_id: str, output_dir: Path) -> Path:
        """
        Export a document from the sync store.
        
        Args:
            doc_id: Document ID to export
            output_dir: Output directory
            
        Returns:
            Path to exported document directory
        """
        output_dir = Path(output_dir)
        doc_dir = output_dir / doc_id
        doc_dir.mkdir(parents=True, exist_ok=True)
        
        for entry in self._state.files.values():
            if entry.document_id != doc_id:
                continue
            
            data = self._load_file(entry.hash)
            if data is None:
                logger.warning(f"Missing file: {entry.hash}")
                continue
            
            # Determine output path
            if "/" in entry.filename:
                parts = entry.filename.split("/", 1)
                out_path = doc_dir / parts[1]
            else:
                out_path = doc_dir / entry.filename
            
            out_path.parent.mkdir(parents=True, exist_ok=True)
            out_path.write_bytes(data)
        
        return doc_dir
    
    # === HTTP Server ===
    
    async def start(
        self,
        host: str = "127.0.0.1",
        port: int = 8080,
        *,
        enable_mdns: bool = True,
    ) -> None:
        """
        Start the HTTP server.
        
        Args:
            host: Bind address
            port: Port number
            enable_mdns: Announce via mDNS/Bonjour
        """
        try:
            from aiohttp import web
        except ImportError:
            raise ImportError(
                "aiohttp required for server. Install with: pip install aiohttp"
            )
        
        app = web.Application()
        
        # Health and info
        app.router.add_get("/health", self._handle_health)

        # Pairing endpoints
        app.router.add_post("/pair/code", self._handle_pair_code)
        app.router.add_get("/pair/status/{code}", self._handle_pair_status)
        app.router.add_post("/pair/complete", self._handle_pair_complete)
        app.router.add_get("/devices", self._handle_list_devices)
        app.router.add_delete("/devices/{device_id}", self._handle_delete_device)

        app.router.add_get("/info", self._handle_info)
        
        # Pairing endpoints
        app.router.add_post("/pair/request", self._handle_pair_request)
        app.router.add_post("/pair/complete", self._handle_pair_complete)
        app.router.add_post("/pair/exchange", self._handle_pair_exchange)
        app.router.add_get("/pair/pending", self._handle_pair_pending)
        app.router.add_post("/pair/approve", self._handle_pair_approve)
        app.router.add_get("/devices", self._handle_devices_list)
        app.router.add_delete("/devices/{device_id}", self._handle_device_revoke)
        
        # Sync endpoints
        app.router.add_get("/sync/v3/root", self._handle_root)
        app.router.add_get("/sync/v3/files/{hash}", self._handle_get_file)
        app.router.add_put("/sync/v3/files/{hash}", self._handle_put_file)
        
        self._app = app
        
        runner = web.AppRunner(app)
        await runner.setup()
        
        site = web.TCPSite(runner, host, port)
        await site.start()
        
        logger.info(f"Local sync server started at http://{host}:{port}")
        logger.info(f"Server name: {self.server_name}")
        if self.require_auth:
            logger.info(f"Auth token: {self.auth_token}")
        else:
            logger.info("Authentication disabled")
        
        # Start mDNS announcement
        if enable_mdns:
            asyncio.create_task(self._announce_mdns(host, port))
    
    async def _announce_mdns(self, host: str, port: int) -> None:
        """Announce server via mDNS/Bonjour."""
        try:
            from zeroconf import ServiceInfo
            from zeroconf.asyncio import AsyncZeroconf
        except ImportError:
            logger.debug("zeroconf not installed, skipping mDNS announcement")
            return
        
        import socket
        
        try:
            # Get local IP
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            s.connect(("8.8.8.8", 80))
            local_ip = s.getsockname()[0]
            s.close()
        except Exception:
            local_ip = "127.0.0.1"
        
        service_type = "_remarkable-sync._tcp.local."
        service_name = f"{self.server_name}._remarkable-sync._tcp.local."
        
        info = ServiceInfo(
            service_type,
            service_name,
            addresses=[socket.inet_aton(local_ip)],
            port=port,
            properties={
                b"name": self.server_name.encode(),
                b"version": SERVER_VERSION.encode(),
                b"secure": b"false",
            },
        )
        
        azc = AsyncZeroconf()
        await azc.async_register_service(info)
        logger.info(f"mDNS: Announcing as {service_name}")
    
    def _check_auth(self, request) -> str | None:
        """
        Check request authentication.
        
        Returns device_id if authenticated, None if not.
        """
        auth_header = request.headers.get("Authorization", "")
        if not auth_header.startswith("Bearer "):
            return None if self.require_auth else "__anonymous__"
        
        token = auth_header[7:]
        
        # Check master token
        if token == self.auth_token:
            return "__master__"
        
        # Check device token
        device_id = self._device_tokens.get(token)
        if device_id:
            # Update last seen
            device = self._registered_devices.get(device_id)
            if device:
                device.last_seen = datetime.now()
            return device_id
        
        return None if self.require_auth else "__anonymous__"
    
    async def _handle_health(self, request) -> Any:
        from aiohttp import web
        return web.json_response({
            "status": "ok",
            "name": self.server_name,
            "version": SERVER_VERSION,
        })
    
    async def _handle_info(self, request) -> Any:
        from aiohttp import web
        return web.json_response({
            "name": self.server_name,
            "version": SERVER_VERSION,
            "documents": len(self._state.documents),
            "files": len(self._state.files),
            "generation": self._state.generation,
            "require_auth": self.require_auth,
            "auto_approve": self.auto_approve,
            "registered_devices": len(self._registered_devices),
        })
    
    # === Pairing Handlers ===
    
    async def _handle_pair_request(self, request) -> Any:
        """Handle pairing request from client."""
        from aiohttp import web
        
        try:
            data = await request.json()
        except Exception:
            return web.Response(status=400, text="Invalid JSON")
        
        device_id = data.get("deviceId", str(uuid4()))
        device_type = data.get("deviceType", "desktop-linux")
        device_desc = data.get("deviceDesc", "Unknown Device")
        
        code = self.create_pairing_code(device_id, device_type, device_desc)
        
        # Auto-approve if enabled
        if self.auto_approve:
            self.approve_pairing(code)
        
        return web.json_response({
            "code": code,
            "expires_in": PAIRING_CODE_EXPIRY_SECONDS,
            "auto_approved": self.auto_approve,
        })
    
    async def _handle_pair_complete(self, request) -> Any:
        """Handle pairing completion check from client."""
        from aiohttp import web
        
        try:
            data = await request.json()
        except Exception:
            return web.Response(status=400, text="Invalid JSON")
        
        code = data.get("code", "").lower()
        device_id = data.get("deviceId", "")
        
        pairing = self._pending_pairings.get(code)
        if pairing is None:
            return web.Response(status=404, text="Invalid pairing code")
        
        if pairing.is_expired:
            del self._pending_pairings[code]
            return web.Response(status=410, text="Pairing code expired")
        
        if pairing.device_id != device_id:
            return web.Response(status=403, text="Device ID mismatch")
        
        if pairing.approved:
            # Return tokens
            return web.json_response({
                "deviceToken": pairing.token,
                "userToken": pairing.token,  # Same token for local server
            })
        
        # Still pending
        return web.Response(status=202, text="Pairing pending approval")
    
    async def _handle_pair_exchange(self, request) -> Any:
        """Handle one-shot code exchange (server displays code, client enters it)."""
        from aiohttp import web
        
        try:
            data = await request.json()
        except Exception:
            return web.Response(status=400, text="Invalid JSON")
        
        code = data.get("code", "").lower()
        device_id = data.get("deviceId", str(uuid4()))
        device_type = data.get("deviceType", "desktop-linux")
        device_desc = data.get("deviceDesc", "Unknown Device")
        
        pairing = self._pending_pairings.get(code)
        if pairing is None:
            return web.Response(status=404, text="Invalid pairing code")
        
        if pairing.is_expired:
            del self._pending_pairings[code]
            return web.Response(status=410, text="Pairing code expired")
        
        # Update pairing with device info and approve
        pairing.device_id = device_id
        pairing.device_type = device_type
        pairing.device_desc = device_desc
        
        device = self.approve_pairing(code)
        if device is None:
            return web.Response(status=500, text="Failed to approve pairing")
        
        return web.json_response({
            "deviceToken": device.token,
            "userToken": device.token,
        })
    
    async def _handle_pair_pending(self, request) -> Any:
        """List pending pairing requests (for server UI)."""
        from aiohttp import web
        
        device_id = self._check_auth(request)
        if device_id not in ("__master__", "__anonymous__"):
            return web.Response(status=401, text="Unauthorized")
        
        pending = self.get_pending_pairings()
        return web.json_response({
            "pending": [
                {
                    "code": p.code,
                    "device_id": p.device_id,
                    "device_type": p.device_type,
                    "device_desc": p.device_desc,
                    "created_at": p.created_at.isoformat(),
                    "expires_in": max(0, int((p.expires_at - datetime.now()).total_seconds())),
                }
                for p in pending
            ],
        })
    
    async def _handle_pair_approve(self, request) -> Any:
        """Approve a pending pairing (for server UI)."""
        from aiohttp import web
        
        device_id = self._check_auth(request)
        if device_id not in ("__master__", "__anonymous__"):
            return web.Response(status=401, text="Unauthorized")
        
        try:
            data = await request.json()
        except Exception:
            return web.Response(status=400, text="Invalid JSON")
        
        code = data.get("code", "").lower()
        
        device = self.approve_pairing(code)
        if device is None:
            return web.Response(status=404, text="Invalid or expired pairing code")
        
        return web.json_response({
            "device_id": device.device_id,
            "device_desc": device.device_desc,
            "token": device.token,
        })
    
    async def _handle_devices_list(self, request) -> Any:
        """List registered devices."""
        from aiohttp import web
        
        device_id = self._check_auth(request)
        if device_id not in ("__master__", "__anonymous__"):
            return web.Response(status=401, text="Unauthorized")
        
        devices = self.get_registered_devices()
        return web.json_response({
            "devices": [d.to_dict() for d in devices],
        })
    
    async def _handle_device_revoke(self, request) -> Any:
        """Revoke a device's access."""
        from aiohttp import web
        
        auth_device = self._check_auth(request)
        if auth_device not in ("__master__", "__anonymous__"):
            return web.Response(status=401, text="Unauthorized")
        
        device_id = request.match_info["device_id"]
        
        if self.revoke_device(device_id):
            return web.Response(status=200, text="Device revoked")
        return web.Response(status=404, text="Device not found")
    
    # === Sync Handlers ===
    
    async def _handle_root(self, request) -> Any:
        from aiohttp import web
        
        if self._check_auth(request) is None:
            return web.Response(status=401, text="Unauthorized")
        
        return web.json_response({
            "hash": self._state.root_hash,
            "generation": self._state.generation,
            "schemaVersion": 3,
        })
    
    async def _handle_get_file(self, request) -> Any:
        from aiohttp import web
        
        if self._check_auth(request) is None:
            return web.Response(status=401, text="Unauthorized")
        
        file_hash = request.match_info["hash"]
        rm_filename = request.headers.get("rm-filename", "")
        
        if not rm_filename:
            return web.Response(
                status=400, 
                text="unexpected 'rm-filename' http header"
            )
        
        # Handle root index specially
        if rm_filename == "root.docSchema":
            root_data = json.dumps(self._state.to_root_index()).encode()
            return web.Response(body=root_data, content_type="application/json")
        
        # Get file from storage
        data = self._load_file(file_hash)
        if data is None:
            return web.Response(status=404, text="File not found")
        
        return web.Response(body=data, content_type="application/octet-stream")
    
    async def _handle_put_file(self, request) -> Any:
        from aiohttp import web
        
        if self._check_auth(request) is None:
            return web.Response(status=401, text="Unauthorized")
        
        file_hash = request.match_info["hash"]
        rm_filename = request.headers.get("rm-filename", "")
        
        if not rm_filename:
            return web.Response(
                status=400,
                text="unexpected 'rm-filename' http header"
            )
        
        # Get document ID from header or filename
        rm_parent = request.headers.get("rm-parent-hash", "")
        
        # Parse document ID from filename (e.g., "doc-id.docSchema" or "doc-id/page.rm")
        if "/" in rm_filename:
            doc_id = rm_filename.split("/")[0]
        else:
            doc_id = rm_filename.rsplit(".", 1)[0]
        
        # Read and store file
        data = await request.read()
        
        # Verify hash
        actual_hash = hashlib.sha256(data).hexdigest()
        if actual_hash != file_hash:
            return web.Response(status=400, text="Hash mismatch")
        
        self._store_file(data, doc_id, rm_filename)
        
        # Update root
        self._state.compute_root()
        self._save_state()
        
        return web.Response(status=200)


# === Server-side Pairing Display ===

def generate_server_pairing_code(server: LocalSyncServer) -> str:
    """
    Generate a pairing code for display on the server.
    
    The code can be entered on a client to pair.
    
    Args:
        server: LocalSyncServer instance
        
    Returns:
        8-character pairing code
    """
    code = generate_pairing_code()
    now = datetime.now()
    
    # Create a placeholder pairing request
    server._pending_pairings[code] = PairingRequest(
        code=code,
        device_id="__pending__",  # Will be set when client connects
        device_type="",
        device_desc="",
        created_at=now,
        expires_at=now + timedelta(seconds=PAIRING_CODE_EXPIRY_SECONDS),
    )
    
    return code



    async def _handle_pair_code(self, request) -> Any:
        """Generate new pairing code."""
        from aiohttp import web
        code = self._pairing.generate_code()
        return web.json_response({"code": code})
    
    async def _handle_pair_status(self, request) -> Any:
        """Check pairing code status."""
        from aiohttp import web
        code = request.match_info["code"]
        status = self._pairing.get_code_status(code)
        return web.json_response(status)
    
    async def _handle_pair_complete(self, request) -> Any:
        """Complete pairing with device info."""
        from aiohttp import web
        try:
            data = await request.json()
            code = data.get("code", "")
            device_info = data.get("device_info", {})
            
            result = self._pairing.complete_pairing(code, device_info)
            return web.json_response(result)
        except ValueError as e:
            return web.json_response({"error": str(e)}, status=400)
    
    async def _handle_list_devices(self, request) -> Any:
        """List registered devices."""
        from aiohttp import web
        devices = list(self._pairing.registered_devices.values())
        return web.json_response({"devices": devices})
    
    async def _handle_delete_device(self, request) -> Any:
        """Delete a registered device."""
        from aiohttp import web
        device_id = request.match_info["device_id"]
        if device_id in self._pairing.registered_devices:
            del self._pairing.registered_devices[device_id]
            return web.json_response({"status": "deleted"})
        return web.json_response({"error": "not found"}, status=404)

# CLI entry point for running server standalone
def run_server(
    storage_dir: str = "./remarkable_sync",
    host: str = "127.0.0.1",
    port: int = 8080,
    require_auth: bool = False,
    auto_approve: bool = False,
    server_name: str = "reMarkable Local Server",
) -> None:
    """Run the local sync server."""
    import asyncio
    
    server = LocalSyncServer(
        storage_dir=storage_dir,
        require_auth=require_auth,
        auto_approve=auto_approve,
        server_name=server_name,
    )
    
    async def main():
        await server.start(host=host, port=port)
        
        # Display pairing instructions
        print(f"\n{'='*50}")
        print("Server is ready for pairing!")
        print(f"{'='*50}")
        print(f"\nTo pair a device, run:")
        print(f"  remarkable pair --local http://{host}:{port}")
        print(f"\nOr from Python:")
        print(f"  from remarkable.local import LocalServerClient")
        print(f"  client = LocalServerClient('http://{host}:{port}')")
        print(f"  await client.pair_with_code('<code>')")
        print(f"\n{'='*50}\n")
        
        # Keep running
        while True:
            await asyncio.sleep(3600)
    
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("Server stopped")


if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="reMarkable local sync server")
    parser.add_argument("--storage", default="./remarkable_sync", help="Storage directory")
    parser.add_argument("--host", default="127.0.0.1", help="Bind address")
    parser.add_argument("--port", type=int, default=8080, help="Port number")
    parser.add_argument("--auth", action="store_true", help="Require authentication")
    parser.add_argument("--auto-approve", action="store_true", help="Auto-approve pairing requests")
    parser.add_argument("--name", default="reMarkable Local Server", help="Server name")
    
    args = parser.parse_args()
    
    logging.basicConfig(level=logging.INFO)
    run_server(
        storage_dir=args.storage,
        host=args.host,
        port=args.port,
        require_auth=args.auth,
        auto_approve=args.auto_approve,
        server_name=args.name,
    )
