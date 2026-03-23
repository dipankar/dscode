use crate::commands::{
    ActivityBarRegistry, CommandRegistry, ConfigurationRegistry, DebugConfigurationRegistry,
    FileSystemRegistry, KeybindingRegistry, LanguageFeaturesRegistry, MarketplaceUIRegistry,
    MenuRegistry, SettingsUIRegistry, StatusBarRegistry, TaskRegistry, TestRunnerRegistry,
    TextDocumentRegistry, ThemeRegistry, WorkspaceRegistry,
};
use crate::config::AppDirectories;
use crate::debug::DebugManager;
use crate::extension_host::{ExtensionHostManager, NngIpcManager};
use crate::lsp::LspManager;
use crate::monitoring::ResourceMonitor;
use crate::session::SessionManager;
use crate::terminal::TerminalManager;
use crate::watcher::FileWatcherState;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, Wry,
};
use tokio::sync::RwLock;

type SetupResult = Result<(), Box<dyn Error>>;

pub fn configure_builder(
    builder: tauri::Builder<Wry>,
    lsp_manager: LspManager,
) -> tauri::Builder<Wry> {
    builder
        .plugin(tauri_plugin_dialog::init())
        .manage(FileWatcherState::new())
        .manage(Mutex::new(ExtensionHostManager::new("global".to_string())))
        .manage(NngIpcManager::new())
        .manage(ResourceMonitor::new())
        .manage(Mutex::new(lsp_manager))
        .manage(Mutex::new(TerminalManager::new()))
        .manage(Mutex::new(DebugManager::new()))
        .setup(setup_app)
}

fn setup_app(app: &mut tauri::App<Wry>) -> SetupResult {
    let app_dirs = register_app_directories(app)?;
    build_tray(app)?;

    let session_manager = register_session_manager(app, app_dirs.clone());
    register_feature_registries(app);
    start_session_initialization(session_manager);

    Ok(())
}

fn register_app_directories(app: &mut tauri::App<Wry>) -> Result<AppDirectories, Box<dyn Error>> {
    let app_dirs = AppDirectories::from_app_config(app.config())
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
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    println!("[Tray] Window restored via tray click");
                }
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
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
        })
        .build(app)?;

    Ok(())
}

fn register_session_manager(
    app: &mut tauri::App<Wry>,
    app_dirs: AppDirectories,
) -> Arc<RwLock<SessionManager>> {
    let session_manager = Arc::new(RwLock::new(SessionManager::new(
        app.handle().clone(),
        app_dirs,
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
    app.manage(WorkspaceRegistry::new(app_handle.clone()));
    app.manage(FileSystemRegistry::new(app_handle.clone()));
    app.manage(TextDocumentRegistry::new(app_handle.clone()));

    let configuration_registry = Arc::new(ConfigurationRegistry::new(app_handle.clone()));
    app.manage(configuration_registry.clone());

    app.manage(SettingsUIRegistry::new(
        app_handle.clone(),
        configuration_registry,
    ));
    app.manage(DebugConfigurationRegistry::new(app_handle.clone()));
    app.manage(ThemeRegistry::new(app_handle.clone()));
    app.manage(TaskRegistry::new(app_handle.clone()));
    app.manage(MarketplaceUIRegistry::new(app_handle.clone()));
    app.manage(TestRunnerRegistry::new(app_handle));
}

fn start_session_initialization(session_manager: Arc<RwLock<SessionManager>>) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = session_manager.read().await.initialize().await {
            eprintln!("[App] Failed to initialize session: {error}");
        }
    });
}
