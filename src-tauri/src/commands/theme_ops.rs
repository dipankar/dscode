use super::theme_registry::*;
use tauri::State;

// ===== Color Themes =====

/// Register color theme
#[tauri::command]
pub async fn register_color_theme(
    theme: ColorTheme, registry: State<'_, ThemeRegistry>,
) -> Result<String, String> {
    registry.register_color_theme(theme).map_err(|e| e.to_string())
}

/// Unregister color theme
#[tauri::command]
pub async fn unregister_color_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<(), String> {
    registry.unregister_color_theme(&theme_id).map_err(|e| e.to_string())
}

/// Get color theme
#[tauri::command]
pub async fn get_color_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<ColorTheme, String> {
    registry.get_color_theme(&theme_id).map_err(|e| e.to_string())
}

/// Get all color themes
#[tauri::command]
pub async fn get_all_color_themes(
    registry: State<'_, ThemeRegistry>,
) -> Result<Vec<ColorTheme>, String> {
    Ok(registry.get_all_color_themes())
}

/// Get color themes by type
#[tauri::command]
pub async fn get_color_themes_by_type(
    theme_type: ThemeType, registry: State<'_, ThemeRegistry>,
) -> Result<Vec<ColorTheme>, String> {
    Ok(registry.get_color_themes_by_type(theme_type))
}

/// Set active color theme
#[tauri::command]
pub async fn set_active_color_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<(), String> {
    registry.set_active_color_theme(theme_id).map_err(|e| e.to_string())
}

/// Get active color theme
#[tauri::command]
pub async fn get_active_color_theme(
    registry: State<'_, ThemeRegistry>,
) -> Result<Option<ColorTheme>, String> {
    Ok(registry.get_active_color_theme())
}

// ===== Icon Themes =====

/// Register icon theme
#[tauri::command]
pub async fn register_icon_theme(
    theme: IconTheme, registry: State<'_, ThemeRegistry>,
) -> Result<String, String> {
    registry.register_icon_theme(theme).map_err(|e| e.to_string())
}

/// Unregister icon theme
#[tauri::command]
pub async fn unregister_icon_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<(), String> {
    registry.unregister_icon_theme(&theme_id).map_err(|e| e.to_string())
}

/// Get icon theme
#[tauri::command]
pub async fn get_icon_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<IconTheme, String> {
    registry.get_icon_theme(&theme_id).map_err(|e| e.to_string())
}

/// Get all icon themes
#[tauri::command]
pub async fn get_all_icon_themes(
    registry: State<'_, ThemeRegistry>,
) -> Result<Vec<IconTheme>, String> {
    Ok(registry.get_all_icon_themes())
}

/// Set active icon theme
#[tauri::command]
pub async fn set_active_icon_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<(), String> {
    registry.set_active_icon_theme(theme_id).map_err(|e| e.to_string())
}

/// Get active icon theme
#[tauri::command]
pub async fn get_active_icon_theme(
    registry: State<'_, ThemeRegistry>,
) -> Result<Option<IconTheme>, String> {
    Ok(registry.get_active_icon_theme())
}

// ===== Product Icon Themes =====

/// Register product icon theme
#[tauri::command]
pub async fn register_product_icon_theme(
    theme: ProductIconTheme, registry: State<'_, ThemeRegistry>,
) -> Result<String, String> {
    registry.register_product_icon_theme(theme).map_err(|e| e.to_string())
}

/// Unregister product icon theme
#[tauri::command]
pub async fn unregister_product_icon_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<(), String> {
    registry.unregister_product_icon_theme(&theme_id).map_err(|e| e.to_string())
}

/// Get product icon theme
#[tauri::command]
pub async fn get_product_icon_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<ProductIconTheme, String> {
    registry.get_product_icon_theme(&theme_id).map_err(|e| e.to_string())
}

/// Get all product icon themes
#[tauri::command]
pub async fn get_all_product_icon_themes(
    registry: State<'_, ThemeRegistry>,
) -> Result<Vec<ProductIconTheme>, String> {
    Ok(registry.get_all_product_icon_themes())
}

/// Set active product icon theme
#[tauri::command]
pub async fn set_active_product_icon_theme(
    theme_id: String, registry: State<'_, ThemeRegistry>,
) -> Result<(), String> {
    registry.set_active_product_icon_theme(theme_id).map_err(|e| e.to_string())
}

/// Get active product icon theme
#[tauri::command]
pub async fn get_active_product_icon_theme(
    registry: State<'_, ThemeRegistry>,
) -> Result<Option<ProductIconTheme>, String> {
    Ok(registry.get_active_product_icon_theme())
}

// ===== Theme Settings =====

/// Get theme settings
#[tauri::command]
pub async fn get_theme_settings(
    registry: State<'_, ThemeRegistry>,
) -> Result<ThemeSettings, String> {
    Ok(registry.get_theme_settings())
}

/// Clear theme data for owner
#[tauri::command]
pub async fn clear_theme_data(
    owner: String, registry: State<'_, ThemeRegistry>,
) -> Result<(), String> {
    registry.clear_theme_data(&owner);
    Ok(())
}