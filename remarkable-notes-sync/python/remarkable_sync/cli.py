"""
CLI for remarkable-sync
"""

import sys
import argparse


def main():
    """Main CLI entry point"""
    parser = argparse.ArgumentParser(
        prog="remarkable-sync",
        description="Sync reMarkable with Obsidian and Notion"
    )
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")
    
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    # Obsidian commands
    obsidian = subparsers.add_parser("obsidian", help="Sync with Obsidian")
    obsidian_sub = obsidian.add_subparsers(dest="action", required=True)
    
    push = obsidian_sub.add_parser("push", help="Push to Obsidian")
    push.add_argument("--vault", "-v", required=True, help="Vault path")
    push.add_argument("--document", "-d", help="Specific document ID")
    
    pull = obsidian_sub.add_parser("pull", help="Pull from Obsidian")
    pull.add_argument("--vault", "-v", required=True, help="Vault path")
    
    watch = obsidian_sub.add_parser("watch", help="Watch vault")
    watch.add_argument("--vault", "-v", required=True, help="Vault path")
    
    # Notion commands
    notion = subparsers.add_parser("notion", help="Sync with Notion")
    notion_sub = notion.add_subparsers(dest="action", required=True)
    
    notion_push = notion_sub.add_parser("push", help="Push to Notion")
    notion_push.add_argument("--token", "-t", required=True, help="Notion API token")
    notion_push.add_argument("--parent", "-p", required=True, help="Parent page/database ID")
    notion_push.add_argument("--document", "-d", help="Specific document ID")
    
    notion_pull = notion_sub.add_parser("pull", help="Pull from Notion")
    notion_pull.add_argument("--token", "-t", required=True, help="Notion API token")
    notion_pull.add_argument("--database", "-d", required=True, help="Database ID")
    
    # Auth commands
    auth = subparsers.add_parser("auth", help="Authentication")
    auth_sub = auth.add_subparsers(dest="action", required=True)
    
    login = auth_sub.add_parser("login", help="Login with code")
    login.add_argument("--code", "-c", required=True, help="One-time code")
    
    auth_sub.add_parser("status", help="Show auth status")
    auth_sub.add_parser("logout", help="Clear credentials")
    
    # Other commands
    subparsers.add_parser("list", help="List documents")
    
    export = subparsers.add_parser("export", help="Export document")
    export.add_argument("--document", "-d", required=True, help="Document ID")
    export.add_argument("--format", "-f", default="svg", help="Output format")
    export.add_argument("--output", "-o", required=True, help="Output path")
    
    args = parser.parse_args()
    
    if args.command == "obsidian":
        handle_obsidian(args)
    elif args.command == "notion":
        handle_notion(args)
    elif args.command == "auth":
        handle_auth(args)
    elif args.command == "list":
        handle_list(args)
    elif args.command == "export":
        handle_export(args)


def handle_obsidian(args):
    from .sync import RemarkableClient, sync_to_obsidian
    from .obsidian import ObsidianVault, watch_vault
    
    if args.action == "push":
        print(f"Pushing to Obsidian vault: {args.vault}")
        client = RemarkableClient.from_config()
        synced = sync_to_obsidian(client, args.vault, args.document)
        print(f"Synced {synced} documents")
        
    elif args.action == "pull":
        print(f"Pulling from Obsidian vault: {args.vault}")
        print("Pull not yet implemented")
        
    elif args.action == "watch":
        print(f"Watching Obsidian vault: {args.vault}")
        
        def on_change(event_type, path):
            print(f"{event_type}: {path}")
        
        watch_vault(args.vault, on_change)


def handle_notion(args):
    from .sync import RemarkableClient, sync_to_notion
    
    if args.action == "push":
        print("Pushing to Notion")
        client = RemarkableClient.from_config()
        synced = sync_to_notion(client, args.token, args.parent, getattr(args, 'document', None))
        print(f"Synced {synced} documents")
        
    elif args.action == "pull":
        print(f"Pulling from Notion database: {args.database}")
        print("Pull not yet implemented")


def handle_auth(args):
    import json
    from pathlib import Path
    
    config_dir = Path.home() / ".config" / "remarkable-sync"
    tokens_path = config_dir / "tokens.json"
    
    if args.action == "login":
        print(f"Authenticating with code: {args.code}")
        # TODO: Implement device registration
        print("Login not yet fully implemented - store device token manually")
        
    elif args.action == "status":
        if tokens_path.exists():
            with open(tokens_path) as f:
                tokens = json.load(f)
            token = tokens.get("device_token", "")
            print(f"Device token: {token[:20]}..." if len(token) > 20 else token)
            if "tectonic_region" in tokens:
                print(f"Region: {tokens['tectonic_region']}")
        else:
            print("Not authenticated")
        
    elif args.action == "logout":
        if tokens_path.exists():
            tokens_path.unlink()
            print("Logged out")
        else:
            print("No stored credentials")


def handle_list(args):
    from .sync import RemarkableClient
    
    client = RemarkableClient.from_config()
    print("Fetching document list...")
    
    documents = client.list_documents()
    print(f"\n{len(documents)} documents found:\n")
    
    for doc in documents:
        name = doc.visible_name or doc.id
        print(f"  {name} ({doc.doc_type}) - v{doc.version}")


def handle_export(args):
    print(f"Exporting {args.document} to {args.output}")
    print("Export not yet implemented")


if __name__ == "__main__":
    main()
