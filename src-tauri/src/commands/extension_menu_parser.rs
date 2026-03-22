use serde_json::Value;
use crate::commands::{MenuItem, MenuRegistry};
use tauri::State;

/// Parse menu contributions from extension package.json and register them
#[tauri::command]
pub async fn parse_and_register_extension_menus(
    extension_id: String,
    package_json: Value,
    menu_registry: State<'_, MenuRegistry>,
) -> Result<usize, String> {
    let contributes = package_json.get("contributes")
        .ok_or("No contributes section in package.json")?;

    let menus = contributes.get("menus");
    if menus.is_none() {
        // No menus contributed, that's fine
        return Ok(0);
    }

    let menus_obj = menus.unwrap().as_object()
        .ok_or("Menus must be an object")?;

    let mut count = 0;

    // Iterate through menu locations (editor/context, explorer/context, etc.)
    for (location, items) in menus_obj {
        let items_array = items.as_array()
            .ok_or(format!("Menu items for {} must be an array", location))?;

        for item_value in items_array {
            let item_obj = item_value.as_object()
                .ok_or("Menu item must be an object")?;

            // Extract command
            let command = item_obj.get("command")
                .and_then(|v| v.as_str())
                .ok_or("Menu item must have a command")?
                .to_string();

            // Extract optional fields
            let when = item_obj.get("when")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let group = item_obj.get("group")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let title = item_obj.get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let icon = item_obj.get("icon")
                .and_then(|v| {
                    // Icon can be a string or an object with light/dark themes
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(obj) = v.as_object() {
                        obj.get("dark")
                            .or_else(|| obj.get("light"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                });

            let alt = item_obj.get("alt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Create MenuItem
            let menu_item = MenuItem {
                command,
                location: Some(location.clone()),
                when,
                group,
                title,
                icon,
                owner: extension_id.clone(),
                alt,
            };

            // Register with MenuRegistry
            menu_registry.register_menu_item(menu_item)?;
            count += 1;
        }
    }

    Ok(count)
}

/// Parse command contributions and extract titles for commands
/// This helps us get better labels for commands in menus
#[tauri::command]
pub async fn parse_command_contributions(
    package_json: Value,
) -> Result<Vec<CommandContribution>, String> {
    let contributes = package_json.get("contributes")
        .ok_or("No contributes section in package.json")?;

    let commands = contributes.get("commands");
    if commands.is_none() {
        return Ok(Vec::new());
    }

    let commands_array = commands.unwrap().as_array()
        .ok_or("Commands must be an array")?;

    let mut contributions = Vec::new();

    for cmd_value in commands_array {
        let cmd_obj = cmd_value.as_object()
            .ok_or("Command must be an object")?;

        let command_id = cmd_obj.get("command")
            .and_then(|v| v.as_str())
            .ok_or("Command must have a command field")?
            .to_string();

        let title = cmd_obj.get("title")
            .and_then(|v| v.as_str())
            .ok_or("Command must have a title")?
            .to_string();

        let category = cmd_obj.get("category")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let icon = cmd_obj.get("icon")
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(obj) = v.as_object() {
                    obj.get("dark")
                        .or_else(|| obj.get("light"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            });

        contributions.push(CommandContribution {
            command: command_id,
            title,
            category,
            icon,
        });
    }

    Ok(contributions)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandContribution {
    pub command: String,
    pub title: String,
    pub category: Option<String>,
    pub icon: Option<String>,
}
