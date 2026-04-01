//! Headless session example (no Tauri required).
//!
//! Demonstrates using configuration management independently of the UI.

use dscode_session::ConfigurationStore;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration store at a temp path
    let tmp = std::env::temp_dir()
        .join("dscode-headless-example")
        .join("settings.json");
    std::fs::create_dir_all(tmp.parent().unwrap())?;
    let mut store = ConfigurationStore::new(tmp)?;

    // Read configuration snapshot
    let editor = store.snapshot(Some("editor"));
    println!("Editor config: {:?}", editor);

    // Update a configuration value
    store.update(Some("editor"), "tabSize", serde_json::json!(4))?;
    println!("Updated tabSize to 4.");

    // Read back the updated value
    let updated = store.snapshot(Some("editor"));
    println!("Updated editor config: {:?}", updated);

    // Update a root-level key
    store.update(None, "theme", serde_json::json!("dark"))?;
    let root = store.snapshot(None);
    println!("Root config keys: {:?}", root.as_object().map(|o| o.keys().collect::<Vec<_>>()));

    Ok(())
}
