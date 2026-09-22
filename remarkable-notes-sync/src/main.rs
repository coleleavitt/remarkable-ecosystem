//! remarkable-sync CLI - Sync reMarkable with Obsidian and Notion

use std::path::PathBuf;
use clap::{Parser, Subcommand, Args};
use tokio;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

use remarkable_sync::{
    AuthTokens, SyncCoordinator, 
    VaultConfig, NotionConfig,
    Strategy,
};

#[derive(Parser)]
#[command(name = "remarkable-sync")]
#[command(about = "Sync reMarkable with Obsidian and Notion", long_about = None)]
#[command(version)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Config file path
    #[arg(short, long, global = true, default_value = "~/.config/remarkable-sync/config.toml")]
    config: PathBuf,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync with Obsidian vault
    Obsidian(ObsidianArgs),
    
    /// Sync with Notion workspace
    Notion(NotionArgs),
    
    /// Authentication commands
    Auth(AuthArgs),
    
    /// List documents on reMarkable
    List,
    
    /// Export a document
    Export(ExportArgs),
}

#[derive(Args)]
struct ObsidianArgs {
    #[command(subcommand)]
    action: ObsidianAction,
}

#[derive(Subcommand)]
enum ObsidianAction {
    /// Push documents to Obsidian
    Push {
        /// Vault path
        #[arg(short, long)]
        vault: PathBuf,
        
        /// Specific document ID (optional)
        #[arg(short, long)]
        document: Option<String>,
    },
    
    /// Pull documents from Obsidian
    Pull {
        /// Vault path
        #[arg(short, long)]
        vault: PathBuf,
    },
    
    /// Watch vault for changes
    Watch {
        /// Vault path
        #[arg(short, long)]
        vault: PathBuf,
    },
}

#[derive(Args)]
struct NotionArgs {
    #[command(subcommand)]
    action: NotionAction,
}

#[derive(Subcommand)]
enum NotionAction {
    /// Push documents to Notion
    Push {
        /// Notion API token
        #[arg(short, long, env = "NOTION_TOKEN")]
        token: String,
        
        /// Parent page or database ID
        #[arg(short, long)]
        parent: String,
        
        /// Specific document ID (optional)
        #[arg(short, long)]
        document: Option<String>,
    },
    
    /// Pull documents from Notion
    Pull {
        /// Notion API token
        #[arg(short, long, env = "NOTION_TOKEN")]
        token: String,
        
        /// Database ID to pull from
        #[arg(short, long)]
        database: String,
    },
}

#[derive(Args)]
struct AuthArgs {
    #[command(subcommand)]
    action: AuthAction,
}

#[derive(Subcommand)]
enum AuthAction {
    /// Login with one-time code
    Login {
        /// One-time code from my.remarkable.com
        #[arg(short, long)]
        code: String,
    },
    
    /// Show current auth status
    Status,
    
    /// Clear stored credentials
    Logout,
}

#[derive(Args)]
struct ExportArgs {
    /// Document ID
    #[arg(short, long)]
    document: String,
    
    /// Output format (svg, md, pdf)
    #[arg(short, long, default_value = "svg")]
    format: String,
    
    /// Output path
    #[arg(short, long)]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    // Expand config path
    let config_path = shellexpand::tilde(&cli.config.to_string_lossy()).to_string();
    let config_path = PathBuf::from(config_path);
    
    match cli.command {
        Commands::Obsidian(args) => handle_obsidian(args, &config_path).await?,
        Commands::Notion(args) => handle_notion(args, &config_path).await?,
        Commands::Auth(args) => handle_auth(args, &config_path).await?,
        Commands::List => handle_list(&config_path).await?,
        Commands::Export(args) => handle_export(args, &config_path).await?,
    }
    
    Ok(())
}

async fn handle_obsidian(args: ObsidianArgs, config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let tokens = load_tokens(config_path)?;
    let data_dir = config_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    
    let mut coordinator = SyncCoordinator::new(tokens, data_dir, Strategy::LastWriteWins);
    
    match args.action {
        ObsidianAction::Push { vault, document } => {
            info!("Pushing to Obsidian vault: {}", vault.display());
            
            let vault_config = VaultConfig {
                path: vault,
                ..Default::default()
            };
            
            coordinator.with_obsidian(vault_config)?;
            let result = coordinator.push_to_obsidian().await?;
            
            info!("Synced {} documents", result.documents_synced);
            for err in &result.errors {
                error!("Error: {}", err);
            }
        }
        ObsidianAction::Pull { vault } => {
            info!("Pulling from Obsidian vault: {}", vault.display());
            // TODO: Implement pull
            info!("Pull not yet implemented");
        }
        ObsidianAction::Watch { vault } => {
            info!("Watching Obsidian vault: {}", vault.display());
            
            let vault_config = VaultConfig {
                path: vault,
                ..Default::default()
            };
            
            coordinator.with_obsidian(vault_config)?;
            coordinator.watch_obsidian().await?;
        }
    }
    
    Ok(())
}

async fn handle_notion(args: NotionArgs, config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let tokens = load_tokens(config_path)?;
    let data_dir = config_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    
    let mut coordinator = SyncCoordinator::new(tokens, data_dir, Strategy::LastWriteWins);
    
    match args.action {
        NotionAction::Push { token, parent, document } => {
            info!("Pushing to Notion");
            
            let notion_config = NotionConfig::new(token);
            coordinator.with_notion(notion_config)?;
            
            let result = coordinator.push_to_notion(&parent).await?;
            
            info!("Synced {} documents", result.documents_synced);
            for err in &result.errors {
                error!("Error: {}", err);
            }
        }
        NotionAction::Pull { token, database } => {
            info!("Pulling from Notion database: {}", database);
            // TODO: Implement pull
            info!("Pull not yet implemented");
        }
    }
    
    Ok(())
}

async fn handle_auth(args: AuthArgs, config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    match args.action {
        AuthAction::Login { code } => {
            info!("Authenticating with code: {}", code);
            // TODO: Implement device registration
            info!("Login not yet fully implemented - store device token manually");
        }
        AuthAction::Status => {
            match load_tokens(config_path) {
                Ok(tokens) => {
                    info!("Device token: {}...", &tokens.device_token[..20.min(tokens.device_token.len())]);
                    if let Some(region) = &tokens.tectonic_region {
                        info!("Region: {}", region);
                    }
                }
                Err(e) => {
                    error!("Not authenticated: {}", e);
                }
            }
        }
        AuthAction::Logout => {
            let tokens_path = config_path.parent()
                .unwrap_or(&PathBuf::from("."))
                .join("tokens.json");
            
            if tokens_path.exists() {
                std::fs::remove_file(&tokens_path)?;
                info!("Logged out");
            } else {
                info!("No stored credentials");
            }
        }
    }
    
    Ok(())
}

async fn handle_list(config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let tokens = load_tokens(config_path)?;
    let mut client = remarkable_sync::RemarkableClient::new(tokens);
    
    info!("Fetching document list...");
    let documents = client.list_documents().await?;
    
    println!("\n{} documents found:\n", documents.len());
    
    for doc in &documents {
        let name = if doc.visible_name.is_empty() {
            &doc.id
        } else {
            &doc.visible_name
        };
        println!("  {} ({}) - v{}", name, doc.doc_type, doc.version);
    }
    
    Ok(())
}

async fn handle_export(args: ExportArgs, config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let tokens = load_tokens(config_path)?;
    let mut client = remarkable_sync::RemarkableClient::new(tokens);
    
    info!("Exporting document {} to {}", args.document, args.output.display());
    
    // TODO: Implement export
    info!("Export not yet implemented");
    
    Ok(())
}

fn load_tokens(config_path: &PathBuf) -> Result<AuthTokens, Box<dyn std::error::Error>> {
    let tokens_path = config_path.parent()
        .unwrap_or(&PathBuf::from("."))
        .join("tokens.json");
    
    if !tokens_path.exists() {
        return Err("No stored credentials. Run 'remarkable-sync auth login' first.".into());
    }
    
    let content = std::fs::read_to_string(&tokens_path)?;
    let tokens: AuthTokens = serde_json::from_str(&content)?;
    
    Ok(tokens)
}
