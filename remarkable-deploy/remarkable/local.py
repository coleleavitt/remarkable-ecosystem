"""
Local server client for reMarkable.

Handles connection to local sync servers, pairing, and auto-discovery.
"""

from __future__ import annotations

import asyncio
import json
import logging
import socket
import ssl
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any
from uuid import uuid4

import httpx

from remarkable.models import DeviceInfo, Document, SyncRoot, TokenPair

logger = logging.getLogger(__name__)

# Default ports
DEFAULT_HTTP_PORT = 8080
DEFAULT_HTTPS_PORT = 8443
MDNS_SERVICE = "_remarkable-sync._tcp.local."


@dataclass
class LocalServerInfo:
    """Information about a discovered local server."""
    host: str
    port: int
    name: str = ""
    version: str = ""
    secure: bool = False
    
    @property
    def url(self) -> str:
        protocol = "https" if self.secure else "http"
        return f"{protocol}://{self.host}:{self.port}"
    
    def __str__(self) -> str:
        name = f" ({self.name})" if self.name else ""
        return f"{self.url}{name}"


@dataclass
class PairingState:
    """State of a pairing request."""
    code: str
    device_id: str
    created_at: datetime
    expires_at: datetime
    completed: bool = False
    token: str | None = None
    
    @property
    def is_expired(self) -> bool:
        return datetime.now() > self.expires_at


class LocalServerClient:
    """
    Client for connecting to a local reMarkable sync server.
    
    Handles:
    - Connection to local servers (HTTP/HTTPS)
    - Self-signed certificate handling
    - Token-based authentication
    - Pairing flow
    
    Example:
        ```python
        async with LocalServerClient("http://192.168.1.100:8080") as client:
            # Pair with server
            code = await client.request_pairing_code()
            print(f"Enter code on server: {code}")
            
            # Wait for pairing completion
            tokens = await client.complete_pairing(code)
            
            # Use synced connection
            docs = await client.list_documents()
        ```
    """
    
    def __init__(
        self,
        server_url: str,
        *,
        device_id: str | None = None,
        device_name: str = "remarkable-sdk",
        verify_ssl: bool = False,
        timeout: float = 30.0,
    ) -> None:
        """
        Initialize the local server client.
        
        Args:
            server_url: URL of the local server (e.g., "http://192.168.1.100:8080")
            device_id: Unique device identifier (generated if not provided)
            device_name: Human-readable device name for pairing
            verify_ssl: Whether to verify SSL certificates (False for self-signed)
            timeout: HTTP request timeout in seconds
        """
        self.server_url = server_url.rstrip("/")
        self.verify_ssl = verify_ssl
        self.timeout = timeout
        
        self._device_info = DeviceInfo(
            device_id=device_id or self._generate_device_id(),
            device_type="desktop-linux",  # Local client type
            device_desc=device_name,
        )
        
        # Create HTTP client with SSL settings
        self._http: httpx.AsyncClient | None = None
        self._tokens: TokenPair | None = None
    
    @staticmethod
    def _generate_device_id() -> str:
        """Generate a random device ID."""
        return f"RM-LOCAL-{uuid4().hex[:8].upper()}"
    
    def _create_http_client(self) -> httpx.AsyncClient:
        """Create HTTP client with appropriate SSL settings."""
        if self.verify_ssl:
            return httpx.AsyncClient(timeout=self.timeout)
        
        # Create SSL context that accepts self-signed certs
        ssl_context = ssl.create_default_context()
        ssl_context.check_hostname = False
        ssl_context.verify_mode = ssl.CERT_NONE
        
        return httpx.AsyncClient(
            timeout=self.timeout,
            verify=False,  # For self-signed certs
        )
    
    @property
    def http(self) -> httpx.AsyncClient:
        """Get or create HTTP client."""
        if self._http is None:
            self._http = self._create_http_client()
        return self._http
    
    @property
    def device_info(self) -> DeviceInfo:
        """Device info for this client."""
        return self._device_info
    
    @property
    def tokens(self) -> TokenPair | None:
        """Current authentication tokens."""
        return self._tokens
    
    @property
    def is_authenticated(self) -> bool:
        """Whether client has valid tokens."""
        return self._tokens is not None and not self._tokens.is_expired
    
    async def __aenter__(self) -> LocalServerClient:
        return self
    
    async def __aexit__(self, *args: Any) -> None:
        await self.close()
    
    async def close(self) -> None:
        """Close HTTP connections."""
        if self._http is not None:
            await self._http.aclose()
            self._http = None
    
    # === Server Discovery ===
    
    async def verify_connection(self) -> bool:
        """
        Verify connection to the local server.
        
        Returns:
            True if server is reachable and responding
        """
        try:
            resp = await self.http.get(
                f"{self.server_url}/health",
                timeout=5.0,
            )
            if resp.status_code == 200:
                data = resp.json()
                logger.info(f"Server OK: {data}")
                return True
        except httpx.ConnectError as e:
            logger.debug(f"Connection failed: {e}")
        except Exception as e:
            logger.debug(f"Health check failed: {e}")
        return False
    
    async def get_server_info(self) -> dict[str, Any]:
        """
        Get server information and capabilities.
        
        Returns:
            Server info dict with version, capabilities, etc.
        """
        resp = await self.http.get(f"{self.server_url}/info")
        resp.raise_for_status()
        return resp.json()
    
    # === Pairing Flow ===
    
    async def request_pairing_code(self) -> str:
        """
        Request a pairing code from the local server.
        
        The server will display or return a code that must be confirmed.
        
        Returns:
            8-character pairing code
        """
        resp = await self.http.post(
            f"{self.server_url}/pair/request",
            json={
                "deviceId": self._device_info.device_id,
                "deviceType": self._device_info.device_type,
                "deviceDesc": self._device_info.device_desc,
            },
        )
        resp.raise_for_status()
        data = resp.json()
        return data["code"]
    
    async def complete_pairing(
        self,
        code: str,
        *,
        poll_interval: float = 1.0,
        timeout: float = 120.0,
    ) -> TokenPair:
        """
        Complete pairing with the server using a code.
        
        This polls the server until pairing is confirmed or times out.
        
        Args:
            code: Pairing code (from request_pairing_code or server display)
            poll_interval: Seconds between poll attempts
            timeout: Maximum seconds to wait for pairing
            
        Returns:
            TokenPair with authentication tokens
        """
        code = code.lower().strip()
        deadline = datetime.now() + timedelta(seconds=timeout)
        
        while datetime.now() < deadline:
            try:
                resp = await self.http.post(
                    f"{self.server_url}/pair/complete",
                    json={
                        "code": code,
                        "deviceId": self._device_info.device_id,
                    },
                )
                
                if resp.status_code == 200:
                    data = resp.json()
                    
                    # Build token pair
                    self._tokens = TokenPair(
                        device_token=data.get("deviceToken", data.get("token", "")),
                        user_token=data.get("userToken", data.get("token", "")),
                        expires_at=None,  # Local tokens don't expire
                        scopes=["sync:local"],
                        region="local",
                    )
                    
                    logger.info(f"Pairing complete for device {self._device_info.device_id}")
                    return self._tokens
                
                elif resp.status_code == 202:
                    # Pending - code accepted, waiting for server approval
                    logger.debug("Pairing pending, waiting for server approval...")
                    
                elif resp.status_code == 404:
                    raise ValueError(f"Invalid pairing code: {code}")
                    
                elif resp.status_code == 410:
                    raise ValueError(f"Pairing code expired: {code}")
                    
            except httpx.ConnectError:
                logger.debug("Server not reachable, retrying...")
            
            await asyncio.sleep(poll_interval)
        
        raise TimeoutError(f"Pairing timed out after {timeout}s")
    
    async def pair_with_code(self, code: str) -> TokenPair:
        """
        One-shot pairing with a code displayed by the server.
        
        Use this when the server displays a code and client enters it.
        
        Args:
            code: Code displayed by the server
            
        Returns:
            TokenPair with authentication tokens
        """
        code = code.lower().strip()
        
        resp = await self.http.post(
            f"{self.server_url}/pair/exchange",
            json={
                "code": code,
                "deviceId": self._device_info.device_id,
                "deviceType": self._device_info.device_type,
                "deviceDesc": self._device_info.device_desc,
            },
        )
        
        if resp.status_code == 404:
            raise ValueError(f"Invalid pairing code: {code}")
        if resp.status_code == 410:
            raise ValueError(f"Pairing code expired: {code}")
        
        resp.raise_for_status()
        data = resp.json()
        
        self._tokens = TokenPair(
            device_token=data.get("deviceToken", data.get("token", "")),
            user_token=data.get("userToken", data.get("token", "")),
            expires_at=None,
            scopes=["sync:local"],
            region="local",
        )
        
        logger.info(f"Paired with server using code {code}")
        return self._tokens
    
    # === Token Management ===
    
    async def save_tokens(self, path: Path | str) -> None:
        """Save tokens to file."""
        if self._tokens is None:
            raise ValueError("No tokens to save")
        
        path = Path(path)
        data = {
            "server_url": self.server_url,
            "device_id": self._device_info.device_id,
            "device_token": self._tokens.device_token,
            "user_token": self._tokens.user_token,
            "region": self._tokens.region,
            "scopes": self._tokens.scopes,
        }
        path.write_text(json.dumps(data, indent=2))
        logger.info(f"Tokens saved to {path}")
    
    async def load_tokens(self, path: Path | str) -> TokenPair:
        """Load tokens from file."""
        path = Path(path)
        if not path.exists():
            raise FileNotFoundError(f"Token file not found: {path}")
        
        data = json.loads(path.read_text())
        
        # Update server URL if stored
        if "server_url" in data:
            self.server_url = data["server_url"]
        
        # Update device ID if stored
        if "device_id" in data:
            self._device_info = DeviceInfo(
                device_id=data["device_id"],
                device_type=self._device_info.device_type,
                device_desc=self._device_info.device_desc,
            )
        
        self._tokens = TokenPair(
            device_token=data["device_token"],
            user_token=data["user_token"],
            expires_at=None,
            scopes=data.get("scopes", ["sync:local"]),
            region=data.get("region", "local"),
        )
        
        return self._tokens
    
    # === Sync Operations ===
    
    def _auth_headers(self) -> dict[str, str]:
        """Get authorization headers."""
        if self._tokens is None:
            return {}
        return {"Authorization": f"Bearer {self._tokens.device_token}"}
    
    async def get_sync_root(self) -> SyncRoot:
        """Get the current sync root hash."""
        resp = await self.http.get(
            f"{self.server_url}/sync/v3/root",
            headers=self._auth_headers(),
        )
        resp.raise_for_status()
        return SyncRoot.from_response(resp.json())
    
    async def get_file(self, file_hash: str, filename: str) -> bytes:
        """
        Download a file by hash.
        
        Args:
            file_hash: SHA256 hash of the file
            filename: Filename for rm-filename header
            
        Returns:
            File content bytes
        """
        headers = self._auth_headers()
        headers["rm-filename"] = filename
        
        resp = await self.http.get(
            f"{self.server_url}/sync/v3/files/{file_hash}",
            headers=headers,
        )
        resp.raise_for_status()
        return resp.content
    
    async def put_file(
        self,
        data: bytes,
        file_hash: str,
        filename: str,
        parent_hash: str = "",
    ) -> None:
        """
        Upload a file.
        
        Args:
            data: File content
            file_hash: SHA256 hash of the content
            filename: Filename for rm-filename header
            parent_hash: Parent hash for rm-parent-hash header
        """
        headers = self._auth_headers()
        headers["rm-filename"] = filename
        if parent_hash:
            headers["rm-parent-hash"] = parent_hash
        
        resp = await self.http.put(
            f"{self.server_url}/sync/v3/files/{file_hash}",
            headers=headers,
            content=data,
        )
        resp.raise_for_status()
    
    async def list_documents(self) -> list[Document]:
        """
        List all documents from the local server.
        
        Returns:
            List of Document objects
        """
        # Get root schema
        root = await self.get_sync_root()
        
        if not root.hash:
            return []
        
        # Get root index
        root_data = await self.get_file(root.hash, "root.docSchema")
        root_index = json.loads(root_data)
        
        documents = []
        
        for file_entry in root_index.get("files", []):
            if not file_entry["filename"].endswith(".docSchema"):
                continue
            
            # Parse document schema
            try:
                schema_data = await self.get_file(
                    file_entry["hash"],
                    file_entry["filename"],
                )
                schema = json.loads(schema_data)
                
                from uuid import UUID
                
                doc = Document(
                    id=UUID(file_entry["documentId"]),
                    name=schema.get("visibleName", "Unknown"),
                    file_type=schema.get("fileType", ""),
                    pages=schema.get("pages", []),
                )
                documents.append(doc)
            except Exception as e:
                logger.warning(f"Failed to parse document {file_entry['documentId']}: {e}")
        
        return documents


# === mDNS/Bonjour Discovery ===

async def discover_local_servers(
    timeout: float = 5.0,
) -> list[LocalServerInfo]:
    """
    Discover local reMarkable sync servers on the network.
    
    Uses mDNS/Bonjour to find servers advertising the
    _remarkable-sync._tcp.local. service.
    
    Args:
        timeout: Discovery timeout in seconds
        
    Returns:
        List of discovered server info
    """
    try:
        from zeroconf import ServiceBrowser, Zeroconf
        from zeroconf.asyncio import AsyncZeroconf
    except ImportError:
        logger.warning(
            "zeroconf not installed. Install with: pip install zeroconf"
        )
        return []
    
    servers: list[LocalServerInfo] = []
    
    class Listener:
        def add_service(self, zc: Zeroconf, type_: str, name: str) -> None:
            info = zc.get_service_info(type_, name)
            if info:
                addresses = info.parsed_addresses()
                if addresses:
                    host = addresses[0]
                    port = info.port
                    
                    # Check properties for additional info
                    props = info.properties
                    server_name = props.get(b"name", b"").decode() or name
                    version = props.get(b"version", b"").decode()
                    secure = props.get(b"secure", b"false").decode().lower() == "true"
                    
                    servers.append(LocalServerInfo(
                        host=host,
                        port=port,
                        name=server_name,
                        version=version,
                        secure=secure,
                    ))
                    logger.info(f"Discovered server: {host}:{port} ({server_name})")
        
        def remove_service(self, zc: Zeroconf, type_: str, name: str) -> None:
            pass
        
        def update_service(self, zc: Zeroconf, type_: str, name: str) -> None:
            pass
    
    azc = AsyncZeroconf()
    browser = ServiceBrowser(
        azc.zeroconf,
        MDNS_SERVICE,
        Listener(),
    )
    
    # Wait for discovery
    await asyncio.sleep(timeout)
    
    await azc.async_close()
    
    return servers


async def scan_local_network(
    ports: list[int] | None = None,
    timeout: float = 2.0,
) -> list[LocalServerInfo]:
    """
    Scan local network for reMarkable sync servers.
    
    Falls back to port scanning when mDNS is unavailable.
    
    Args:
        ports: Ports to scan (default: 8080, 8443)
        timeout: Timeout per host in seconds
        
    Returns:
        List of discovered servers
    """
    if ports is None:
        ports = [DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT]
    
    # Get local network range
    try:
        # Get local IP
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        local_ip = s.getsockname()[0]
        s.close()
    except Exception:
        local_ip = "192.168.1.1"
    
    # Scan /24 network
    network_prefix = ".".join(local_ip.split(".")[:-1])
    
    servers: list[LocalServerInfo] = []
    
    async def check_host(host: str, port: int) -> LocalServerInfo | None:
        url = f"http://{host}:{port}"
        try:
            async with httpx.AsyncClient(timeout=timeout, verify=False) as client:
                resp = await client.get(f"{url}/health")
                if resp.status_code == 200:
                    logger.info(f"Found server at {url}")
                    return LocalServerInfo(
                        host=host,
                        port=port,
                        name="",
                        secure=False,
                    )
        except Exception:
            pass
        return None
    
    # Check common ports on local network
    tasks = []
    for i in range(1, 255):
        host = f"{network_prefix}.{i}"
        for port in ports:
            tasks.append(check_host(host, port))
    
    # Run with limited concurrency
    semaphore = asyncio.Semaphore(50)
    
    async def limited_check(coro):
        async with semaphore:
            return await coro
    
    results = await asyncio.gather(
        *[limited_check(t) for t in tasks],
        return_exceptions=True,
    )
    
    for result in results:
        if isinstance(result, LocalServerInfo):
            servers.append(result)
    
    return servers


async def find_local_server(
    timeout: float = 5.0,
    fallback_scan: bool = True,
) -> LocalServerInfo | None:
    """
    Find a local reMarkable sync server.
    
    Tries mDNS first, then falls back to network scanning.
    
    Args:
        timeout: Discovery timeout
        fallback_scan: Whether to scan network if mDNS fails
        
    Returns:
        First discovered server, or None
    """
    # Try mDNS first
    servers = await discover_local_servers(timeout=timeout)
    if servers:
        return servers[0]
    
    # Fallback to scanning
    if fallback_scan:
        servers = await scan_local_network(timeout=timeout / 2)
        if servers:
            return servers[0]
    
    return None


# === Convenience Functions ===

async def pair_with_local_server(
    server_url: str,
    code: str | None = None,
    *,
    device_name: str = "remarkable-sdk",
    token_path: Path | str | None = None,
) -> tuple[LocalServerClient, TokenPair]:
    """
    Pair with a local server and optionally save tokens.
    
    Args:
        server_url: URL of the local server
        code: Pairing code (if None, requests one from server)
        device_name: Name for this device
        token_path: Path to save tokens (optional)
        
    Returns:
        Tuple of (client, tokens)
    """
    client = LocalServerClient(
        server_url,
        device_name=device_name,
    )
    
    if code:
        # Use provided code
        tokens = await client.pair_with_code(code)
    else:
        # Request code from server
        code = await client.request_pairing_code()
        print(f"Pairing code: {code}")
        print("Approve this code on the server to complete pairing...")
        tokens = await client.complete_pairing(code)
    
    if token_path:
        await client.save_tokens(token_path)
    
    return client, tokens
