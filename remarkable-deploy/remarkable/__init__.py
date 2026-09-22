"""
reMarkable Python SDK

A comprehensive SDK for interacting with reMarkable tablets via cloud sync,
local server, USB, or SSH.
"""

from remarkable.client import RemarkableClient
from remarkable.models import (
    Document,
    Collection,
    DeviceInfo,
    SyncRoot,
    DocumentSchema,
    PageData,
)
from remarkable.formats import RmParser, Stroke, Point, Layer
from remarkable.device import DeviceConnection, SSHConnection, USBConnection
from remarkable.local import (
    LocalServerClient,
    LocalServerInfo,
    discover_local_servers,
    scan_local_network,
    find_local_server,
    pair_with_local_server,
)
from remarkable.server import LocalSyncServer

__version__ = "0.1.0"
__all__ = [
    # Cloud client
    "RemarkableClient",
    # Local server client
    "LocalServerClient",
    "LocalServerInfo",
    "LocalSyncServer",
    # Discovery
    "discover_local_servers",
    "scan_local_network",
    "find_local_server",
    "pair_with_local_server",
    # Models
    "Document",
    "Collection", 
    "DeviceInfo",
    "SyncRoot",
    "DocumentSchema",
    "PageData",
    # Formats
    "RmParser",
    "Stroke",
    "Point",
    "Layer",
    # Device connections
    "DeviceConnection",
    "SSHConnection",
    "USBConnection",
]
