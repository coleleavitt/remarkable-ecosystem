//! reMarkable CRDT Diff & Merge Viewer Server
//!
//! Web server providing APIs for:
//! - Loading and parsing .rm files
//! - Computing diffs between versions
//! - Visualizing CRDT operation timelines
//! - Merging conflicting versions
//! - Exporting merged results

use axum::{
    Router,
    routing::{get, post},
};
use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod state;
mod parsers;
mod error;

use state::AppState;

#[derive(Parser, Debug)]
#[command(name = "remarkable-diff")]
#[command(about = "Visual CRDT merge/diff viewer for reMarkable documents")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "3001")]
    port: u16,
    
    /// Static files directory
    #[arg(long, default_value = "./web/dist")]
    static_dir: String,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Initialize tracing
    let filter = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| filter.into()))
        .init();
    
    tracing::info!("Starting remarkable-diff server on port {}", args.port);
    
    // Create app state
    let state = Arc::new(RwLock::new(AppState::new()));
    
    // Build router
    let api = Router::new()
        // Document operations
        .route("/documents", get(handlers::list_documents))
        .route("/documents", post(handlers::upload_document))
        .route("/documents/:id", get(handlers::get_document))
        .route("/documents/:id/versions", get(handlers::list_versions))
        .route("/documents/:id/versions/:version_id", get(handlers::get_version))
        
        // Diff operations
        .route("/diff", post(handlers::compute_diff))
        .route("/diff/:id", get(handlers::get_diff))
        
        // Merge operations
        .route("/merge", post(handlers::merge_versions))
        .route("/merge/:id/resolve", post(handlers::resolve_conflict))
        .route("/merge/:id/export", get(handlers::export_merged))
        
        // Timeline
        .route("/timeline/:doc_id", get(handlers::get_timeline))
        .route("/timeline/:doc_id/filter", post(handlers::filter_timeline))
        
        // Stroke rendering
        .route("/render/stroke/:stroke_id", get(handlers::render_stroke))
        .route("/render/page/:page_id", get(handlers::render_page))
        
        .with_state(state);
    
    let app = Router::new()
        .nest("/api", api)
        .nest_service("/", ServeDir::new(&args.static_dir))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", args.port)).await?;
    tracing::info!("Listening on http://0.0.0.0:{}", args.port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
