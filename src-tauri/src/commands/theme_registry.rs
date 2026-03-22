use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};

/// Color theme type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeType {
    Light,
    Dark,
    HighContrast,
    HighContrastLight,
}

/// Color theme contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTheme {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub theme_type: ThemeType,
    pub colors: HashMap<String, String>,
    pub token_colors: Vec<TokenColor>,
}

/// Token color rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenColor {
    pub name: Option<String>,
    pub scope: Vec<String>,
    pub settings: TokenColorSettings,
}

/// Token color settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenColorSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<String>,
}

/// Icon theme contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconTheme {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub icon_definitions: HashMap<String, IconDefinition>,
    pub file_associations: HashMap<String, String>,
    pub folder_associations: HashMap<String, String>,
    pub file_extensions: HashMap<String, String>,
    pub language_ids: HashMap<String, String>,
}

/// Icon definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconDefinition {
    pub icon_path: String,
}

/// Product icon theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductIconTheme {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub icon_definitions: HashMap<String, ProductIconDefinition>,
}

/// Product icon definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductIconDefinition {
    pub font_character: String,
    pub font_color: Option<String>,
}

/// Theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub active_color_theme: Option<String>,
    pub active_icon_theme: Option<String>,
    pub active_product_icon_theme: Option<String>,
}

/// Theme change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeChangeEvent {
    pub theme_id: String,
    pub theme_type: String, // "color", "icon", or "product-icon"
}

/// Theme registry
pub struct ThemeRegistry {
    color_themes: Arc<RwLock<HashMap<String, ColorTheme>>>,
    icon_themes: Arc<RwLock<HashMap<String, IconTheme>>>,
    product_icon_themes: Arc<RwLock<HashMap<String, ProductIconTheme>>>,
    theme_settings: Arc<RwLock<ThemeSettings>>,
    app_handle: AppHandle,
}

impl ThemeRegistry {
    /// Create a new theme registry
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            color_themes: Arc::new(RwLock::new(HashMap::new())),
            icon_themes: Arc::new(RwLock::new(HashMap::new())),
            product_icon_themes: Arc::new(RwLock::new(HashMap::new())),
            theme_settings: Arc::new(RwLock::new(ThemeSettings {
                active_color_theme: None,
                active_icon_theme: None,
                active_product_icon_theme: None,
            })),
            app_handle,
        }
    }

    // ===== Color Themes =====

    /// Register color theme
    pub fn register_color_theme(&self, theme: ColorTheme) -> Result<String, String> {
        let mut themes = self.color_themes.write().map_err(|e| e.to_string())?;

        let id = theme.id.clone();
        themes.insert(id.clone(), theme.clone());

        println!(
            "[Theme] Registered color theme: {} ({})",
            theme.label, theme.id
        );

        Ok(id)
    }

    /// Unregister color theme
    pub fn unregister_color_theme(&self, theme_id: &str) -> Result<(), String> {
        let mut themes = self.color_themes.write().map_err(|e| e.to_string())?;

        themes.remove(theme_id).ok_or_else(|| {
            format!("Color theme not found: {}", theme_id)
        })?;

        println!("[Theme] Unregistered color theme: {}", theme_id);

        Ok(())
    }

    /// Get color theme
    pub fn get_color_theme(&self, theme_id: &str) -> Result<ColorTheme, String> {
        let themes = self.color_themes.read().map_err(|e| e.to_string())?;

        themes
            .get(theme_id)
            .cloned()
            .ok_or_else(|| format!("Color theme not found: {}", theme_id))
    }

    /// Get all color themes
    pub fn get_all_color_themes(&self) -> Vec<ColorTheme> {
        self.color_themes.read().unwrap().values().cloned().collect()
    }

    /// Get color themes by type
    pub fn get_color_themes_by_type(&self, theme_type: ThemeType) -> Vec<ColorTheme> {
        let themes = self.color_themes.read().unwrap();
        themes
            .values()
            .filter(|t| t.theme_type == theme_type)
            .cloned()
            .collect()
    }

    /// Set active color theme
    pub fn set_active_color_theme(&self, theme_id: String) -> Result<(), String> {
        // Verify theme exists
        let themes = self.color_themes.read().map_err(|e| e.to_string())?;
        if !themes.contains_key(&theme_id) {
            return Err(format!("Color theme not found: {}", theme_id));
        }
        drop(themes);

        // Update settings
        let mut settings = self.theme_settings.write().map_err(|e| e.to_string())?;
        settings.active_color_theme = Some(theme_id.clone());
        drop(settings);

        println!("[Theme] Set active color theme: {}", theme_id);

        // Emit event
        if let Err(e) = self.app_handle.emit(
            "theme-changed",
            ThemeChangeEvent {
                theme_id,
                theme_type: "color".to_string(),
            },
        ) {
            eprintln!("[Theme] Failed to emit theme changed event: {}", e);
        }

        Ok(())
    }

    /// Get active color theme
    pub fn get_active_color_theme(&self) -> Option<ColorTheme> {
        let settings = self.theme_settings.read().unwrap();
        let theme_id = settings.active_color_theme.clone()?;
        drop(settings);

        self.get_color_theme(&theme_id).ok()
    }

    // ===== Icon Themes =====

    /// Register icon theme
    pub fn register_icon_theme(&self, theme: IconTheme) -> Result<String, String> {
        let mut themes = self.icon_themes.write().map_err(|e| e.to_string())?;

        let id = theme.id.clone();
        themes.insert(id.clone(), theme.clone());

        println!(
            "[Theme] Registered icon theme: {} ({})",
            theme.label, theme.id
        );

        Ok(id)
    }

    /// Unregister icon theme
    pub fn unregister_icon_theme(&self, theme_id: &str) -> Result<(), String> {
        let mut themes = self.icon_themes.write().map_err(|e| e.to_string())?;

        themes.remove(theme_id).ok_or_else(|| {
            format!("Icon theme not found: {}", theme_id)
        })?;

        println!("[Theme] Unregistered icon theme: {}", theme_id);

        Ok(())
    }

    /// Get icon theme
    pub fn get_icon_theme(&self, theme_id: &str) -> Result<IconTheme, String> {
        let themes = self.icon_themes.read().map_err(|e| e.to_string())?;

        themes
            .get(theme_id)
            .cloned()
            .ok_or_else(|| format!("Icon theme not found: {}", theme_id))
    }

    /// Get all icon themes
    pub fn get_all_icon_themes(&self) -> Vec<IconTheme> {
        self.icon_themes.read().unwrap().values().cloned().collect()
    }

    /// Set active icon theme
    pub fn set_active_icon_theme(&self, theme_id: String) -> Result<(), String> {
        // Verify theme exists
        let themes = self.icon_themes.read().map_err(|e| e.to_string())?;
        if !themes.contains_key(&theme_id) {
            return Err(format!("Icon theme not found: {}", theme_id));
        }
        drop(themes);

        // Update settings
        let mut settings = self.theme_settings.write().map_err(|e| e.to_string())?;
        settings.active_icon_theme = Some(theme_id.clone());
        drop(settings);

        println!("[Theme] Set active icon theme: {}", theme_id);

        // Emit event
        if let Err(e) = self.app_handle.emit(
            "theme-changed",
            ThemeChangeEvent {
                theme_id,
                theme_type: "icon".to_string(),
            },
        ) {
            eprintln!("[Theme] Failed to emit theme changed event: {}", e);
        }

        Ok(())
    }

    /// Get active icon theme
    pub fn get_active_icon_theme(&self) -> Option<IconTheme> {
        let settings = self.theme_settings.read().unwrap();
        let theme_id = settings.active_icon_theme.clone()?;
        drop(settings);

        self.get_icon_theme(&theme_id).ok()
    }

    // ===== Product Icon Themes =====

    /// Register product icon theme
    pub fn register_product_icon_theme(&self, theme: ProductIconTheme) -> Result<String, String> {
        let mut themes = self.product_icon_themes.write().map_err(|e| e.to_string())?;

        let id = theme.id.clone();
        themes.insert(id.clone(), theme.clone());

        println!(
            "[Theme] Registered product icon theme: {} ({})",
            theme.label, theme.id
        );

        Ok(id)
    }

    /// Unregister product icon theme
    pub fn unregister_product_icon_theme(&self, theme_id: &str) -> Result<(), String> {
        let mut themes = self.product_icon_themes.write().map_err(|e| e.to_string())?;

        themes.remove(theme_id).ok_or_else(|| {
            format!("Product icon theme not found: {}", theme_id)
        })?;

        println!("[Theme] Unregistered product icon theme: {}", theme_id);

        Ok(())
    }

    /// Get product icon theme
    pub fn get_product_icon_theme(&self, theme_id: &str) -> Result<ProductIconTheme, String> {
        let themes = self.product_icon_themes.read().map_err(|e| e.to_string())?;

        themes
            .get(theme_id)
            .cloned()
            .ok_or_else(|| format!("Product icon theme not found: {}", theme_id))
    }

    /// Get all product icon themes
    pub fn get_all_product_icon_themes(&self) -> Vec<ProductIconTheme> {
        self.product_icon_themes.read().unwrap().values().cloned().collect()
    }

    /// Set active product icon theme
    pub fn set_active_product_icon_theme(&self, theme_id: String) -> Result<(), String> {
        // Verify theme exists
        let themes = self.product_icon_themes.read().map_err(|e| e.to_string())?;
        if !themes.contains_key(&theme_id) {
            return Err(format!("Product icon theme not found: {}", theme_id));
        }
        drop(themes);

        // Update settings
        let mut settings = self.theme_settings.write().map_err(|e| e.to_string())?;
        settings.active_product_icon_theme = Some(theme_id.clone());
        drop(settings);

        println!("[Theme] Set active product icon theme: {}", theme_id);

        // Emit event
        if let Err(e) = self.app_handle.emit(
            "theme-changed",
            ThemeChangeEvent {
                theme_id,
                theme_type: "product-icon".to_string(),
            },
        ) {
            eprintln!("[Theme] Failed to emit theme changed event: {}", e);
        }

        Ok(())
    }

    /// Get active product icon theme
    pub fn get_active_product_icon_theme(&self) -> Option<ProductIconTheme> {
        let settings = self.theme_settings.read().unwrap();
        let theme_id = settings.active_product_icon_theme.clone()?;
        drop(settings);

        self.get_product_icon_theme(&theme_id).ok()
    }

    /// Get theme settings
    pub fn get_theme_settings(&self) -> ThemeSettings {
        self.theme_settings.read().unwrap().clone()
    }

    /// Clear all theme data for an owner
    pub fn clear_theme_data(&self, owner: &str) {
        // Clear color themes
        {
            let mut themes = self.color_themes.write().unwrap();
            let before = themes.len();
            themes.retain(|_, t| t.owner != owner);
            let removed = before - themes.len();
            if removed > 0 {
                println!(
                    "[Theme] Cleared {} color theme(s) for owner: {}",
                    removed, owner
                );
            }
        }

        // Clear icon themes
        {
            let mut themes = self.icon_themes.write().unwrap();
            let before = themes.len();
            themes.retain(|_, t| t.owner != owner);
            let removed = before - themes.len();
            if removed > 0 {
                println!(
                    "[Theme] Cleared {} icon theme(s) for owner: {}",
                    removed, owner
                );
            }
        }

        // Clear product icon themes
        {
            let mut themes = self.product_icon_themes.write().unwrap();
            let before = themes.len();
            themes.retain(|_, t| t.owner != owner);
            let removed = before - themes.len();
            if removed > 0 {
                println!(
                    "[Theme] Cleared {} product icon theme(s) for owner: {}",
                    removed, owner
                );
            }
        }
    }
}
