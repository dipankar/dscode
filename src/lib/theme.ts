import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export enum ThemeType {
  Light = 'light',
  Dark = 'dark',
  HighContrast = 'highcontrast',
  HighContrastLight = 'highcontrastlight',
}

export interface TokenColor {
  name?: string;
  scope: string[];
  settings: TokenColorSettings;
}

export interface TokenColorSettings {
  foreground?: string;
  background?: string;
  font_style?: string;
}

export interface ColorTheme {
  id: string;
  label: string;
  owner: string;
  theme_type: ThemeType;
  colors: Record<string, string>;
  token_colors: TokenColor[];
}

export interface IconDefinition {
  icon_path: string;
}

export interface IconTheme {
  id: string;
  label: string;
  owner: string;
  icon_definitions: Record<string, IconDefinition>;
  file_associations: Record<string, string>;
  folder_associations: Record<string, string>;
  file_extensions: Record<string, string>;
  language_ids: Record<string, string>;
}

export interface ProductIconDefinition {
  font_character: string;
  font_color?: string;
}

export interface ProductIconTheme {
  id: string;
  label: string;
  owner: string;
  icon_definitions: Record<string, ProductIconDefinition>;
}

export interface ThemeSettings {
  active_color_theme?: string;
  active_icon_theme?: string;
  active_product_icon_theme?: string;
}

export interface ThemeChangeEvent {
  theme_id: string;
  theme_type: string; // "color", "icon", or "product-icon"
}

/**
 * Theme API manager
 */
export class ThemeManager {
  private onDidChangeThemeCallbacks: Array<(event: ThemeChangeEvent) => void> = [];

  constructor() {}

  /**
   * Initialize theme manager
   */
  async initialize(): Promise<void> {
    // Listen for theme changes
    await listen<ThemeChangeEvent>('theme-changed', (event) => {
      this.notifyThemeChanged(event.payload);
    });

    console.log('[Theme] Initialized');
  }

  // ===== Color Themes =====

  /**
   * Register color theme
   */
  async registerColorTheme(theme: ColorTheme): Promise<string> {
    return await invoke<string>('register_color_theme', { theme });
  }

  /**
   * Unregister color theme
   */
  async unregisterColorTheme(themeId: string): Promise<void> {
    await invoke('unregister_color_theme', { themeId });
  }

  /**
   * Get color theme
   */
  async getColorTheme(themeId: string): Promise<ColorTheme | null> {
    try {
      return await invoke<ColorTheme>('get_color_theme', { themeId });
    } catch {
      return null;
    }
  }

  /**
   * Get all color themes
   */
  async getAllColorThemes(): Promise<ColorTheme[]> {
    return await invoke<ColorTheme[]>('get_all_color_themes');
  }

  /**
   * Get color themes by type
   */
  async getColorThemesByType(themeType: ThemeType): Promise<ColorTheme[]> {
    return await invoke<ColorTheme[]>('get_color_themes_by_type', { themeType });
  }

  /**
   * Set active color theme
   */
  async setActiveColorTheme(themeId: string): Promise<void> {
    await invoke('set_active_color_theme', { themeId });
  }

  /**
   * Get active color theme
   */
  async getActiveColorTheme(): Promise<ColorTheme | null> {
    return await invoke<ColorTheme | null>('get_active_color_theme');
  }

  // ===== Icon Themes =====

  /**
   * Register icon theme
   */
  async registerIconTheme(theme: IconTheme): Promise<string> {
    return await invoke<string>('register_icon_theme', { theme });
  }

  /**
   * Unregister icon theme
   */
  async unregisterIconTheme(themeId: string): Promise<void> {
    await invoke('unregister_icon_theme', { themeId });
  }

  /**
   * Get icon theme
   */
  async getIconTheme(themeId: string): Promise<IconTheme | null> {
    try {
      return await invoke<IconTheme>('get_icon_theme', { themeId });
    } catch {
      return null;
    }
  }

  /**
   * Get all icon themes
   */
  async getAllIconThemes(): Promise<IconTheme[]> {
    return await invoke<IconTheme[]>('get_all_icon_themes');
  }

  /**
   * Set active icon theme
   */
  async setActiveIconTheme(themeId: string): Promise<void> {
    await invoke('set_active_icon_theme', { themeId });
  }

  /**
   * Get active icon theme
   */
  async getActiveIconTheme(): Promise<IconTheme | null> {
    return await invoke<IconTheme | null>('get_active_icon_theme');
  }

  // ===== Product Icon Themes =====

  /**
   * Register product icon theme
   */
  async registerProductIconTheme(theme: ProductIconTheme): Promise<string> {
    return await invoke<string>('register_product_icon_theme', { theme });
  }

  /**
   * Unregister product icon theme
   */
  async unregisterProductIconTheme(themeId: string): Promise<void> {
    await invoke('unregister_product_icon_theme', { themeId });
  }

  /**
   * Get product icon theme
   */
  async getProductIconTheme(themeId: string): Promise<ProductIconTheme | null> {
    try {
      return await invoke<ProductIconTheme>('get_product_icon_theme', { themeId });
    } catch {
      return null;
    }
  }

  /**
   * Get all product icon themes
   */
  async getAllProductIconThemes(): Promise<ProductIconTheme[]> {
    return await invoke<ProductIconTheme[]>('get_all_product_icon_themes');
  }

  /**
   * Set active product icon theme
   */
  async setActiveProductIconTheme(themeId: string): Promise<void> {
    await invoke('set_active_product_icon_theme', { themeId });
  }

  /**
   * Get active product icon theme
   */
  async getActiveProductIconTheme(): Promise<ProductIconTheme | null> {
    return await invoke<ProductIconTheme | null>('get_active_product_icon_theme');
  }

  // ===== Theme Settings =====

  /**
   * Get theme settings
   */
  async getThemeSettings(): Promise<ThemeSettings> {
    return await invoke<ThemeSettings>('get_theme_settings');
  }

  // ===== Event Subscriptions =====

  /**
   * Subscribe to theme changes
   */
  onDidChangeTheme(callback: (event: ThemeChangeEvent) => void): () => void {
    this.onDidChangeThemeCallbacks.push(callback);

    return () => {
      const index = this.onDidChangeThemeCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeThemeCallbacks.splice(index, 1);
      }
    };
  }

  // ===== Utility Methods =====

  /**
   * Clear theme data for owner
   */
  async clearThemeData(owner: string): Promise<void> {
    await invoke('clear_theme_data', { owner });
  }

  /**
   * Create color theme
   */
  createColorTheme(
    id: string,
    label: string,
    owner: string,
    themeType: ThemeType,
    colors: Record<string, string> = {},
    tokenColors: TokenColor[] = []
  ): ColorTheme {
    return {
      id,
      label,
      owner,
      theme_type: themeType,
      colors,
      token_colors: tokenColors,
    };
  }

  /**
   * Create icon theme
   */
  createIconTheme(
    id: string,
    label: string,
    owner: string,
    iconDefinitions: Record<string, IconDefinition> = {},
    fileAssociations: Record<string, string> = {},
    folderAssociations: Record<string, string> = {},
    fileExtensions: Record<string, string> = {},
    languageIds: Record<string, string> = {}
  ): IconTheme {
    return {
      id,
      label,
      owner,
      icon_definitions: iconDefinitions,
      file_associations: fileAssociations,
      folder_associations: folderAssociations,
      file_extensions: fileExtensions,
      language_ids: languageIds,
    };
  }

  /**
   * Apply color theme to CSS variables
   */
  applyColorThemeToCss(theme: ColorTheme): void {
    const root = document.documentElement;

    // Apply theme colors as CSS variables
    for (const [key, value] of Object.entries(theme.colors)) {
      // Convert key to CSS variable format (e.g., "editor.background" -> "--editor-background")
      const cssVar = `--${key.replace(/\./g, '-')}`;
      root.style.setProperty(cssVar, value);
    }

    // Set theme type as data attribute
    root.setAttribute('data-theme-type', theme.theme_type);

    console.log(`[Theme] Applied color theme: ${theme.label}`);
  }

  /**
   * Notify theme change listeners
   */
  private notifyThemeChanged(event: ThemeChangeEvent): void {
    for (const callback of this.onDidChangeThemeCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[Theme] Error in theme change callback:', error);
      }
    }
  }
}

// Export singleton instance
export const themeManager = new ThemeManager();
