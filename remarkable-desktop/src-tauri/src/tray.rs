//! System tray integration

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, Runtime,
};
use tracing::info;

// use crate::state::AppState;  // Reserved for future tray status updates

/// Set up system tray
pub fn setup_tray<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let sync_now = MenuItem::with_id(app, "sync_now", "Sync Now", true, None::<&str>)?;
    let offline = MenuItem::with_id(app, "offline", "Toggle Offline Mode", true, None::<&str>)?;
    let separator = MenuItem::with_id(app, "sep", "─────────────────", false, None::<&str>)?;
    
    let menu = Menu::with_items(
        app,
        &[&show, &hide, &separator, &sync_now, &offline, &separator, &quit],
    )?;
    
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("reMarkable Sync")
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    
    info!("System tray initialized");
    Ok(())
}

/// Handle tray menu events
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "quit" => {
            info!("Quit requested from tray");
            app.exit(0);
        }
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        "sync_now" => {
            info!("Sync requested from tray");
            let _ = app.emit("trigger-sync", ());
        }
        "offline" => {
            info!("Offline toggle requested from tray");
            let _ = app.emit("toggle-offline", ());
        }
        _ => {}
    }
}
