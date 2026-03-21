// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Configure memory allocator
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(feature = "mimalloc-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// system-allocator uses the default Rust allocator, no configuration needed

mod commands;
mod core;
mod watcher;
mod extension_host;
mod monitoring;
mod lsp;
mod marketplace;
mod terminal;
mod debug;
mod session;
mod config;

use commands::*;
use watcher::FileWatcherState;
use extension_host::{ExtensionHostManager, NngIpcManager};
use monitoring::ResourceMonitor;
use lsp::LspManager;
use terminal::TerminalManager;
use debug::DebugManager;
use session::SessionManager;
use config::AppDirectories;
use std::sync::{Mutex, Arc};
use tokio::sync::RwLock;
use tauri::{Manager, menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent}, image::Image};

fn main() {
    // Initialize LSP manager with default language servers
    let lsp_manager = LspManager::new();
    lsp_manager.initialize_defaults();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(FileWatcherState::new())
        .manage(Mutex::new(ExtensionHostManager::new("global".to_string())))
        .manage(NngIpcManager::new())
        .manage(ResourceMonitor::new())
        .manage(Mutex::new(lsp_manager))
        .manage(Mutex::new(TerminalManager::new()))
        .manage(Mutex::new(DebugManager::new()))
        .setup(|app| {
            let app_dirs = AppDirectories::from_app_config(app.config())
                .map_err(|e| Box::<dyn std::error::Error>::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            app.manage(app_dirs.clone());

            // Load tray icon from PNG bytes
            let icon_bytes = include_bytes!("../icons/tray-icon.png");
            let icon = Image::from_bytes(icon_bytes)
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to load icon: {}", e))))?;

            // Create tray menu
            let show = MenuItem::with_id(app, "show", "Show DSCode", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } => {
                            // Left click on tray - show window
                            if let Some(app) = tray.app_handle().get_webview_window("main") {
                                let _ = app.show();
                                let _ = app.set_focus();
                                println!("[Tray] Window restored via tray click");
                            }
                        }
                        _ => {}
                    }
                })
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                println!("[Tray] Window restored via menu");
                            }
                        }
                        "quit" => {
                            println!("[Tray] Quit requested");
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Initialize Session Manager
            let session_manager = Arc::new(RwLock::new(
                SessionManager::new(app.handle().clone(), app_dirs.clone())
            ));

            // Store in app state
            app.manage(session_manager.clone());

            // Initialize session asynchronously
            let session = session_manager.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = session.read().await.initialize().await {
                    eprintln!("[App] Failed to initialize session: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            read_file,
            write_file,
            list_directory,
            read_directory_tree,
            get_file_language,
            start_watching_file,
            stop_watching_file,
            create_file,
            create_folder,
            delete_file,
            delete_folder,
            rename_path,
            git_status,
            get_app_version,
            start_extension_host,
            get_extensions_dir,
            install_extension,
            uninstall_extension,
            list_extensions,
            get_extension_contributions,
            search_marketplace,
            get_marketplace_extension,
            install_from_marketplace,
            get_resource_metrics,
            create_terminal,
            write_to_terminal,
            resize_terminal,
            close_terminal,
            list_terminals,
            terminal_ready,
            minimize_to_tray,
            restore_from_tray,
            is_window_visible,
            window_message_action,
            window_quick_pick_select,
            window_input_box_submit,
            create_debug_session,
            get_debug_session,
            list_debug_sessions,
            start_debugging,
            pause_debugging,
            continue_debugging,
            stop_debugging,
            step_over,
            step_into,
            step_out,
            set_breakpoints,
            get_breakpoints,
            clear_breakpoints,
            get_all_breakpoints,
            search_in_files,
            replace_in_files,
            git_commit,
            git_stage_file,
            git_unstage_file,
            git_stage_all,
            git_get_diff,
            git_push,
            git_pull,
            git_fetch,
            git_list_branches,
            git_create_branch,
            git_checkout_branch,
            git_delete_branch,
            extension_tree_get_children,
            extension_tree_get_item,
            extension_execute_command,
            extension_tree_notify_event,
            initialize_session,
            get_session_state,
            get_installed_extensions,
            get_active_extensions,
            session_load_extension,
            session_unload_extension,
            session_delete_extension,
            add_workspace_folder,
            remove_workspace_folder,
            editor_selection_changed,
            editor_visible_ranges_changed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
