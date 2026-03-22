import { invoke } from '@tauri-apps/api/core';

export interface DebugConfiguration {
  type: string;
  name: string;
  request: string; // "launch" or "attach"
  program?: string;
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
  console?: string;
  [key: string]: any; // Additional properties
}

export interface DebugConfigurationProvider {
  id: string;
  owner: string;
  debug_type: string;
}

export type DebugAdapterDescriptor =
  | {
      type: 'executable';
      command: string;
      args: string[];
      options?: DebugAdapterExecutableOptions;
    }
  | {
      type: 'server';
      port: number;
      host?: string;
    }
  | {
      type: 'namedpipe';
      path: string;
    };

export interface DebugAdapterExecutableOptions {
  cwd?: string;
  env?: Record<string, string>;
}

export interface DebugAdapterDescriptorFactory {
  id: string;
  owner: string;
  debug_type: string;
}

export interface LaunchConfiguration {
  version: string;
  configurations: DebugConfiguration[];
}

/**
 * Debug Configuration API manager
 */
export class DebugConfigurationManager {
  constructor() {}

  /**
   * Initialize debug configuration manager
   */
  async initialize(): Promise<void> {
    console.log('[DebugConfiguration] Initialized');
  }

  // ===== Debug Configuration Providers =====

  /**
   * Register debug configuration provider
   */
  async registerDebugConfigurationProvider(
    providerId: string,
    owner: string,
    debugType: string
  ): Promise<string> {
    const provider: DebugConfigurationProvider = {
      id: providerId,
      owner,
      debug_type: debugType,
    };

    return await invoke<string>('register_debug_configuration_provider', { provider });
  }

  /**
   * Unregister debug configuration provider
   */
  async unregisterDebugConfigurationProvider(providerId: string): Promise<void> {
    await invoke('unregister_debug_configuration_provider', { providerId });
  }

  /**
   * Get debug configuration providers for type
   */
  async getDebugConfigurationProviders(debugType: string): Promise<DebugConfigurationProvider[]> {
    return await invoke<DebugConfigurationProvider[]>('get_debug_configuration_providers', {
      debugType,
    });
  }

  // ===== Debug Adapter Descriptor Factories =====

  /**
   * Register debug adapter descriptor factory
   */
  async registerDebugAdapterDescriptorFactory(
    factoryId: string,
    owner: string,
    debugType: string
  ): Promise<string> {
    const factory: DebugAdapterDescriptorFactory = {
      id: factoryId,
      owner,
      debug_type: debugType,
    };

    return await invoke<string>('register_debug_adapter_descriptor_factory', { factory });
  }

  /**
   * Unregister debug adapter descriptor factory
   */
  async unregisterDebugAdapterDescriptorFactory(factoryId: string): Promise<void> {
    await invoke('unregister_debug_adapter_descriptor_factory', { factoryId });
  }

  /**
   * Get debug adapter descriptor factories for type
   */
  async getDebugAdapterDescriptorFactories(
    debugType: string
  ): Promise<DebugAdapterDescriptorFactory[]> {
    return await invoke<DebugAdapterDescriptorFactory[]>(
      'get_debug_adapter_descriptor_factories',
      {
        debugType,
      }
    );
  }

  // ===== Launch Configurations =====

  /**
   * Set launch configuration for workspace
   */
  async setLaunchConfiguration(
    workspaceUri: string,
    configuration: LaunchConfiguration
  ): Promise<void> {
    await invoke('set_launch_configuration', {
      workspaceUri,
      configuration,
    });
  }

  /**
   * Get launch configuration for workspace
   */
  async getLaunchConfiguration(workspaceUri: string): Promise<LaunchConfiguration | null> {
    try {
      return await invoke<LaunchConfiguration>('get_launch_configuration', {
        workspaceUri,
      });
    } catch {
      return null;
    }
  }

  /**
   * Get all launch configurations
   */
  async getAllLaunchConfigurations(): Promise<Record<string, LaunchConfiguration>> {
    return await invoke<Record<string, LaunchConfiguration>>('get_all_launch_configurations');
  }

  // ===== Utility Methods =====

  /**
   * Clear debug configuration data for owner
   */
  async clearDebugConfigurationData(owner: string): Promise<void> {
    await invoke('clear_debug_configuration_data', { owner });
  }

  /**
   * Create debug configuration
   */
  createDebugConfiguration(
    type: string,
    name: string,
    request: 'launch' | 'attach',
    additionalProperties: Record<string, any> = {}
  ): DebugConfiguration {
    return {
      type,
      name,
      request,
      ...additionalProperties,
    };
  }

  /**
   * Create launch configuration
   */
  createLaunchConfiguration(configurations: DebugConfiguration[]): LaunchConfiguration {
    return {
      version: '0.2.0',
      configurations,
    };
  }
}

// Export singleton instance
export const debugConfigurationManager = new DebugConfigurationManager();
