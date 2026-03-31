use crate::commands::{
    ActivityBarRegistry, CommandRegistry, ConfigurationRegistry, DebugConfigurationRegistry,
    FileSystemRegistry, KeybindingRegistry, LanguageFeaturesRegistry, MarketplaceUIRegistry,
    MenuRegistry, SettingsUIRegistry, StatusBarRegistry, TaskRegistry, TestRunnerRegistry,
    TextDocumentRegistry, ThemeRegistry,
};
use crate::monitoring::ResourceMonitor;
use crate::session::SessionManager;
use crate::watcher::FileWatcherState;
use dscode_core::AppDirectories;
use dscode_dap::{DebugAdapterPool, DebugManager};
use dscode_extension_host::PathValidator;
use dscode_lsp::LspManager;
use dscode_terminal::{TerminalManager, TauriEventSender};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, Wry,
};
use tokio::sync::RwLock;
use tracing::{error, info};

type SetupResult = Result<(), Box<dyn Error>>;

pub fn configure_builder(
    builder: tauri::Builder<Wry>, lsp_manager: LspManager,
) -> tauri::Builder<Wry> {
    builder
        .plugin(tauri_plugin_dialog::init())
        .manage(FileWatcherState::new())
        .manage(ResourceMonitor::new())
        .manage(tokio::sync::Mutex::new(lsp_manager))
        .setup(setup_app)
}

fn setup_app(app: &mut tauri::App<Wry>) -> SetupResult {
    let app_dirs = register_app_directories(app)?;
    build_tray(app)?;

    let path_validator = create_path_validator(&app_dirs);
    app.manage(path_validator.clone());

    // Create TerminalManager with TauriEventSender for forwarding PTY events
    let terminal_manager = TerminalManager::new(Box::new(TauriEventSender::new(app.handle().clone())));
    app.manage(Mutex::new(terminal_manager));

    app.manage(Mutex::new(DebugManager::new()));
    app.manage(RwLock::new(DebugAdapterPool::new()));

    let session_manager = register_session_manager(app, app_dirs.clone(), path_validator);
    register_feature_registries(app);
    start_session_initialization(session_manager);

    Ok(())
}

fn create_path_validator(app_dirs: &AppDirectories) -> Arc<RwLock<PathValidator>> {
    let mut pv = PathValidator::new();
    pv.set_extensions_dir(app_dirs.extensions_dir.clone());
    pv.set_storage_dir(app_dirs.storage_dir.clone());
    pv.set_logs_dir(app_dirs.logs_dir.clone());
    pv.set_temp_dir(std::env::temp_dir());
    Arc::new(RwLock::new(pv))
}

fn register_app_directories(app: &mut tauri::App<Wry>) -> Result<AppDirectories, Box<dyn Error>> {
    let app_dirs = AppDirectories::resolve()
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    app.manage(app_dirs.clone());
    Ok(app_dirs)
}

fn build_tray(app: &mut tauri::App<Wry>) -> SetupResult {
    let icon_bytes = include_bytes!("../icons/tray-icon.png");
    let icon = Image::from_bytes(icon_bytes)
        .map_err(|error| std::io::Error::other(format!("Failed to load icon: {error}")))?;

    let show = MenuItem::with_id(app, "show", "Show DSCode", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    info!("[Tray] Window restored via tray click");
                }
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    info!("[Tray] Window restored via menu");
                }
            }
            "quit" => {
                info!("[Tray] Quit requested");
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn register_session_manager(
    app: &mut tauri::App<Wry>, app_dirs: AppDirectories,
    path_validator: Arc<RwLock<PathValidator>>,
) -> Arc<RwLock<SessionManager>> {
    let session_manager = Arc::new(RwLock::new(SessionManager::new(
        app.handle().clone(),
        app_dirs,
        path_validator,
    )));

    app.manage(session_manager.clone());
    session_manager
}

fn register_feature_registries(app: &mut tauri::App<Wry>) {
    let app_handle = app.handle().clone();

    app.manage(CommandRegistry::new(app_handle.clone()));
    app.manage(MenuRegistry::new(app_handle.clone()));
    app.manage(KeybindingRegistry::new(app_handle.clone()));
    app.manage(StatusBarRegistry::new(app_handle.clone()));
    app.manage(ActivityBarRegistry::new(app_handle.clone()));
    app.manage(LanguageFeaturesRegistry::new(app_handle.clone()));
    app.manage(FileSystemRegistry::new(app_handle.clone()));
    app.manage(TextDocumentRegistry::new(app_handle.clone()));

    let configuration_registry = Arc::new(ConfigurationRegistry::new(app_handle.clone()));
    app.manage(configuration_registry.clone());

    app.manage(SettingsUIRegistry::new(app_handle.clone(), configuration_registry));
    app.manage(DebugConfigurationRegistry::new(app_handle.clone()));
    app.manage(ThemeRegistry::new(app_handle.clone()));
    app.manage(TaskRegistry::new(app_handle.clone()));
    app.manage(MarketplaceUIRegistry::new(app_handle.clone()));
    app.manage(TestRunnerRegistry::new(app_handle));
}

fn start_session_initialization(session_manager: Arc<RwLock<SessionManager>>) {
    // Clean up stale IPC sockets from previous sessions before starting
    crate::session::ipc::cleanup_stale_sockets();

    tauri::async_runtime::spawn(async move {
        let sm = session_manager.read().await;
        // Start periodic cleanup of stale pending requests
        sm.start_pending_request_cleanup();

        if let Err(error) = sm.initialize().await {
            error!("[App] Failed to initialize session: {error}");
        }
    });
}

/// Register default LSP servers asynchronously after app startup
pub fn register_defaults_async(lsp_manager: Arc<tokio::sync::Mutex<LspManager>>) {
    tauri::async_runtime::spawn(async move {
        let manager = lsp_manager.lock().await;
        manager.register_defaults().await;
    });
}
