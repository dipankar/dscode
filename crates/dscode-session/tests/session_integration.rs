//! Integration tests for dscode-session crate.
//!
//! Tests multiple session components working together: ConfigurationStore with
//! real file I/O, DocumentManager with language detection, WorkspaceManager
//! path/URI conversions, ExtensionContributes parsing from JSON, and
//! SessionLifecycle state machine transitions.

use dscode_session::{
    detect_language_id, workspace_path_from_uri, workspace_uri_from_path, ConfigurationStore,
    DocumentManager, ExtensionContributes, SessionLifecycle, WorkspaceManager,
};
use serde_json::json;
use std::path::{Path, PathBuf};

// ── ConfigurationStore with real file I/O ────────────────────────────────────

#[test]
fn test_configuration_store_create_and_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    // Write a config file
    let content = r#"{"editor": {"fontSize": 14, "tabSize": 4}}"#;
    std::fs::write(&path, content).unwrap();

    // Load it
    let store = ConfigurationStore::new(path).unwrap();
    let snapshot = store.snapshot(Some("editor"));
    assert_eq!(snapshot.get("fontSize"), Some(&json!(14)));
    assert_eq!(snapshot.get("tabSize"), Some(&json!(4)));
}

#[test]
fn test_configuration_store_update_and_persist() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    // Create and update
    {
        let mut store = ConfigurationStore::new(path.clone()).unwrap();
        store.update(None, "theme", json!("dark")).unwrap();
        store.update(Some("editor"), "fontSize", json!(16)).unwrap();
    }

    // Reload from disk
    let store = ConfigurationStore::new(path).unwrap();
    let root_snapshot = store.snapshot(None);
    assert_eq!(root_snapshot.get("theme"), Some(&json!("dark")));

    let editor_snapshot = store.snapshot(Some("editor"));
    assert_eq!(editor_snapshot.get("fontSize"), Some(&json!(16)));
}

#[test]
fn test_configuration_store_update_returns_change_flag() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let mut store = ConfigurationStore::new(path).unwrap();

    // First set should report change
    let changed = store.update(None, "key", json!("value1")).unwrap();
    assert!(changed);

    // Setting same value should report no change
    let changed = store.update(None, "key", json!("value1")).unwrap();
    assert!(!changed);

    // Setting different value should report change
    let changed = store.update(None, "key", json!("value2")).unwrap();
    assert!(changed);
}

#[test]
fn test_configuration_store_remove_key_with_null() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let mut store = ConfigurationStore::new(path).unwrap();

    store.update(None, "temp", json!("temporary")).unwrap();
    let snapshot = store.snapshot(None);
    assert_eq!(snapshot.get("temp"), Some(&json!("temporary")));

    // Remove by setting to null
    let changed = store.update(None, "temp", json!(null)).unwrap();
    assert!(changed);
}

#[test]
fn test_configuration_store_nested_sections() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let mut store = ConfigurationStore::new(path).unwrap();
    store
        .update(Some("editor.formatting"), "tabSize", json!(2))
        .unwrap();
    store
        .update(Some("editor.formatting"), "insertSpaces", json!(true))
        .unwrap();
    store
        .update(Some("terminal.integrated"), "shell", json!("/bin/bash"))
        .unwrap();

    // Read nested sections
    let formatting = store.snapshot(Some("editor.formatting"));
    assert_eq!(formatting.get("tabSize"), Some(&json!(2)));
    assert_eq!(formatting.get("insertSpaces"), Some(&json!(true)));

    let terminal = store.snapshot(Some("terminal.integrated"));
    assert_eq!(terminal.get("shell"), Some(&json!("/bin/bash")));

    // Read parent section
    let editor = store.snapshot(Some("editor"));
    assert!(editor.get("formatting").is_some());
}

#[test]
fn test_configuration_store_missing_file_creates_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.json");

    let store = ConfigurationStore::new(path).unwrap();
    let snapshot = store.snapshot(None);
    assert!(snapshot.is_object());
    assert!(snapshot.as_object().unwrap().is_empty());
}

#[test]
fn test_configuration_store_persistence_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    // Write multiple settings
    {
        let mut store = ConfigurationStore::new(path.clone()).unwrap();
        store.update(None, "theme", json!("monokai")).unwrap();
        store.update(Some("editor"), "fontSize", json!(14)).unwrap();
        store.update(Some("editor"), "fontFamily", json!("Fira Code")).unwrap();
        store.update(Some("workbench"), "colorTheme", json!("Default Dark+")).unwrap();
    }

    // Reload and verify all settings persisted
    let store = ConfigurationStore::new(path).unwrap();
    assert_eq!(store.snapshot(None).get("theme"), Some(&json!("monokai")));

    let editor = store.snapshot(Some("editor"));
    assert_eq!(editor.get("fontSize"), Some(&json!(14)));
    assert_eq!(editor.get("fontFamily"), Some(&json!("Fira Code")));

    let workbench = store.snapshot(Some("workbench"));
    assert_eq!(workbench.get("colorTheme"), Some(&json!("Default Dark+")));
}

#[test]
fn test_configuration_store_snapshot_missing_section() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let store = ConfigurationStore::new(path).unwrap();
    let snapshot = store.snapshot(Some("nonexistent"));
    assert!(snapshot.is_object());
    assert!(snapshot.as_object().unwrap().is_empty());
}

// ── DocumentManager with language detection ─────────────────────────────────

#[tokio::test]
async fn test_document_manager_new() {
    let dm = DocumentManager::new();
    let version = dm.get_document_version("test.rs").await;
    assert_eq!(version, 1);
}

#[tokio::test]
async fn test_document_manager_version_tracking() {
    let dm = DocumentManager::new();

    let v1 = dm.get_document_version("main.rs").await;
    assert_eq!(v1, 1);

    let v2 = dm.bump_document_version("main.rs").await;
    assert_eq!(v2, 2);

    let v3 = dm.bump_document_version("main.rs").await;
    assert_eq!(v3, 3);

    // Different document should have independent version
    let other_v = dm.get_document_version("other.py").await;
    assert_eq!(other_v, 1);
}

#[test]
fn test_detect_language_id_rust() {
    assert_eq!(detect_language_id("main.rs"), "rust");
    assert_eq!(detect_language_id("/home/user/src/lib.rs"), "rust");
    assert_eq!(detect_language_id("src/main.RS"), "rust"); // case insensitive
}

#[test]
fn test_detect_language_id_typescript_and_javascript() {
    assert_eq!(detect_language_id("app.ts"), "typescript");
    assert_eq!(detect_language_id("component.tsx"), "typescript");
    assert_eq!(detect_language_id("index.js"), "javascript");
    assert_eq!(detect_language_id("App.jsx"), "javascript");
}

#[test]
fn test_detect_language_id_common_languages() {
    assert_eq!(detect_language_id("script.py"), "python");
    assert_eq!(detect_language_id("main.go"), "go");
    assert_eq!(detect_language_id("App.java"), "java");
    assert_eq!(detect_language_id("main.cpp"), "cpp");
    assert_eq!(detect_language_id("app.rb"), "ruby");
    assert_eq!(detect_language_id("main.swift"), "swift");
    assert_eq!(detect_language_id("app.kt"), "kotlin");
    assert_eq!(detect_language_id("style.css"), "css");
    assert_eq!(detect_language_id("style.scss"), "css");
    assert_eq!(detect_language_id("style.less"), "css");
    assert_eq!(detect_language_id("index.html"), "html");
    assert_eq!(detect_language_id("index.htm"), "html");
    assert_eq!(detect_language_id("data.json"), "json");
    assert_eq!(detect_language_id("config.yaml"), "yaml");
    assert_eq!(detect_language_id("config.yml"), "yaml");
    assert_eq!(detect_language_id("Cargo.toml"), "toml");
    assert_eq!(detect_language_id("README.md"), "markdown");
    assert_eq!(detect_language_id("main.cs"), "csharp");
    assert_eq!(detect_language_id("app.php"), "php");
}

#[test]
fn test_detect_language_id_unknown() {
    assert_eq!(detect_language_id("file.xyz"), "plaintext");
    assert_eq!(detect_language_id("noext"), "plaintext");
    assert_eq!(detect_language_id(""), "plaintext");
}

#[tokio::test]
async fn test_document_manager_persist_and_open() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    let file_path_str = file_path.to_string_lossy().to_string();

    let dm = DocumentManager::new();

    // Persist a document
    let version = dm.persist_document(&file_path_str, "fn main() {}").await.unwrap();
    assert_eq!(version, 2); // Initial 1 + bump

    // Open it and verify content
    let doc = dm.open_text_document(&file_path_str).await.unwrap();
    assert_eq!(doc.get("text").unwrap().as_str(), Some("fn main() {}"));
    assert_eq!(doc.get("languageId").unwrap().as_str(), Some("rust"));
}

#[tokio::test]
async fn test_document_manager_open_nonexistent_file() {
    let dm = DocumentManager::new();
    let doc = dm.open_text_document("/nonexistent/path/file.rs").await.unwrap();
    // Nonexistent file should have empty text
    assert_eq!(doc.get("text").unwrap().as_str(), Some(""));
    assert_eq!(doc.get("languageId").unwrap().as_str(), Some("rust"));
}

// ── WorkspaceManager path/URI conversions ───────────────────────────────────

#[test]
fn test_workspace_path_from_uri() {
    assert_eq!(
        workspace_path_from_uri("file:///home/user/project"),
        PathBuf::from("/home/user/project")
    );
    assert_eq!(
        workspace_path_from_uri("file:///Users/test/code"),
        PathBuf::from("/Users/test/code")
    );
}

#[test]
fn test_workspace_uri_from_path() {
    let path = PathBuf::from("/home/user/project");
    assert_eq!(workspace_uri_from_path(&path), "file:///home/user/project");
}

#[test]
fn test_workspace_uri_path_roundtrip() {
    let original = "/home/user/myproject";
    let uri = workspace_uri_from_path(Path::new(original));
    let back = workspace_path_from_uri(&uri);
    assert_eq!(back, PathBuf::from(original));
}

#[tokio::test]
async fn test_workspace_manager_new() {
    let wm = WorkspaceManager::new();
    // No configurations yet
    assert!(wm
        .get_workspace_configuration("editor", None)
        .await
        .is_none());
}

#[tokio::test]
async fn test_workspace_manager_update_and_get_config() {
    let wm = WorkspaceManager::new();

    wm.update_workspace_configuration(
        "editor".to_string(),
        None,
        "fontSize".to_string(),
        json!(14),
    )
    .await
    .unwrap();

    let config = wm.get_workspace_configuration("editor", None).await;
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.section, "editor");
    assert_eq!(config.values.get("fontSize"), Some(&json!(14)));
}

#[tokio::test]
async fn test_workspace_manager_update_with_scope() {
    let wm = WorkspaceManager::new();

    wm.update_workspace_configuration(
        "editor".to_string(),
        Some("workspace".to_string()),
        "tabSize".to_string(),
        json!(4),
    )
    .await
    .unwrap();

    let config = wm.get_workspace_configuration("editor", Some("workspace")).await;
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.values.get("tabSize"), Some(&json!(4)));
}

#[tokio::test]
async fn test_workspace_manager_file_decorations() {
    let wm = WorkspaceManager::new();

    use dscode_session::FileDecoration;
    use dscode_session::FileDecorationProvider;

    // Register a decoration provider
    let provider = FileDecorationProvider {
        id: "git-provider".to_string(),
        owner: "git".to_string(),
    };
    wm.register_file_decoration_provider(provider).await.unwrap();

    // Update decorations
    let decorations = vec![FileDecoration {
        uri: "file:///project/main.rs".to_string(),
        badge: Some("M".to_string()),
        tooltip: Some("Modified".to_string()),
        color: Some("green".to_string()),
        propagate: false,
    }];
    wm.update_file_decorations("git-provider".to_string(), decorations)
        .await
        .unwrap();

    // Get decorations for a URI
    let found = wm.get_file_decorations("file:///project/main.rs").await;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].badge, Some("M".to_string()));
}

#[tokio::test]
async fn test_workspace_manager_clear_workspace_data() {
    let wm = WorkspaceManager::new();

    use dscode_session::FileDecorationProvider;

    let provider = FileDecorationProvider {
        id: "test-provider".to_string(),
        owner: "test-owner".to_string(),
    };
    wm.register_file_decoration_provider(provider).await.unwrap();

    // Clear data for the owner
    wm.clear_workspace_data("test-owner").await.unwrap();
}

// ── ExtensionContributes parsing from JSON ───────────────────────────────────

#[test]
fn test_extension_contributes_from_json_empty() {
    let ec = ExtensionContributes::from_json(&json!({}));
    assert!(ec.commands.is_empty());
    assert!(ec.languages.is_empty());
    assert!(ec.grammars.is_empty());
}

#[test]
fn test_extension_contributes_from_json_null() {
    let ec = ExtensionContributes::from_json(&json!(null));
    assert!(ec.commands.is_empty());
}

#[test]
fn test_extension_contributes_from_json_commands() {
    let value = json!({
        "commands": [
            { "command": "ext.hello", "title": "Hello" },
            { "command": "ext.goodbye", "title": "Goodbye", "category": "Social" }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);

    assert_eq!(ec.commands.len(), 2);
    assert_eq!(ec.commands[0].command, "ext.hello");
    assert_eq!(ec.commands[0].title, "Hello");
    assert_eq!(ec.commands[1].command, "ext.goodbye");
    assert_eq!(ec.commands[1].category, Some("Social".to_string()));
}

#[test]
fn test_extension_contributes_from_json_languages() {
    let value = json!({
        "languages": [
            { "id": "rust", "extensions": [".rs"] },
            { "id": "python", "extensions": [".py"], "aliases": ["Python3"] }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);

    assert_eq!(ec.languages.len(), 2);
    assert_eq!(ec.languages[0].id, "rust");
    assert_eq!(ec.languages[0].extensions, vec![".rs"]);
    assert_eq!(ec.languages[1].id, "python");
    assert_eq!(ec.languages[1].aliases, vec!["Python3"]);
}

#[test]
fn test_extension_contributes_from_json_keybindings() {
    let value = json!({
        "keybindings": [
            { "key": "ctrl+shift+p", "command": "ext.command", "mac": "cmd+shift+p" }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);

    assert_eq!(ec.keybindings.len(), 1);
    assert_eq!(ec.keybindings[0].key, "ctrl+shift+p");
    assert_eq!(ec.keybindings[0].command, "ext.command");
    assert_eq!(ec.keybindings[0].mac, Some("cmd+shift+p".to_string()));
}

#[test]
fn test_extension_contributes_from_json_menus() {
    let value = json!({
        "menus": {
            "editor/context": [
                { "command": "ext.format", "group": "1_modification" }
            ]
        }
    });
    let ec = ExtensionContributes::from_json(&value);

    assert!(ec.menus.contains_key("editor/context"));
    let items = ec.menus.get("editor/context").unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].command, "ext.format");
}

#[test]
fn test_extension_contributes_from_json_debuggers() {
    let value = json!({
        "debuggers": [
            {
                "type": "lldb",
                "label": "LLDB Debugger",
                "languages": ["rust", "cpp"]
            }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);

    assert_eq!(ec.debuggers.len(), 1);
    assert_eq!(ec.debuggers[0].r#type, "lldb");
    assert_eq!(ec.debuggers[0].label, "LLDB Debugger");
}

#[test]
fn test_extension_contributes_from_json_views() {
    let value = json!({
        "views": {
            "explorer": [
                { "id": "fileTree", "name": "Files" }
            ]
        }
    });
    let ec = ExtensionContributes::from_json(&value);

    assert!(ec.views.contains_key("explorer"));
    let views = ec.views.get("explorer").unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].id, "fileTree");
}

#[test]
fn test_extension_contributes_from_json_configuration() {
    let value = json!({
        "configuration": {
            "title": "My Extension",
            "properties": {
                "myExt.setting1": {
                    "type": "string",
                    "default": "hello",
                    "description": "A sample setting"
                }
            }
        }
    });
    let ec = ExtensionContributes::from_json(&value);

    assert!(ec.configuration.is_some());
    let config = ec.configuration.unwrap();
    assert_eq!(config.title, "My Extension");
    assert!(config.properties.contains_key("myExt.setting1"));
}

#[test]
fn test_extension_contributes_from_json_terminal_profiles() {
    let value = json!({
        "terminalProfiles": [
            { "id": "myShell", "title": "My Custom Shell" }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);

    assert_eq!(ec.terminal_profiles.len(), 1);
    assert_eq!(ec.terminal_profiles[0].id, "myShell");
    assert_eq!(ec.terminal_profiles[0].title, "My Custom Shell");
}

#[test]
fn test_extension_contributes_from_json_complex() {
    // A complex manifest with multiple contribution types
    let value = json!({
        "commands": [
            { "command": "ext.run", "title": "Run" }
        ],
        "languages": [
            { "id": "custom", "extensions": [".custom"] }
        ],
        "keybindings": [
            { "key": "f5", "command": "ext.run" }
        ],
        "menus": {
            "editor/title": [
                { "command": "ext.run" }
            ]
        },
        "configuration": {
            "title": "Custom Extension",
            "properties": {
                "custom.value": {
                    "type": "number",
                    "default": 42
                }
            }
        }
    });
    let ec = ExtensionContributes::from_json(&value);

    assert_eq!(ec.commands.len(), 1);
    assert_eq!(ec.languages.len(), 1);
    assert_eq!(ec.keybindings.len(), 1);
    assert!(ec.menus.contains_key("editor/title"));
    assert!(ec.configuration.is_some());
}

#[test]
fn test_extension_contributes_all_command_ids() {
    let value = json!({
        "commands": [
            { "command": "ext.cmd1", "title": "Command 1" },
            { "command": "ext.cmd2", "title": "Command 2" },
            { "command": "ext.cmd3", "title": "Command 3" }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);

    let ids = ec.all_command_ids();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"ext.cmd1".to_string()));
    assert!(ids.contains(&"ext.cmd2".to_string()));
    assert!(ids.contains(&"ext.cmd3".to_string()));
}

#[test]
fn test_extension_contributes_activation_events() {
    let value = json!({
        "commands": [
            { "command": "ext.run", "title": "Run" }
        ],
        "languages": [
            { "id": "rust", "extensions": [".rs"] }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);

    let events = ec.activation_events();
    assert!(events.contains(&"onCommand:ext.run".to_string()));
    assert!(events.contains(&"onLanguage:rust".to_string()));
}

#[test]
fn test_extension_contributes_language_ids_for_extension() {
    let value = json!({
        "languages": [
            { "id": "rust", "extensions": [".rs"] },
            { "id": "python", "extensions": ["py"] }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);

    assert_eq!(ec.language_ids_for_extension(".rs"), vec!["rust"]);
    assert_eq!(ec.language_ids_for_extension("py"), vec!["python"]);
}

// ── SessionLifecycle state machine transitions ──────────────────────────────

#[test]
fn test_session_lifecycle_valid_transitions() {
    // Uninitialized -> Initializing
    assert!(SessionLifecycle::Uninitialized.can_transition_to(SessionLifecycle::Initializing));

    // Initializing -> Ready
    assert!(SessionLifecycle::Initializing.can_transition_to(SessionLifecycle::Ready));

    // Initializing -> Error
    assert!(SessionLifecycle::Initializing.can_transition_to(SessionLifecycle::Error));

    // Ready -> ShuttingDown
    assert!(SessionLifecycle::Ready.can_transition_to(SessionLifecycle::ShuttingDown));

    // ShuttingDown -> Shutdown
    assert!(SessionLifecycle::ShuttingDown.can_transition_to(SessionLifecycle::Shutdown));

    // Error -> Initializing (retry)
    assert!(SessionLifecycle::Error.can_transition_to(SessionLifecycle::Initializing));

    // Error -> ShuttingDown
    assert!(SessionLifecycle::Error.can_transition_to(SessionLifecycle::ShuttingDown));
}

#[test]
fn test_session_lifecycle_invalid_transitions() {
    // Cannot skip Initializing
    assert!(!SessionLifecycle::Uninitialized.can_transition_to(SessionLifecycle::Ready));
    assert!(!SessionLifecycle::Uninitialized.can_transition_to(SessionLifecycle::ShuttingDown));
    assert!(!SessionLifecycle::Uninitialized.can_transition_to(SessionLifecycle::Shutdown));

    // Cannot go back from Ready
    assert!(!SessionLifecycle::Ready.can_transition_to(SessionLifecycle::Initializing));
    assert!(!SessionLifecycle::Ready.can_transition_to(SessionLifecycle::Error));

    // Cannot go back from Shutdown
    assert!(!SessionLifecycle::Shutdown.can_transition_to(SessionLifecycle::Ready));
    assert!(!SessionLifecycle::Shutdown.can_transition_to(SessionLifecycle::Initializing));
}

#[test]
fn test_session_lifecycle_equality() {
    assert_eq!(SessionLifecycle::Ready, SessionLifecycle::Ready);
    assert_ne!(SessionLifecycle::Ready, SessionLifecycle::Initializing);
    assert_ne!(SessionLifecycle::Uninitialized, SessionLifecycle::Shutdown);
}

#[test]
fn test_session_lifecycle_full_lifecycle() {
    // Simulate a full lifecycle: Uninitialized -> Initializing -> Ready -> ShuttingDown -> Shutdown
    let mut state = SessionLifecycle::Uninitialized;

    assert!(state.can_transition_to(SessionLifecycle::Initializing));
    state = SessionLifecycle::Initializing;

    assert!(state.can_transition_to(SessionLifecycle::Ready));
    state = SessionLifecycle::Ready;

    assert!(state.can_transition_to(SessionLifecycle::ShuttingDown));
    state = SessionLifecycle::ShuttingDown;

    assert!(state.can_transition_to(SessionLifecycle::Shutdown));
    state = SessionLifecycle::Shutdown;

    // Terminal state
    assert_eq!(state, SessionLifecycle::Shutdown);
}

#[test]
fn test_session_lifecycle_error_recovery() {
    // Simulate: Uninitialized -> Initializing -> Error -> Initializing -> Ready
    let mut state = SessionLifecycle::Uninitialized;

    state = SessionLifecycle::Initializing;
    state = SessionLifecycle::Error;

    // Recovery: Error -> Initializing
    assert!(state.can_transition_to(SessionLifecycle::Initializing));
    state = SessionLifecycle::Initializing;

    // Now can proceed to Ready
    assert!(state.can_transition_to(SessionLifecycle::Ready));
    state = SessionLifecycle::Ready;

    assert_eq!(state, SessionLifecycle::Ready);
}

#[test]
fn test_session_lifecycle_error_shutdown() {
    // Simulate: Error -> ShuttingDown -> Shutdown
    let state = SessionLifecycle::Error;
    assert!(state.can_transition_to(SessionLifecycle::ShuttingDown));

    let state = SessionLifecycle::ShuttingDown;
    assert!(state.can_transition_to(SessionLifecycle::Shutdown));
}

// ── Cross-component integration ─────────────────────────────────────────────

#[tokio::test]
async fn test_configuration_and_document_manager_integration() {
    // ConfigurationStore configures editor settings, DocumentManager tracks documents
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("settings.json");

    let mut config = ConfigurationStore::new(config_path.clone()).unwrap();
    config
        .update(Some("editor"), "fontSize", json!(14))
        .unwrap();
    config
        .update(Some("editor"), "tabSize", json!(4))
        .unwrap();

    let dm = DocumentManager::new();

    // Create and persist a document
    let file_path = dir.path().join("main.rs");
    let file_path_str = file_path.to_string_lossy().to_string();
    dm.persist_document(&file_path_str, "fn main() {\n    println!(\"hello\");\n}\n")
        .await
        .unwrap();

    // Read it back
    let doc = dm.open_text_document(&file_path_str).await.unwrap();
    assert_eq!(doc.get("languageId").unwrap().as_str(), Some("rust"));

    // Verify config was persisted alongside the document
    let config = ConfigurationStore::new(config_path).unwrap();
    let editor = config.snapshot(Some("editor"));
    assert_eq!(editor.get("fontSize"), Some(&json!(14)));
}

#[test]
fn test_workspace_and_contributes_integration() {
    // WorkspaceManager provides path/URI conversions, ExtensionContributes
    // declares language contributions -- these work together
    let project_path = "/home/user/project";
    let uri = workspace_uri_from_path(Path::new(project_path));
    assert_eq!(uri, "file:///home/user/project");

    let back = workspace_path_from_uri(&uri);
    assert_eq!(back, PathBuf::from(project_path));

    // ExtensionContributes can declare languages for the project
    let value = json!({
        "languages": [
            { "id": "rust", "extensions": [".rs"] }
        ]
    });
    let ec = ExtensionContributes::from_json(&value);
    let rust_ids = ec.language_ids_for_extension(".rs");
    assert_eq!(rust_ids, vec!["rust"]);

    // Language detection matches the contributed language
    let detected = detect_language_id(&format!("{}/main.rs", project_path));
    assert_eq!(detected, "rust");
}

#[tokio::test]
async fn test_lifecycle_and_workspace_integration() {
    // Session lifecycle transitions with workspace operations
    let lifecycle = SessionLifecycle::Uninitialized;

    // Can only start initializing from Uninitialized
    assert!(lifecycle.can_transition_to(SessionLifecycle::Initializing));

    // After initializing, workspace can be set up
    let wm = WorkspaceManager::new();
    // Workspace is empty before initialization completes
    assert!(wm
        .get_workspace_configuration("editor", None)
        .await
        .is_none());
}