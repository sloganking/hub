// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod license_commands;
mod process_manager;
mod tauri_commands;

use hub_common::{HubConfig, ToolId};
use parking_lot::RwLock;
use process_manager::ProcessManager;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

/// Application state shared across the app
pub struct AppState {
    pub config: RwLock<HubConfig>,
    pub process_manager: RwLock<ProcessManager>,
}

impl AppState {
    pub fn new(config: HubConfig) -> Self {
        let mut pm = ProcessManager::new();
        // Detect already-running tools (done here so it's ready when UI loads)
        pm.init_detect_running();
        
        Self {
            config: RwLock::new(config),
            process_manager: RwLock::new(pm),
        }
    }
}

fn create_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let show_dashboard = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    Menu::with_items(app, &[&show_dashboard, &quit_item])
}

fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            // Don't stop tools - let them keep running
            app.exit(0);
        }
        _ => {}
    }
}

fn auto_start_tools<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let config = state.config.read().clone();
    let has_api_key = hub_common::config::has_api_key();
    
    for tool_id in ToolId::all() {
        let tool_config = config.get_tool_config(tool_id);
        
        if tool_config.enabled && tool_config.auto_start {
            // Skip if tool requires API key but we don't have one
            if tool_id.requires_api_key() && !has_api_key {
                continue;
            }
            
            let mut pm = state.process_manager.write();
            let _ = pm.start_tool_with_config(tool_id, &tool_config);
        }
    }
}

fn main() {
    // Load configuration
    let config = HubConfig::load().unwrap_or_default();
    let should_minimize = config.start_minimized;
    let app_state = AppState::new(config);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            println!("Second instance detected - bringing existing window to front");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Config commands
            tauri_commands::get_config,
            tauri_commands::save_config,
            tauri_commands::has_api_key,
            tauri_commands::get_api_key_masked,
            tauri_commands::get_api_key,
            tauri_commands::save_api_key,
            tauri_commands::delete_api_key,
            tauri_commands::validate_api_key,
            tauri_commands::get_tool_statuses,
            tauri_commands::scan_external_processes,
            tauri_commands::start_tool,
            tauri_commands::stop_tool,
            tauri_commands::open_tool_settings,
            // License commands
            license_commands::get_auth_status,
            license_commands::is_authorized,
            license_commands::get_trial_info,
            license_commands::start_trial,
            license_commands::activate_license,
            license_commands::validate_license,
            license_commands::deactivate_license,
            license_commands::get_checkout_url,
            license_commands::open_checkout,
        ])
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().as_ref());
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            let handle_for_tray = app.handle().clone();

            // Create tray menu
            let menu = create_tray_menu(&handle)?;
            let icon = app.default_window_icon().cloned();

            let mut builder = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .tooltip("Productivity Hub");
            if let Some(icon) = icon {
                builder = builder.icon(icon);
            }

            let _tray = builder
                .on_tray_icon_event(move |_tray, event| {
                    handle_tray_event(&handle_for_tray, event);
                })
                .build(app)?;

            // Setup window close behavior (hide instead of close)
            if let Some(window) = app.get_webview_window("main") {
                let window_handle = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_handle.hide();
                    }
                });

                if !should_minimize {
                    let _ = window.show();
                    println!("Starting with window visible");
                } else {
                    println!("Starting minimized to tray");
                }
            }

            // Auto-start configured tools
            auto_start_tools(&handle);

            Ok(())
        })
        .run(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");
}
