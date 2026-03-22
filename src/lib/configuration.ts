import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export enum ConfigurationScope {
  User = 'user',
  Workspace = 'workspace',
  WorkspaceFolder = 'workspacefolder',
}

export interface ConfigurationSchema {
  key: string;
  scope: ConfigurationScope;
  type: string;
  default: any;
  description: string;
  enum_values?: any[];
}

export interface ConfigurationContribution {
  extension_id: string;
  title: string;
  properties: ConfigurationSchema[];
}

export interface ConfigurationChangeEvent {
  affected_keys: string[];
  scope: ConfigurationScope;
}

export interface ConfigurationValue {
  key: string;
  value: any;
  scope: ConfigurationScope;
  default_value?: any;
}

/**
 * Configuration API manager
 */
export class ConfigurationManager {
  private onDidChangeConfigurationCallbacks: Array<(event: ConfigurationChangeEvent) => void> = [];

  constructor() {}

  /**
   * Initialize configuration manager
   */
  async initialize(): Promise<void> {
    // Listen for configuration changes
    await listen<ConfigurationChangeEvent>('configuration-changed', (event) => {
      this.notifyConfigurationChanged(event.payload);
    });

    console.log('[Configuration] Initialized');
  }

  /**
   * Set settings file path
   */
  async setSettingsPath(path: string): Promise<void> {
    await invoke('set_settings_path', { path });
  }

  /**
   * Set workspace path
   */
  async setWorkspacePath(path: string): Promise<void> {
    await invoke('set_workspace_path', { path });
  }

  /**
   * Register configuration schema
   */
  async registerConfigurationSchema(contribution: ConfigurationContribution): Promise<void> {
    await invoke('register_configuration_schema', { contribution });
  }

  /**
   * Get configuration value
   */
  async get<T = any>(key: string, scope: ConfigurationScope = ConfigurationScope.User): Promise<T | null> {
    const value = await invoke<T | null>('get_configuration_value', { key, scope });
    return value;
  }

  /**
   * Get configuration with fallback
   */
  async getWithFallback<T = any>(key: string, scope: ConfigurationScope = ConfigurationScope.User): Promise<T | null> {
    const value = await invoke<T | null>('get_configuration_with_fallback', { key, scope });
    return value;
  }

  /**
   * Update configuration value
   */
  async update(key: string, value: any, scope: ConfigurationScope = ConfigurationScope.User): Promise<void> {
    await invoke('update_configuration_value', { key, value, scope });
  }

  /**
   * Get all configuration keys for a scope
   */
  async getAllKeys(scope: ConfigurationScope = ConfigurationScope.User): Promise<string[]> {
    return await invoke<string[]>('get_all_configuration_keys', { scope });
  }

  /**
   * Check if configuration key exists
   */
  async has(key: string, scope: ConfigurationScope = ConfigurationScope.User): Promise<boolean> {
    return await invoke<boolean>('has_configuration_key', { key, scope });
  }

  /**
   * Subscribe to configuration changes
   */
  onDidChangeConfiguration(callback: (event: ConfigurationChangeEvent) => void): () => void {
    this.onDidChangeConfigurationCallbacks.push(callback);

    return () => {
      const index = this.onDidChangeConfigurationCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeConfigurationCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Clear configuration data for owner
   */
  async clearConfigurationData(owner: string): Promise<void> {
    await invoke('clear_configuration_data', { owner });
  }

  /**
   * Notify configuration change listeners
   */
  private notifyConfigurationChanged(event: ConfigurationChangeEvent): void {
    for (const callback of this.onDidChangeConfigurationCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[Configuration] Error in configuration change callback:', error);
      }
    }
  }

  /**
   * Inspect configuration (get value with metadata)
   */
  async inspect<T = any>(key: string): Promise<{
    user?: T;
    workspace?: T;
    workspaceFolder?: T;
    defaultValue?: T;
  }> {
    const user = await this.get<T>(key, ConfigurationScope.User);
    const workspace = await this.get<T>(key, ConfigurationScope.Workspace);
    const workspaceFolder = await this.get<T>(key, ConfigurationScope.WorkspaceFolder);

    return {
      user: user ?? undefined,
      workspace: workspace ?? undefined,
      workspaceFolder: workspaceFolder ?? undefined,
    };
  }
}

// Export singleton instance
export const configurationManager = new ConfigurationManager();
