use serde_json::Value;
use crate::commands::{Keybinding, KeybindingRegistry};
use tauri::State;

/// Parse keybinding contributions from extension package.json and register them
#[tauri::command]
pub async fn parse_and_register_extension_keybindings(
    extension_id: String,
    package_json: Value,
    keybinding_registry: State<'_, KeybindingRegistry>,
) -> Result<usize, String> {
    let contributes = package_json.get("contributes")
        .ok_or("No contributes section in package.json")?;

    let keybindings = contributes.get("keybindings");
    if keybindings.is_none() {
        // No keybindings contributed, that's fine
        return Ok(0);
    }

    let keybindings_array = keybindings.unwrap().as_array()
        .ok_or("Keybindings must be an array")?;

    let mut count = 0;

    for kb_value in keybindings_array {
        let kb_obj = kb_value.as_object()
            .ok_or("Keybinding must be an object")?;

        // Extract command
        let command = kb_obj.get("command")
            .and_then(|v| v.as_str())
            .ok_or("Keybinding must have a command")?
            .to_string();

        // Extract key
        let key = kb_obj.get("key")
            .and_then(|v| v.as_str())
            .ok_or("Keybinding must have a key")?
            .to_string();

        // Extract optional fields
        let when = kb_obj.get("when")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Platform-specific key (mac, win, linux)
        let mut platform = None;
        let mut platform_key = key.clone();

        // Check for platform-specific keys
        if let Some(mac_key) = kb_obj.get("mac").and_then(|v| v.as_str()) {
            #[cfg(target_os = "macos")]
            {
                platform_key = mac_key.to_string();
                platform = Some("macos".to_string());
            }
        }

        if let Some(win_key) = kb_obj.get("win").and_then(|v| v.as_str()) {
            #[cfg(target_os = "windows")]
            {
                platform_key = win_key.to_string();
                platform = Some("windows".to_string());
            }
        }

        if let Some(linux_key) = kb_obj.get("linux").and_then(|v| v.as_str()) {
            #[cfg(target_os = "linux")]
            {
                platform_key = linux_key.to_string();
                platform = Some("linux".to_string());
            }
        }

        // Extract args
        let args = kb_obj.get("args").cloned();

        // Create Keybinding
        let keybinding = Keybinding {
            command,
            key: platform_key,
            when,
            platform,
            owner: extension_id.clone(),
            args,
        };

        // Register with KeybindingRegistry
        keybinding_registry.register_keybinding(keybinding)?;
        count += 1;
    }

    Ok(count)
}
