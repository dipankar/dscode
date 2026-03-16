// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod watcher;
mod extension_host;
mod monitoring;
mod lsp;

use commands::*;
use watcher::FileWatcherState;
use extension_host::ExtensionHostManager;
use monitoring::ResourceMonitor;
use lsp::LspManager;
use std::sync::Mutex;

fn main() {
    // Initialize LSP manager with default language servers
    let lsp_manager = LspManager::new();
    lsp_manager.initialize_defaults();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(FileWatcherState::new())
        .manage(Mutex::new(ExtensionHostManager::new()))
        .manage(ResourceMonitor::new())
        .manage(Mutex::new(lsp_manager))
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
            get_resource_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
