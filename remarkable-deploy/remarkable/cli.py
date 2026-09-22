"""
Command-line interface for reMarkable SDK.

Provides commands for authentication, sync, and document management.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import logging
import sys
from pathlib import Path

logging.basicConfig(
    level=logging.INFO,
    format="%(levelname)s: %(message)s",
)
logger = logging.getLogger(__name__)


def main() -> int:
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        prog="remarkable",
        description="reMarkable SDK CLI",
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Enable verbose output",
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Commands")
    
    # pair command
    pair_parser = subparsers.add_parser("pair", help="Pair with server")
    pair_parser.add_argument(
        "code",
        nargs="?",
        help="Pairing code (optional, will request one if not provided)",
    )
    pair_parser.add_argument(
        "--local", "-l",
        metavar="URL",
        help="Local server URL (e.g., http://192.168.1.100:8080)",
    )
    pair_parser.add_argument(
        "--discover", "-d",
        action="store_true",
        help="Auto-discover local server on network",
    )
    pair_parser.add_argument(
        "--tokens", "-t",
        default="tokens.json",
        help="Token file path (default: tokens.json)",
    )
    pair_parser.add_argument(
        "--name", "-n",
        default="remarkable-cli",
        help="Device name for registration",
    )
    
    # auth command
    auth_parser = subparsers.add_parser("auth", help="Authentication (cloud)")
    auth_parser.add_argument("code", nargs="?", help="One-time code from my.remarkable.com")
    auth_parser.add_argument(
        "--tokens", "-t",
        default="tokens.json",
        help="Token file path (default: tokens.json)",
    )
    
    # sync command  
    sync_parser = subparsers.add_parser("sync", help="Sync operations")
    sync_parser.add_argument(
        "action",
        choices=["list", "pull", "backup", "root"],
        help="Sync action",
    )
    sync_parser.add_argument(
        "--tokens", "-t",
        default="tokens.json",
        help="Token file path",
    )
    sync_parser.add_argument(
        "--local", "-l",
        metavar="URL",
        help="Use local server instead of cloud",
    )
    sync_parser.add_argument(
        "--output", "-o",
        help="Output directory",
    )
    sync_parser.add_argument(
        "--doc", "-d",
        help="Document ID for pull",
    )
    
    # parse command
    parse_parser = subparsers.add_parser("parse", help="Parse .rm files")
    parse_parser.add_argument("file", help="Path to .rm file")
    parse_parser.add_argument(
        "--svg", "-s",
        help="Output SVG file",
    )
    parse_parser.add_argument(
        "--json", "-j",
        action="store_true",
        help="Output as JSON",
    )
    
    # device command
    device_parser = subparsers.add_parser("device", help="Device operations")
    device_parser.add_argument(
        "action",
        choices=["status", "list", "download", "upload"],
        help="Device action",
    )
    device_parser.add_argument(
        "--host",
        default="10.11.99.1",
        help="Device IP address",
    )
    device_parser.add_argument(
        "--password", "-p",
        help="SSH password",
    )
    device_parser.add_argument(
        "--doc", "-d",
        help="Document ID",
    )
    device_parser.add_argument(
        "--output", "-o",
        help="Output directory",
    )
    
    # server command
    server_parser = subparsers.add_parser("server", help="Run local sync server")
    server_sub = server_parser.add_subparsers(dest="server_action", help="Server actions")
    
    # server start
    start_parser = server_sub.add_parser("start", help="Start the server")
    start_parser.add_argument(
        "--storage",
        default="./remarkable_sync",
        help="Storage directory",
    )
    start_parser.add_argument(
        "--host",
        default="0.0.0.0",
        help="Bind address",
    )
    start_parser.add_argument(
        "--port",
        type=int,
        default=8080,
        help="Port number",
    )
    start_parser.add_argument(
        "--auth",
        action="store_true",
        help="Require authentication",
    )
    start_parser.add_argument(
        "--auto-approve",
        action="store_true",
        help="Auto-approve pairing requests",
    )
    start_parser.add_argument(
        "--name",
        default="reMarkable Local Server",
        help="Server name",
    )
    
    # server pair (generate pairing code)
    server_pair_parser = server_sub.add_parser("pair", help="Generate pairing code")
    server_pair_parser.add_argument(
        "--storage",
        default="./remarkable_sync",
        help="Storage directory",
    )
    
    # server devices
    server_devices_parser = server_sub.add_parser("devices", help="List registered devices")
    server_devices_parser.add_argument(
        "--storage",
        default="./remarkable_sync",
        help="Storage directory",
    )
    
    # discover command
    discover_parser = subparsers.add_parser("discover", help="Discover local servers")
    discover_parser.add_argument(
        "--timeout", "-t",
        type=float,
        default=5.0,
        help="Discovery timeout in seconds",
    )
    discover_parser.add_argument(
        "--scan", "-s",
        action="store_true",
        help="Fall back to network scanning if mDNS fails",
    )
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    if args.command is None:
        parser.print_help()
        return 1
    
    try:
        if args.command == "pair":
            return asyncio.run(cmd_pair(args))
        elif args.command == "auth":
            return asyncio.run(cmd_auth(args))
        elif args.command == "sync":
            return asyncio.run(cmd_sync(args))
        elif args.command == "parse":
            return cmd_parse(args)
        elif args.command == "device":
            return asyncio.run(cmd_device(args))
        elif args.command == "server":
            return cmd_server(args)
        elif args.command == "discover":
            return asyncio.run(cmd_discover(args))
    except KeyboardInterrupt:
        return 130
    except Exception as e:
        logger.error(f"Error: {e}")
        if args.verbose:
            import traceback
            traceback.print_exc()
        return 1
    
    return 0


async def cmd_pair(args) -> int:
    """Handle pair command."""
    token_path = Path(args.tokens)
    
    if args.local:
        # Pair with local server
        return await cmd_pair_local(args, token_path)
    elif args.discover:
        # Auto-discover and pair
        return await cmd_pair_discover(args, token_path)
    else:
        # Cloud pairing - redirect to auth
        logger.info("For cloud pairing, use: remarkable auth <code>")
        logger.info("For local server pairing, use: remarkable pair --local <url>")
        return 1


async def cmd_pair_local(args, token_path: Path) -> int:
    """Pair with a local server."""
    from remarkable.local import LocalServerClient
    
    server_url = args.local
    device_name = getattr(args, "name", "remarkable-cli")
    
    print(f"Connecting to local server: {server_url}")
    
    async with LocalServerClient(server_url, device_name=device_name) as client:
        # Verify connection
        if not await client.verify_connection():
            logger.error(f"Cannot connect to server at {server_url}")
            return 1
        
        print(f"Connected to server")
        
        if args.code:
            # Use provided code
            code = args.code
            print(f"Using pairing code: {code}")
        else:
            # Request code from server
            code = await client.request_pairing_code()
            print(f"\nPairing code: {code}")
            print("Waiting for server approval...")
        
        try:
            tokens = await client.complete_pairing(code)
        except TimeoutError:
            logger.error("Pairing timed out. The server may need to approve the request.")
            return 1
        except ValueError as e:
            logger.error(str(e))
            return 1
        
        # Save tokens
        await client.save_tokens(token_path)
        
        print(f"\nPaired successfully!")
        print(f"Device ID: {client.device_info.device_id}")
        print(f"Server: {server_url}")
        print(f"Tokens saved to: {token_path}")
    
    return 0


async def cmd_pair_discover(args, token_path: Path) -> int:
    """Discover local server and pair."""
    from remarkable.local import find_local_server, LocalServerClient
    
    print("Searching for local reMarkable servers...")
    
    server = await find_local_server(timeout=5.0)
    if server is None:
        logger.error("No local server found on network")
        logger.info("Make sure the server is running and try again")
        return 1
    
    print(f"Found server: {server}")
    
    device_name = getattr(args, "name", "remarkable-cli")
    
    async with LocalServerClient(server.url, device_name=device_name) as client:
        if args.code:
            code = args.code
        else:
            code = await client.request_pairing_code()
            print(f"\nPairing code: {code}")
            print("Waiting for server approval...")
        
        try:
            tokens = await client.complete_pairing(code)
        except TimeoutError:
            logger.error("Pairing timed out")
            return 1
        except ValueError as e:
            logger.error(str(e))
            return 1
        
        await client.save_tokens(token_path)
        
        print(f"\nPaired successfully!")
        print(f"Device ID: {client.device_info.device_id}")
        print(f"Server: {server.url}")
        print(f"Tokens saved to: {token_path}")
    
    return 0


async def cmd_auth(args) -> int:
    """Handle auth command."""
    from remarkable.client import RemarkableClient
    
    token_path = Path(args.tokens)
    
    if args.code:
        # Authenticate with code
        async with RemarkableClient() as client:
            tokens = await client.authenticate_with_code(args.code)
            await client.save_tokens(token_path)
            
            print(f"Authenticated successfully!")
            print(f"Device ID: {client.device_info.device_id}")
            print(f"Region: {tokens.region}")
            print(f"Scopes: {' '.join(tokens.scopes)}")
            print(f"Tokens saved to: {token_path}")
    else:
        # Show current auth status
        if token_path.exists():
            async with RemarkableClient() as client:
                tokens = await client.load_tokens(token_path)
                print(f"Token file: {token_path}")
                print(f"Device ID: {client.device_info.device_id}")
                print(f"Region: {tokens.region}")
                print(f"Scopes: {' '.join(tokens.scopes)}")
                print(f"Expired: {tokens.is_expired}")
        else:
            print(f"No tokens found at {token_path}")
            print("Run: remarkable auth <code>")
            return 1
    
    return 0


async def cmd_sync(args) -> int:
    """Handle sync command."""
    token_path = Path(args.tokens)
    if not token_path.exists():
        logger.error(f"Token file not found: {token_path}")
        logger.error("Run: remarkable auth <code>")
        return 1
    
    # Check if using local server
    if args.local:
        return await cmd_sync_local(args, token_path)
    
    from remarkable.client import RemarkableClient
    
    async with RemarkableClient() as client:
        await client.load_tokens(token_path)
        
        if args.action == "root":
            root = await client.get_sync_root()
            print(f"Root hash: {root.hash}")
            print(f"Generation: {root.generation}")
            
        elif args.action == "list":
            docs = await client.list_documents()
            print(f"Found {len(docs)} documents:\n")
            for doc in sorted(docs, key=lambda d: d.name):
                pages = f"{doc.page_count} pages" if doc.page_count else ""
                ftype = f" [{doc.file_type}]" if doc.file_type else ""
                print(f"  {doc.name}{ftype} ({pages})")
                print(f"    ID: {doc.id}")
            
        elif args.action == "pull":
            if not args.doc:
                logger.error("Document ID required: --doc <id>")
                return 1
            
            output_dir = Path(args.output or ".")
            path = await client.download_document(args.doc, output_dir)
            print(f"Downloaded to: {path}")
            
        elif args.action == "backup":
            output_dir = Path(args.output or "./backup")
            stats = await client.sync.backup(output_dir)
            print(f"Backup complete!")
            print(f"  Files: {stats['succeeded']}/{stats['total_files']}")
            print(f"  Output: {stats['output_dir']}")
    
    return 0


async def cmd_sync_local(args, token_path: Path) -> int:
    """Handle sync with local server."""
    from remarkable.local import LocalServerClient
    
    async with LocalServerClient(args.local) as client:
        await client.load_tokens(token_path)
        
        if args.action == "root":
            root = await client.get_sync_root()
            print(f"Root hash: {root.hash}")
            print(f"Generation: {root.generation}")
            
        elif args.action == "list":
            docs = await client.list_documents()
            print(f"Found {len(docs)} documents:\n")
            for doc in sorted(docs, key=lambda d: d.name):
                pages = f"{doc.page_count} pages" if doc.page_count else ""
                ftype = f" [{doc.file_type}]" if doc.file_type else ""
                print(f"  {doc.name}{ftype} ({pages})")
                print(f"    ID: {doc.id}")
            
        elif args.action == "pull":
            if not args.doc:
                logger.error("Document ID required: --doc <id>")
                return 1
            
            output_dir = Path(args.output or ".")
            # TODO: Implement document download for local server
            logger.error("Document pull not yet implemented for local server")
            return 1
            
        elif args.action == "backup":
            # TODO: Implement backup for local server
            logger.error("Backup not yet implemented for local server")
            return 1
    
    return 0


def cmd_parse(args) -> int:
    """Handle parse command."""
    from remarkable.formats import RmParser
    
    rm_path = Path(args.file)
    if not rm_path.exists():
        logger.error(f"File not found: {rm_path}")
        return 1
    
    parser = RmParser()
    doc = parser.parse_file(rm_path)
    
    if args.json:
        # Output stats as JSON
        data = {
            "version": doc.version,
            "layers": doc.layer_count,
            "strokes": doc.stroke_count,
            "points": doc.point_count,
        }
        print(json.dumps(data, indent=2))
        
    elif args.svg:
        # Export to SVG
        svg_path = Path(args.svg)
        svg_content = doc.to_svg()
        svg_path.write_text(svg_content)
        print(f"Exported to: {svg_path}")
        
    else:
        # Print summary
        print(f"File: {rm_path.name}")
        print(f"Version: v{doc.version}")
        print(f"Layers: {doc.layer_count}")
        print(f"Strokes: {doc.stroke_count}")
        print(f"Points: {doc.point_count}")
    
    return 0


async def cmd_device(args) -> int:
    """Handle device command."""
    from remarkable.device import SSHConnection
    
    async with SSHConnection(host=args.host, password=args.password) as conn:
        if args.action == "status":
            status = await conn.get_status()
            print(f"Model: {status.model}")
            print(f"Serial: {status.serial}")
            print(f"Firmware: {status.firmware_version}")
            print(f"Battery: {status.battery_level}% {'(charging)' if status.charging else ''}")
            print(f"Disk: {status.disk_free_mb:.1f} MB free ({status.disk_percent_used:.1f}% used)")
            
        elif args.action == "list":
            docs = await conn.list_documents()
            print(f"Found {len(docs)} documents:\n")
            for doc in docs:
                name = doc.get("visibleName", "Unknown")
                doc_type = doc.get("type", "")
                print(f"  {name} [{doc_type}]")
                print(f"    ID: {doc.get('id', '')}")
            
        elif args.action == "download":
            if not args.doc:
                logger.error("Document ID required: --doc <id>")
                return 1
            
            output_dir = Path(args.output or ".")
            path = await conn.download_document(args.doc, output_dir)
            print(f"Downloaded to: {path}")
            
        elif args.action == "upload":
            if not args.doc:
                logger.error("Document path required: --doc <path>")
                return 1
            
            doc_path = Path(args.doc)
            doc_id = await conn.upload_document(doc_path)
            print(f"Uploaded as: {doc_id}")
    
    return 0


def cmd_server(args) -> int:
    """Handle server command."""
    if args.server_action == "start" or args.server_action is None:
        return cmd_server_start(args)
    elif args.server_action == "pair":
        return cmd_server_pair(args)
    elif args.server_action == "devices":
        return cmd_server_devices(args)
    else:
        logger.error(f"Unknown server action: {args.server_action}")
        return 1


def cmd_server_start(args) -> int:
    """Start the local sync server."""
    from remarkable.server import run_server
    
    storage = getattr(args, "storage", "./remarkable_sync")
    host = getattr(args, "host", "0.0.0.0")
    port = getattr(args, "port", 8080)
    auth = getattr(args, "auth", False)
    auto_approve = getattr(args, "auto_approve", False)
    name = getattr(args, "name", "reMarkable Local Server")
    
    print(f"Starting local sync server...")
    print(f"Storage: {storage}")
    print(f"URL: http://{host}:{port}")
    
    run_server(
        storage_dir=storage,
        host=host,
        port=port,
        require_auth=auth,
        auto_approve=auto_approve,
        server_name=name,
    )
    
    return 0


def cmd_server_pair(args) -> int:
    """Generate a pairing code on the server."""
    from remarkable.server import LocalSyncServer, generate_server_pairing_code
    
    storage = getattr(args, "storage", "./remarkable_sync")
    
    server = LocalSyncServer(storage_dir=storage)
    code = generate_server_pairing_code(server)
    
    print(f"\nPairing Code: {code}")
    print(f"\nEnter this code on the client:")
    print(f"  remarkable pair --local http://<server-ip>:8080 {code}")
    print(f"\nCode expires in 5 minutes.")
    
    return 0


def cmd_server_devices(args) -> int:
    """List registered devices."""
    from remarkable.server import LocalSyncServer
    
    storage = getattr(args, "storage", "./remarkable_sync")
    
    server = LocalSyncServer(storage_dir=storage)
    devices = server.get_registered_devices()
    
    if not devices:
        print("No registered devices")
        return 0
    
    print(f"Registered devices ({len(devices)}):\n")
    for dev in devices:
        last_seen = dev.last_seen.isoformat() if dev.last_seen else "never"
        print(f"  {dev.device_desc}")
        print(f"    ID: {dev.device_id}")
        print(f"    Type: {dev.device_type}")
        print(f"    Registered: {dev.registered_at.isoformat()}")
        print(f"    Last seen: {last_seen}")
        print()
    
    return 0


async def cmd_discover(args) -> int:
    """Discover local servers."""
    from remarkable.local import discover_local_servers, scan_local_network
    
    print("Searching for local reMarkable servers...")
    
    # Try mDNS first
    servers = await discover_local_servers(timeout=args.timeout)
    
    if not servers and args.scan:
        print("No mDNS servers found, scanning network...")
        servers = await scan_local_network(timeout=args.timeout / 2)
    
    if not servers:
        print("No servers found")
        return 1
    
    print(f"\nFound {len(servers)} server(s):\n")
    for server in servers:
        name = f" ({server.name})" if server.name else ""
        version = f" v{server.version}" if server.version else ""
        secure = " [HTTPS]" if server.secure else ""
        print(f"  {server.url}{name}{version}{secure}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
