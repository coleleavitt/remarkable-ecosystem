"""
Main reMarkable API client.

Handles authentication, token management, and provides access to sync,
document, and device operations.
"""

from __future__ import annotations

import json
import logging
from datetime import datetime, timedelta
from pathlib import Path
from typing import TYPE_CHECKING, Any

import httpx

from remarkable.models import (
    DeviceInfo,
    Document,
    DocumentSchema,
    SyncRoot,
    TokenPair,
)

if TYPE_CHECKING:
    from remarkable.sync import SyncClient

logger = logging.getLogger(__name__)

# reMarkable API endpoints
DISCOVERY_URL = "https://service-manager-production-dot-remarkable-production.appspot.com"
AUTH_URL = "https://webapp-prod.cloud.remarkable.engineering"
MY_REMARKABLE_URL = "https://my.remarkable.com"

# Auth0 client ID for my.remarkable.com
AUTH0_CLIENT_ID = "W88nD5PiTqa5X9BaB29rmille0W802fK"


class RemarkableClient:
    """
    Main client for interacting with reMarkable cloud and devices.
    
    Supports both cloud sync and local server modes.
    
    Example:
        ```python
        async with RemarkableClient() as client:
            # Authenticate with one-time code
            await client.authenticate_with_code("abcd1234")
            
            # Or load existing tokens
            await client.load_tokens("tokens.json")
            
            # List documents
            docs = await client.list_documents()
            for doc in docs:
                print(f"{doc.name} ({doc.page_count} pages)")
        ```
    """
    
    def __init__(
        self,
        *,
        device_id: str | None = None,
        device_type: str = "remarkable",
        device_desc: str = "remarkable-sdk",
        base_url: str | None = None,
        timeout: float = 30.0,
    ) -> None:
        """
        Initialize the client.
        
        Args:
            device_id: Unique device identifier (generated if not provided)
            device_type: Device type for registration ("remarkable", "desktop-linux", etc)
            device_desc: Device description for registration
            base_url: Override base URL for local server mode
            timeout: HTTP request timeout in seconds
        """
        self._http = httpx.AsyncClient(timeout=timeout)
        self._tokens: TokenPair | None = None
        self._device_info = DeviceInfo(
            device_id=device_id or self._generate_device_id(),
            device_type=device_type,
            device_desc=device_desc,
        )
        self._base_url = base_url
        self._endpoints: dict[str, str] = {}
        self._sync_client: SyncClient | None = None
    
    @staticmethod
    def _generate_device_id() -> str:
        """Generate a random device ID."""
        import uuid
        return f"RM110-SDK-{uuid.uuid4().hex[:8].upper()}"
    
    async def __aenter__(self) -> RemarkableClient:
        return self
    
    async def __aexit__(self, *args: Any) -> None:
        await self.close()
    
    async def close(self) -> None:
        """Close HTTP connections."""
        await self._http.aclose()
    
    # === Authentication ===
    
    async def discover_endpoints(self) -> dict[str, str]:
        """
        Discover API endpoints from the service manager.
        
        Returns:
            Dict mapping service names to URLs
        """
        if self._endpoints:
            return self._endpoints
        
        resp = await self._http.get(f"{DISCOVERY_URL}/discovery/v1/endpoints")
        resp.raise_for_status()
        self._endpoints = resp.json()
        return self._endpoints
    
    async def request_one_time_code(self, session_cookie: str) -> str:
        """
        Request a one-time pairing code using an authenticated browser session.
        
        Args:
            session_cookie: Session cookie from my.remarkable.com
            
        Returns:
            8-character one-time code
        """
        headers = {"Cookie": session_cookie}
        resp = await self._http.post(
            f"{MY_REMARKABLE_URL}/devices/v1",
            headers=headers,
            json={
                "deviceID": self._device_info.device_id,
                "deviceType": self._device_info.device_type,
                "deviceDesc": self._device_info.device_desc,
            },
        )
        resp.raise_for_status()
        data = resp.json()
        return data["onetimeCode"]
    
    async def authenticate_with_code(self, code: str) -> TokenPair:
        """
        Exchange a one-time code for device and user tokens.
        
        Args:
            code: 8-character one-time code from my.remarkable.com
            
        Returns:
            TokenPair with device_token and user_token
        """
        code = code.lower().strip()
        
        # Exchange code for device token
        resp = await self._http.post(
            f"{AUTH_URL}/token/json/2/device/new",
            json={
                "code": code,
                "deviceID": self._device_info.device_id,
                "deviceDesc": self._device_info.device_desc,
            },
        )
        resp.raise_for_status()
        device_token = resp.text.strip().strip('"')
        
        # Get user token
        user_token = await self._refresh_user_token(device_token)
        
        # Parse token claims
        claims = self._parse_jwt_claims(user_token)
        scopes = claims.get("scopes", "").split()
        region = claims.get("tectonic-shard", "eu")
        
        # UserToken expires in 3 hours
        expires_at = datetime.now() + timedelta(hours=3)
        
        self._tokens = TokenPair(
            device_token=device_token,
            user_token=user_token,
            expires_at=expires_at,
            scopes=scopes,
            region=region,
        )
        
        logger.info(
            f"Authenticated device {self._device_info.device_id} "
            f"with scopes: {scopes}, region: {region}"
        )
        
        return self._tokens
    
    async def _refresh_user_token(self, device_token: str) -> str:
        """Refresh the user token using device token."""
        resp = await self._http.post(
            f"{AUTH_URL}/token/json/2/user/new",
            headers={"Authorization": f"Bearer {device_token}"},
        )
        resp.raise_for_status()
        return resp.text.strip().strip('"')
    
    async def refresh_tokens(self) -> TokenPair:
        """
        Refresh the user token if expired or expiring soon.
        
        Returns:
            Updated TokenPair
        """
        if not self._tokens:
            raise ValueError("No tokens to refresh. Call authenticate_with_code first.")
        
        user_token = await self._refresh_user_token(self._tokens.device_token)
        claims = self._parse_jwt_claims(user_token)
        
        self._tokens = TokenPair(
            device_token=self._tokens.device_token,
            user_token=user_token,
            expires_at=datetime.now() + timedelta(hours=3),
            scopes=claims.get("scopes", "").split(),
            region=claims.get("tectonic-shard", self._tokens.region),
        )
        
        return self._tokens
    
    def _parse_jwt_claims(self, token: str) -> dict[str, Any]:
        """Parse JWT claims without validation (for extracting region/scopes)."""
        import base64
        
        parts = token.split(".")
        if len(parts) != 3:
            return {}
        
        # Decode payload (add padding)
        payload = parts[1]
        padding = 4 - len(payload) % 4
        if padding != 4:
            payload += "=" * padding
        
        try:
            decoded = base64.urlsafe_b64decode(payload)
            return json.loads(decoded)
        except Exception:
            return {}
    
    # === Token persistence ===
    
    async def save_tokens(self, path: str | Path) -> None:
        """Save tokens to a JSON file."""
        if not self._tokens:
            raise ValueError("No tokens to save")
        
        path = Path(path)
        data = {
            "device_id": self._device_info.device_id,
            "device_type": self._device_info.device_type,
            "device_token": self._tokens.device_token,
            "user_token": self._tokens.user_token,
            "expires_at": self._tokens.expires_at.isoformat() if self._tokens.expires_at else None,
            "scopes": self._tokens.scopes,
            "region": self._tokens.region,
        }
        path.write_text(json.dumps(data, indent=2))
        logger.info(f"Saved tokens to {path}")
    
    async def load_tokens(self, path: str | Path) -> TokenPair:
        """Load tokens from a JSON file."""
        path = Path(path)
        data = json.loads(path.read_text())
        
        self._device_info = DeviceInfo(
            device_id=data["device_id"],
            device_type=data.get("device_type", "remarkable"),
        )
        
        expires_at = None
        if data.get("expires_at"):
            expires_at = datetime.fromisoformat(data["expires_at"])
        
        self._tokens = TokenPair(
            device_token=data["device_token"],
            user_token=data["user_token"],
            expires_at=expires_at,
            scopes=data.get("scopes", []),
            region=data.get("region", "eu"),
        )
        
        # Refresh if expired
        if self._tokens.is_expired:
            logger.info("Token expired, refreshing...")
            await self.refresh_tokens()
        
        return self._tokens
    
    # === Properties ===
    
    @property
    def tokens(self) -> TokenPair | None:
        """Current authentication tokens."""
        return self._tokens
    
    @property
    def device_info(self) -> DeviceInfo:
        """Device information."""
        return self._device_info
    
    @property
    def is_authenticated(self) -> bool:
        """Check if client has valid tokens."""
        return self._tokens is not None and not self._tokens.is_expired
    
    @property
    def tectonic_url(self) -> str:
        """Get the tectonic sync URL for the current region."""
        if not self._tokens:
            return "https://eu.tectonic.remarkable.com"
        return f"https://{self._tokens.region}.tectonic.remarkable.com"
    
    # === Sync operations ===
    
    @property
    def sync(self) -> SyncClient:
        """Get the sync client."""
        if self._sync_client is None:
            from remarkable.sync import SyncClient
            self._sync_client = SyncClient(self)
        return self._sync_client
    
    async def get_sync_root(self) -> SyncRoot:
        """Get the current sync root hash and generation."""
        return await self.sync.get_root()
    
    async def list_documents(self) -> list[Document]:
        """
        List all documents in the library.
        
        Returns:
            List of Document objects with metadata
        """
        return await self.sync.list_documents()
    
    async def get_document(self, doc_id: str) -> DocumentSchema | None:
        """
        Get a specific document by ID.
        
        Args:
            doc_id: Document UUID
            
        Returns:
            DocumentSchema or None if not found
        """
        return await self.sync.get_document(doc_id)
    
    async def download_document(
        self, doc_id: str, output_dir: str | Path
    ) -> Path:
        """
        Download a document and all its pages.
        
        Args:
            doc_id: Document UUID
            output_dir: Directory to save files
            
        Returns:
            Path to the document directory
        """
        return await self.sync.download_document(doc_id, Path(output_dir))
    
    # === HTTP helpers ===
    
    async def _auth_headers(self) -> dict[str, str]:
        """Get authorization headers, refreshing token if needed."""
        if not self._tokens:
            raise ValueError("Not authenticated")
        
        if self._tokens.is_expired:
            await self.refresh_tokens()
        
        return {"Authorization": f"Bearer {self._tokens.user_token}"}
    
    async def _get(self, url: str, **kwargs: Any) -> httpx.Response:
        """Make an authenticated GET request."""
        headers = await self._auth_headers()
        headers.update(kwargs.pop("headers", {}))
        return await self._http.get(url, headers=headers, **kwargs)
    
    async def _post(self, url: str, **kwargs: Any) -> httpx.Response:
        """Make an authenticated POST request."""
        headers = await self._auth_headers()
        headers.update(kwargs.pop("headers", {}))
        return await self._http.post(url, headers=headers, **kwargs)
    
    async def _put(self, url: str, **kwargs: Any) -> httpx.Response:
        """Make an authenticated PUT request."""
        headers = await self._auth_headers()
        headers.update(kwargs.pop("headers", {}))
        return await self._http.put(url, headers=headers, **kwargs)
