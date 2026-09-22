"""
Device interaction for reMarkable tablets.

Supports USB and SSH connections for direct device access.
"""

from __future__ import annotations

import asyncio
import json
import logging
import os
from abc import ABC, abstractmethod
from dataclasses import dataclass
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)

# Default device paths
XOCHITL_CONFIG = "/home/root/.config/remarkable/xochitl.conf"
DOCUMENTS_PATH = "/home/root/.local/share/remarkable/xochitl"
TEMPLATES_PATH = "/usr/share/remarkable/templates"


@dataclass
class DeviceStatus:
    """Device status information."""
    model: str = ""
    serial: str = ""
    firmware_version: str = ""
    battery_level: int = 0
    charging: bool = False
    disk_free_bytes: int = 0
    disk_total_bytes: int = 0
    
    @property
    def disk_free_mb(self) -> float:
        return self.disk_free_bytes / (1024 * 1024)
    
    @property
    def disk_percent_used(self) -> float:
        if self.disk_total_bytes == 0:
            return 0.0
        return 100 * (1 - self.disk_free_bytes / self.disk_total_bytes)


class DeviceConnection(ABC):
    """Abstract base class for device connections."""
    
    @abstractmethod
    async def connect(self) -> None:
        """Establish connection to device."""
        pass
    
    @abstractmethod
    async def disconnect(self) -> None:
        """Close connection."""
        pass
    
    @abstractmethod
    async def read_file(self, path: str) -> bytes:
        """Read a file from the device."""
        pass
    
    @abstractmethod
    async def write_file(self, path: str, data: bytes) -> None:
        """Write a file to the device."""
        pass
    
    @abstractmethod
    async def list_dir(self, path: str) -> list[str]:
        """List directory contents."""
        pass
    
    @abstractmethod
    async def execute(self, command: str) -> tuple[str, str, int]:
        """Execute a command on the device."""
        pass
    
    async def __aenter__(self) -> DeviceConnection:
        await self.connect()
        return self
    
    async def __aexit__(self, *args: Any) -> None:
        await self.disconnect()
    
    # === High-level operations ===
    
    async def get_status(self) -> DeviceStatus:
        """Get device status information."""
        status = DeviceStatus()
        
        # Get model/serial from device info
        try:
            stdout, _, _ = await self.execute("cat /sys/devices/soc0/machine")
            status.model = stdout.strip()
        except Exception:
            pass
        
        try:
            stdout, _, _ = await self.execute("cat /sys/devices/soc0/serial_number")
            status.serial = stdout.strip()
        except Exception:
            pass
        
        # Get firmware version
        try:
            config = await self.read_config()
            # Version is typically in /etc/remarkable.conf or similar
            stdout, _, _ = await self.execute(
                "grep -r 'REMARKABLE_RELEASE_VERSION' /usr/share/remarkable/ 2>/dev/null | head -1"
            )
            if "=" in stdout:
                status.firmware_version = stdout.split("=")[-1].strip().strip('"')
        except Exception:
            pass
        
        # Get battery info
        try:
            stdout, _, _ = await self.execute(
                "cat /sys/class/power_supply/*/capacity 2>/dev/null | head -1"
            )
            status.battery_level = int(stdout.strip())
        except Exception:
            pass
        
        try:
            stdout, _, _ = await self.execute(
                "cat /sys/class/power_supply/*/status 2>/dev/null | head -1"
            )
            status.charging = "Charging" in stdout
        except Exception:
            pass
        
        # Get disk space
        try:
            stdout, _, _ = await self.execute("df -B1 /home | tail -1")
            parts = stdout.split()
            if len(parts) >= 4:
                status.disk_total_bytes = int(parts[1])
                status.disk_free_bytes = int(parts[3])
        except Exception:
            pass
        
        return status
    
    async def read_config(self) -> dict[str, str]:
        """Read xochitl configuration."""
        try:
            data = await self.read_file(XOCHITL_CONFIG)
            config = {}
            for line in data.decode().splitlines():
                line = line.strip()
                if "=" in line and not line.startswith("#"):
                    key, value = line.split("=", 1)
                    config[key.strip()] = value.strip()
            return config
        except Exception as e:
            logger.warning(f"Failed to read config: {e}")
            return {}
    
    async def write_config(self, config: dict[str, str]) -> None:
        """Write xochitl configuration."""
        lines = [f"{k}={v}" for k, v in config.items()]
        data = "\n".join(lines) + "\n"
        await self.write_file(XOCHITL_CONFIG, data.encode())
    
    async def update_device_token(self, token: str) -> None:
        """Update the device token in config."""
        config = await self.read_config()
        config["devicetoken"] = token
        await self.write_config(config)
        logger.info("Updated device token in xochitl.conf")
    
    async def restart_xochitl(self) -> None:
        """Restart the xochitl service."""
        await self.execute("systemctl restart xochitl")
        logger.info("Restarted xochitl service")
    
    async def list_documents(self) -> list[dict[str, Any]]:
        """List all documents on device."""
        documents = []
        
        files = await self.list_dir(DOCUMENTS_PATH)
        
        for filename in files:
            if filename.endswith(".metadata"):
                doc_id = filename.replace(".metadata", "")
                try:
                    data = await self.read_file(
                        f"{DOCUMENTS_PATH}/{filename}"
                    )
                    metadata = json.loads(data.decode())
                    metadata["id"] = doc_id
                    documents.append(metadata)
                except Exception as e:
                    logger.warning(f"Failed to read {filename}: {e}")
        
        return documents
    
    async def download_document(
        self, doc_id: str, output_dir: Path
    ) -> Path:
        """Download a document from device."""
        output_dir = Path(output_dir)
        doc_dir = output_dir / doc_id
        doc_dir.mkdir(parents=True, exist_ok=True)
        
        # Get list of files for this document
        files = await self.list_dir(DOCUMENTS_PATH)
        doc_files = [f for f in files if f.startswith(doc_id)]
        
        for filename in doc_files:
            src_path = f"{DOCUMENTS_PATH}/{filename}"
            
            # Handle subdirectories (pages)
            if "/" in filename:
                dest_path = doc_dir / Path(filename).relative_to(doc_id)
            else:
                dest_path = doc_dir / filename
            
            dest_path.parent.mkdir(parents=True, exist_ok=True)
            
            try:
                data = await self.read_file(src_path)
                dest_path.write_bytes(data)
            except Exception as e:
                logger.warning(f"Failed to download {filename}: {e}")
        
        # Also get content directory if it exists
        content_dir = f"{DOCUMENTS_PATH}/{doc_id}"
        try:
            content_files = await self.list_dir(content_dir)
            for filename in content_files:
                src_path = f"{content_dir}/{filename}"
                dest_path = doc_dir / filename
                
                try:
                    data = await self.read_file(src_path)
                    dest_path.write_bytes(data)
                except Exception as e:
                    logger.warning(f"Failed to download {filename}: {e}")
        except Exception:
            pass  # Directory may not exist
        
        return doc_dir
    
    async def upload_document(
        self, doc_dir: Path, doc_id: str | None = None
    ) -> str:
        """Upload a document to device."""
        doc_dir = Path(doc_dir)
        
        if doc_id is None:
            # Use directory name as doc ID
            doc_id = doc_dir.name
        
        dest_base = DOCUMENTS_PATH
        
        # Upload all files
        for path in doc_dir.rglob("*"):
            if path.is_file():
                rel_path = path.relative_to(doc_dir)
                
                if "/" in str(rel_path):
                    # File in subdirectory
                    dest_path = f"{dest_base}/{doc_id}/{rel_path}"
                else:
                    # Top-level file
                    dest_path = f"{dest_base}/{doc_id}.{rel_path.suffix[1:]}"
                
                data = path.read_bytes()
                await self.write_file(dest_path, data)
        
        return doc_id


class SSHConnection(DeviceConnection):
    """SSH connection to reMarkable device."""
    
    def __init__(
        self,
        host: str = "10.11.99.1",
        port: int = 22,
        username: str = "root",
        password: str | None = None,
        key_path: str | None = None,
    ) -> None:
        """
        Initialize SSH connection.
        
        Args:
            host: Device IP address (default: USB interface)
            port: SSH port
            username: SSH username (always root)
            password: SSH password (from device settings)
            key_path: Path to SSH private key
        """
        self.host = host
        self.port = port
        self.username = username
        self.password = password
        self.key_path = key_path
        self._client = None
        self._sftp = None
    
    async def connect(self) -> None:
        """Establish SSH connection."""
        import paramiko
        
        self._client = paramiko.SSHClient()
        self._client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        
        connect_kwargs: dict[str, Any] = {
            "hostname": self.host,
            "port": self.port,
            "username": self.username,
        }
        
        if self.key_path:
            connect_kwargs["key_filename"] = self.key_path
        elif self.password:
            connect_kwargs["password"] = self.password
        
        # Run in thread pool (paramiko is blocking)
        loop = asyncio.get_event_loop()
        await loop.run_in_executor(
            None, lambda: self._client.connect(**connect_kwargs)
        )
        
        self._sftp = await loop.run_in_executor(
            None, self._client.open_sftp
        )
        
        logger.info(f"Connected to {self.host}:{self.port}")
    
    async def disconnect(self) -> None:
        """Close SSH connection."""
        loop = asyncio.get_event_loop()
        
        if self._sftp:
            await loop.run_in_executor(None, self._sftp.close)
            self._sftp = None
        
        if self._client:
            await loop.run_in_executor(None, self._client.close)
            self._client = None
        
        logger.info("Disconnected")
    
    async def read_file(self, path: str) -> bytes:
        """Read a file via SFTP."""
        if not self._sftp:
            raise RuntimeError("Not connected")
        
        loop = asyncio.get_event_loop()
        
        def _read():
            with self._sftp.open(path, "rb") as f:
                return f.read()
        
        return await loop.run_in_executor(None, _read)
    
    async def write_file(self, path: str, data: bytes) -> None:
        """Write a file via SFTP."""
        if not self._sftp:
            raise RuntimeError("Not connected")
        
        loop = asyncio.get_event_loop()
        
        def _write():
            # Ensure parent directory exists
            parent = os.path.dirname(path)
            try:
                self._sftp.stat(parent)
            except FileNotFoundError:
                self._sftp.mkdir(parent)
            
            with self._sftp.open(path, "wb") as f:
                f.write(data)
        
        await loop.run_in_executor(None, _write)
    
    async def list_dir(self, path: str) -> list[str]:
        """List directory via SFTP."""
        if not self._sftp:
            raise RuntimeError("Not connected")
        
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self._sftp.listdir, path)
    
    async def execute(self, command: str) -> tuple[str, str, int]:
        """Execute command via SSH."""
        if not self._client:
            raise RuntimeError("Not connected")
        
        loop = asyncio.get_event_loop()
        
        def _exec():
            stdin, stdout, stderr = self._client.exec_command(command)
            exit_code = stdout.channel.recv_exit_status()
            return stdout.read().decode(), stderr.read().decode(), exit_code
        
        return await loop.run_in_executor(None, _exec)


class USBConnection(DeviceConnection):
    """
    USB connection to reMarkable device.
    
    Uses the device's built-in web interface for file access.
    """
    
    def __init__(
        self,
        host: str = "10.11.99.1",
        port: int = 80,
    ) -> None:
        """
        Initialize USB web connection.
        
        Args:
            host: Device IP address
            port: Web interface port
        """
        self.host = host
        self.port = port
        self.base_url = f"http://{host}:{port}"
        self._http = None
    
    async def connect(self) -> None:
        """Check USB web interface is available."""
        import httpx
        
        self._http = httpx.AsyncClient(base_url=self.base_url, timeout=10.0)
        
        # Test connection
        try:
            resp = await self._http.get("/")
            resp.raise_for_status()
            logger.info(f"Connected to USB web interface at {self.base_url}")
        except Exception as e:
            raise ConnectionError(f"USB web interface not available: {e}")
    
    async def disconnect(self) -> None:
        """Close HTTP client."""
        if self._http:
            await self._http.aclose()
            self._http = None
    
    async def read_file(self, path: str) -> bytes:
        """Read file via web interface (limited support)."""
        # The USB web interface has limited file access
        # Most operations require SSH
        raise NotImplementedError(
            "Direct file read not supported via USB web interface. "
            "Use SSHConnection for full file access."
        )
    
    async def write_file(self, path: str, data: bytes) -> None:
        """Write file via web interface (limited support)."""
        raise NotImplementedError(
            "Direct file write not supported via USB web interface. "
            "Use SSHConnection for full file access."
        )
    
    async def list_dir(self, path: str) -> list[str]:
        """List directory via web interface (limited support)."""
        raise NotImplementedError(
            "Directory listing not supported via USB web interface. "
            "Use SSHConnection for full file access."
        )
    
    async def execute(self, command: str) -> tuple[str, str, int]:
        """Execute command (not supported via web interface)."""
        raise NotImplementedError(
            "Command execution not supported via USB web interface. "
            "Use SSHConnection for shell access."
        )
    
    async def list_documents_web(self) -> list[dict[str, Any]]:
        """List documents via USB web interface."""
        if not self._http:
            raise RuntimeError("Not connected")
        
        resp = await self._http.get("/documents/")
        resp.raise_for_status()
        return resp.json()
    
    async def download_document_web(
        self, doc_id: str, output_path: Path
    ) -> Path:
        """Download document via USB web interface."""
        if not self._http:
            raise RuntimeError("Not connected")
        
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        
        resp = await self._http.get(f"/download/{doc_id}/placeholder")
        resp.raise_for_status()
        
        output_path.write_bytes(resp.content)
        return output_path
    
    async def upload_document_web(
        self, file_path: Path, parent_id: str = ""
    ) -> str:
        """Upload document via USB web interface."""
        if not self._http:
            raise RuntimeError("Not connected")
        
        file_path = Path(file_path)
        
        files = {"file": (file_path.name, file_path.read_bytes())}
        resp = await self._http.post(
            "/upload",
            files=files,
            data={"parent": parent_id},
        )
        resp.raise_for_status()
        
        # Returns the new document ID
        return resp.json().get("ID", "")
