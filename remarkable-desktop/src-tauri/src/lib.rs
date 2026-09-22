//! reMarkable Desktop Sync Application
//! 
//! Provides background sync with reMarkable cloud or local server,
//! system tray integration, conflict resolution, and folder selection.

mod commands;
mod sync;
mod state;
mod tray;
mod error;
mod config;

use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_log::{Target, TargetKind};
use tracing_subscriber::EnvFilter;

pub use error::{Error, Result};
pub use state::AppState;

/// Initialize logging
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
}

/// Main library entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
            .targets([
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::LogDir { file_name: None }),
            ])
            .build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state::AppState::new())
        .setup(|app| {
            // Initialize system tray
            tray::setup_tray(app)?;
            
            // Start background sync
            let app_handle = app.handle().clone();
            
            tokio::spawn(async move {
                sync::start_background_sync(app_handle).await;
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_sync_status,
            commands::set_sync_config,
            commands::get_sync_config,
            commands::start_sync,
            commands::stop_sync,
            commands::get_documents,
            commands::get_folders,
            commands::set_selected_folders,
            commands::get_conflicts,
            commands::resolve_conflict,
            commands::set_offline_mode,
            commands::get_offline_mode,
            commands::test_connection,
            commands::export_document,
            commands::import_document,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                RunEvent::WindowEvent { label, event, .. } => {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // Hide window instead of closing when close is requested
                        let window = app_handle.get_webview_window(&label).unwrap();
                        window.hide().unwrap();
                        api.prevent_close();
                    }
                }
                RunEvent::ExitRequested { .. } => {
                    // Allow exit when explicitly requested
                }
                _ => {}
            }
        });
}
