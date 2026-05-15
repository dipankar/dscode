import { invoke } from '@tauri-apps/api/core';

// ===== Type Definitions =====

export interface SettingsCategory {
  id: string;
  title: string;
  description?: string;
  icon?: string;
  order: number;
  owner: string;
}

export interface SettingUISchema {
  title: string;
  description?: string;
  uiType: SettingUIType;
  default: any;
  category: string;
  order: number;
  tags: string[];
  scope: SettingScope;
  deprecated: boolean;
  deprecationMessage?: string;
}

export type SettingUIType =
  | { type: 'string'; pattern?: string; minLength?: number; maxLength?: number }
  | { type: 'number'; min?: number; max?: number; step?: number }
  | { type: 'boolean' }
  | { type: 'enum'; values: EnumValue[]; allowCustom?: boolean }
  | { type: 'array'; itemType: string; minItems?: number; maxItems?: number }
  | { type: 'object' };

export interface EnumValue {
  value: string;
  label: string;
  description?: string;
}

export type SettingScope = 'application' | 'window' | 'resource' | 'language' | 'machine';

export type ConfigurationScope = 'user' | 'workspace' | 'workspaceFolder';

export interface SettingWithValue {
  key: string;
  schema: SettingUISchema;
  userValue?: any;
  workspaceValue?: any;
}

export interface SettingSearchResult {
  key: string;
  schema: SettingUISchema;
  score: number;
}

/**
 * Settings UI Manager
 */
export class SettingsManager {
  private categories: SettingsCategory[] = [];
  private uiSchemas: Map<string, SettingUISchema> = new Map();

  constructor() {}

  /**
   * Initialize settings manager
   */
  async initialize(): Promise<void> {
    // Load categories and schemas
    this.categories = await this.getCategories();
    const schemas = await this.getAllUISchemas();
    this.uiSchemas = new Map(Object.entries(schemas));

    console.log('[Settings] Initialized');
  }

  // ===== Category Management =====

  /**
   * Register settings category
   */
  async registerCategory(category: SettingsCategory): Promise<string> {
    const id = await invoke<string>('register_settings_category', { category });
    this.categories.push(category);
    this.categories.sort((a, b) => a.order - b.order);
    return id;
  }

  /**
   * Get all categories
   */
  async getCategories(): Promise<SettingsCategory[]> {
    return await invoke<SettingsCategory[]>('get_settings_categories');
  }

  /**
   * Get category by ID
   */
  async getCategory(categoryId: string): Promise<SettingsCategory> {
    return await invoke<SettingsCategory>('get_settings_category', { categoryId });
  }

  /**
   * Get cached categories
   */
  getCachedCategories(): SettingsCategory[] {
    return this.categories;
  }

  // ===== UI Schema Management =====

  /**
   * Register UI schema for a setting
   */
  async registerUISchema(key: string, schema: SettingUISchema): Promise<void> {
    await invoke('register_setting_ui_schema', { key, schema });
    this.uiSchemas.set(key, schema);
  }

  /**
   * Get UI schema for a setting
   */
  async getUISchema(key: string): Promise<SettingUISchema | null> {
    const schema = await invoke<SettingUISchema | null>('get_setting_ui_schema', { key });
    return schema;
  }

  /**
   * Get all UI schemas
   */
  async getAllUISchemas(): Promise<Record<string, SettingUISchema>> {
    return await invoke<Record<string, SettingUISchema>>('get_all_setting_ui_schemas');
  }

  /**
   * Get cached UI schema
   */
  getCachedUISchema(key: string): SettingUISchema | undefined {
    return this.uiSchemas.get(key);
  }

  // ===== Settings Search =====

  /**
   * Search settings by query
   */
  async searchSettings(query: string): Promise<SettingSearchResult[]> {
    return await invoke<SettingSearchResult[]>('search_settings', { query });
  }

  /**
   * Get settings by category
   */
  async getSettingsByCategory(categoryId: string): Promise<SettingWithValue[]> {
    return await invoke<SettingWithValue[]>('get_settings_by_category', { categoryId });
  }

  // ===== Settings Value Management =====

  /**
   * Get setting with all scope values
   */
  async getSettingWithValues(key: string): Promise<SettingWithValue> {
    return await invoke<SettingWithValue>('get_setting_with_values', { key });
  }

  /**
   * Get effective setting value (resolved)
   */
  async getEffectiveValue(key: string, scope: ConfigurationScope): Promise<any> {
    return await invoke('get_effective_setting_value', { key, scope });
  }

  /**
   * Update setting value
   */
  async updateSettingValue(key: string, value: any, scope: ConfigurationScope): Promise<void> {
    await invoke('update_setting_value', { key, value, scope });
  }

  /**
   * Reset setting to default
   */
  async resetToDefault(key: string, scope: ConfigurationScope): Promise<void> {
    await invoke('reset_setting_to_default', { key, scope });
  }

  // ===== Validation =====

  /**
   * Validate setting value
   */
  async validateValue(key: string, value: any): Promise<{ valid: boolean; error?: string }> {
    try {
      await invoke('validate_setting_value', { key, value });
      return { valid: true };
    } catch (error) {
      return { valid: false, error: String(error) };
    }
  }

  // ===== Utility Methods =====

  /**
   * Clear settings UI data for owner
   */
  async clearSettingsUIData(owner: string): Promise<void> {
    await invoke('clear_settings_ui_data', { owner });
  }

  /**
   * Create category helper
   */
  createCategory(
    id: string,
    title: string,
    owner: string,
    order: number = 0,
    description?: string,
    icon?: string
  ): SettingsCategory {
    return {
      id,
      title,
      description,
      icon,
      order,
      owner,
    };
  }

  /**
   * Create string setting schema
   */
  createStringSchema(
    title: string,
    defaultValue: string,
    category: string,
    options: {
      description?: string;
      pattern?: string;
      minLength?: number;
      maxLength?: number;
      order?: number;
      tags?: string[];
      scope?: SettingScope;
    } = {}
  ): SettingUISchema {
    return {
      title,
      description: options.description,
      uiType: {
        type: 'string',
        pattern: options.pattern,
        minLength: options.minLength,
        maxLength: options.maxLength,
      },
      default: defaultValue,
      category,
      order: options.order ?? 0,
      tags: options.tags ?? [],
      scope: options.scope ?? 'application',
      deprecated: false,
    };
  }

  /**
   * Create number setting schema
   */
  createNumberSchema(
    title: string,
    defaultValue: number,
    category: string,
    options: {
      description?: string;
      min?: number;
      max?: number;
      step?: number;
      order?: number;
      tags?: string[];
      scope?: SettingScope;
    } = {}
  ): SettingUISchema {
    return {
      title,
      description: options.description,
      uiType: {
        type: 'number',
        min: options.min,
        max: options.max,
        step: options.step,
      },
      default: defaultValue,
      category,
      order: options.order ?? 0,
      tags: options.tags ?? [],
      scope: options.scope ?? 'application',
      deprecated: false,
    };
  }

  /**
   * Create boolean setting schema
   */
  createBooleanSchema(
    title: string,
    defaultValue: boolean,
    category: string,
    options: {
      description?: string;
      order?: number;
      tags?: string[];
      scope?: SettingScope;
    } = {}
  ): SettingUISchema {
    return {
      title,
      description: options.description,
      uiType: { type: 'boolean' },
      default: defaultValue,
      category,
      order: options.order ?? 0,
      tags: options.tags ?? [],
      scope: options.scope ?? 'application',
      deprecated: false,
    };
  }

  /**
   * Create enum setting schema
   */
  createEnumSchema(
    title: string,
    defaultValue: string,
    values: EnumValue[],
    category: string,
    options: {
      description?: string;
      allowCustom?: boolean;
      order?: number;
      tags?: string[];
      scope?: SettingScope;
    } = {}
  ): SettingUISchema {
    return {
      title,
      description: options.description,
      uiType: {
        type: 'enum',
        values,
        allowCustom: options.allowCustom,
      },
      default: defaultValue,
      category,
      order: options.order ?? 0,
      tags: options.tags ?? [],
      scope: options.scope ?? 'application',
      deprecated: false,
    };
  }

  /**
   * Create array setting schema
   */
  createArraySchema(
    title: string,
    defaultValue: any[],
    itemType: string,
    category: string,
    options: {
      description?: string;
      minItems?: number;
      maxItems?: number;
      order?: number;
      tags?: string[];
      scope?: SettingScope;
    } = {}
  ): SettingUISchema {
    return {
      title,
      description: options.description,
      uiType: {
        type: 'array',
        itemType,
        minItems: options.minItems,
        maxItems: options.maxItems,
      },
      default: defaultValue,
      category,
      order: options.order ?? 0,
      tags: options.tags ?? [],
      scope: options.scope ?? 'application',
      deprecated: false,
    };
  }

  /**
   * Create object setting schema
   */
  createObjectSchema(
    title: string,
    defaultValue: Record<string, any>,
    category: string,
    options: {
      description?: string;
      order?: number;
      tags?: string[];
      scope?: SettingScope;
    } = {}
  ): SettingUISchema {
    return {
      title,
      description: options.description,
      uiType: { type: 'object' },
      default: defaultValue,
      category,
      order: options.order ?? 0,
      tags: options.tags ?? [],
      scope: options.scope ?? 'application',
      deprecated: false,
    };
  }

  /**
   * Mark schema as deprecated
   */
  deprecateSchema(schema: SettingUISchema, message: string): SettingUISchema {
    return {
      ...schema,
      deprecated: true,
      deprecationMessage: message,
    };
  }

  /**
   * Filter settings by tags
   */
  filterByTags(settings: SettingWithValue[], tags: string[]): SettingWithValue[] {
    return settings.filter((setting) =>
      tags.some((tag) => setting.schema.tags.includes(tag))
    );
  }

  /**
   * Group settings by category
   */
  groupByCategory(settings: SettingWithValue[]): Map<string, SettingWithValue[]> {
    const grouped = new Map<string, SettingWithValue[]>();

    for (const setting of settings) {
      const category = setting.schema.category;
      if (!grouped.has(category)) {
        grouped.set(category, []);
      }
      grouped.get(category)!.push(setting);
    }

    // Sort within each category
    for (const [, categorySettings] of grouped) {
      categorySettings.sort((a, b) => a.schema.order - b.schema.order);
    }

    return grouped;
  }

  /**
   * Get modified settings (different from default)
   */
  getModifiedSettings(settings: SettingWithValue[]): SettingWithValue[] {
    return settings.filter(
      (setting) =>
        setting.userValue !== undefined ||
        setting.workspaceValue !== undefined
    );
  }

  /**
   * Export settings as JSON
   */
  async exportSettings(scope: ConfigurationScope): Promise<string> {
    const schemas = await this.getAllUISchemas();
    const settings: Record<string, any> = {};

    for (const [key] of Object.entries(schemas)) {
      try {
        const value = await this.getEffectiveValue(key, scope);
        if (value !== null) {
          settings[key] = value;
        }
      } catch {
        // Skip settings that don't have values
      }
    }

    return JSON.stringify(settings, null, 2);
  }

  /**
   * Import settings from JSON
   */
  async importSettings(
    json: string,
    scope: ConfigurationScope,
    overwrite: boolean = false
  ): Promise<{ success: number; failed: number; errors: string[] }> {
    let settings: Record<string, any>;

    try {
      settings = JSON.parse(json);
    } catch {
      return { success: 0, failed: 0, errors: ['Invalid JSON'] };
    }

    let success = 0;
    let failed = 0;
    const errors: string[] = [];

    for (const [key, value] of Object.entries(settings)) {
      try {
        // Check if setting exists
        const schema = await this.getUISchema(key);
        if (!schema) {
          failed++;
          errors.push(`Setting '${key}' not found`);
          continue;
        }

        // If not overwriting, skip settings that already have values
        if (!overwrite) {
          const existing = await this.getSettingWithValues(key);
          if (
            (scope === 'user' && existing.userValue !== undefined) ||
            (scope === 'workspace' && existing.workspaceValue !== undefined)
          ) {
            continue;
          }
        }

        // Validate and update
        const validation = await this.validateValue(key, value);
        if (!validation.valid) {
          failed++;
          errors.push(`Validation failed for '${key}': ${validation.error}`);
          continue;
        }

        await this.updateSettingValue(key, value, scope);
        success++;
      } catch (error) {
        failed++;
        errors.push(`Failed to import '${key}': ${error}`);
      }
    }

    return { success, failed, errors };
  }
}

// Export singleton instance
export const settingsManager = new SettingsManager();
