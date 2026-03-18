/**
 * Extension Manager
 *
 * Handles loading, activating, and deactivating extensions.
 */

import * as path from 'path';
import * as fs from 'fs';
import { ExtensionHostBridge } from '../bridge';
import * as vscodeAPI from '../api/vscode';

export interface ExtensionManifest {
  name: string;
  displayName?: string;
  version: string;
  publisher: string;
  main?: string;
  activationEvents?: string[];
  contributes?: {
    commands?: Array<{ command: string; title: string }>;
    languages?: any[];
    grammars?: any[];
    themes?: any[];
  };
}

export interface LoadedExtension {
  id: string;
  manifest: ExtensionManifest;
  extensionPath: string;
  isActive: boolean;
  context?: any;
  exports?: any;
}

export class ExtensionManager {
  private extensions = new Map<string, LoadedExtension>();
  private bridge: ExtensionHostBridge;

  constructor(bridge: ExtensionHostBridge) {
    this.bridge = bridge;

    // Initialize vscode API
    vscodeAPI.initializeAPI(bridge);

    // Inject vscode module into require cache
    // This allows extensions to do: const vscode = require('vscode')
    const Module = require('module');
    const originalRequire = Module.prototype.require;

    Module.prototype.require = function (id: string) {
      if (id === 'vscode') {
        return vscodeAPI;
      }
      return originalRequire.apply(this, arguments);
    };
  }

  /**
   * Load all installed extensions
   */
  async loadExtensions() {
    // Get extensions directory from main app
    const extensionsDir = await this.bridge.request('get-extensions-dir', {});

    if (!fs.existsSync(extensionsDir)) {
      console.error('[ExtensionManager] Extensions directory does not exist:', extensionsDir);
      return;
    }

    const extensionDirs = fs.readdirSync(extensionsDir, { withFileTypes: true })
      .filter(dirent => dirent.isDirectory())
      .map(dirent => dirent.name);

    for (const dir of extensionDirs) {
      try {
        await this.loadExtension(path.join(extensionsDir, dir));
      } catch (error) {
        console.error(`[ExtensionManager] Failed to load extension ${dir}:`, error);
      }
    }

    console.error(`[ExtensionManager] Loaded ${this.extensions.size} extensions`);
  }

  /**
   * Load a single extension
   */
  private async loadExtension(extensionPath: string) {
    const manifestPath = path.join(extensionPath, 'package.json');

    if (!fs.existsSync(manifestPath)) {
      throw new Error('Extension manifest not found');
    }

    const manifest: ExtensionManifest = JSON.parse(
      fs.readFileSync(manifestPath, 'utf-8')
    );

    const extensionId = `${manifest.publisher}.${manifest.name}`;

    this.extensions.set(extensionId, {
      id: extensionId,
      manifest,
      extensionPath,
      isActive: false,
    });

    console.error(`[ExtensionManager] Loaded: ${extensionId}`);

    // Auto-activate extensions with "*" or "onStartupFinished" activation event
    if (manifest.activationEvents?.includes('*') ||
        manifest.activationEvents?.includes('onStartupFinished')) {
      await this.activateExtension(extensionId);
    }
  }

  /**
   * Activate an extension
   */
  async activateExtension(extensionId: string) {
    const extension = this.extensions.get(extensionId);

    if (!extension) {
      throw new Error(`Extension not found: ${extensionId}`);
    }

    if (extension.isActive) {
      console.error(`[ExtensionManager] Extension already active: ${extensionId}`);
      return;
    }

    console.error(`[ExtensionManager] Activating: ${extensionId}`);

    try {
      // Load the extension's main file
      if (extension.manifest.main) {
        const mainPath = path.join(extension.extensionPath, extension.manifest.main);

        // Create extension context
        const context: vscodeAPI.ExtensionContext = {
          subscriptions: [],
          extensionPath: extension.extensionPath,
          extensionUri: { fsPath: extension.extensionPath, scheme: 'file' } as any,
          globalState: {
            get: (key: string) => undefined,
            update: (key: string, value: any) => Promise.resolve(),
            keys: () => [],
          },
          workspaceState: {
            get: (key: string) => undefined,
            update: (key: string, value: any) => Promise.resolve(),
            keys: () => [],
          },
          secrets: {
            get: async (key: string) => undefined,
            store: async (key: string, value: string) => {},
            delete: async (key: string) => {},
            onDidChange: (() => ({ dispose: () => {} })) as any,
          } as any,
          extensionMode: 1, // Production
          asAbsolutePath: (relativePath: string) => {
            return path.join(extension.extensionPath, relativePath);
          },
          storageUri: undefined,
          globalStorageUri: { fsPath: extension.extensionPath, scheme: 'file' } as any,
          logUri: { fsPath: extension.extensionPath, scheme: 'file' } as any,
        };

        // Load and activate the extension
        const extensionModule = require(mainPath);

        if (typeof extensionModule.activate === 'function') {
          extension.exports = await extensionModule.activate(context);
          extension.context = context;
          extension.isActive = true;

          console.error(`[ExtensionManager] Activated: ${extensionId}`);
        } else {
          console.warn(`[ExtensionManager] No activate function: ${extensionId}`);
        }
      }
    } catch (error) {
      console.error(`[ExtensionManager] Failed to activate ${extensionId}:`, error);
      throw error;
    }
  }

  /**
   * Deactivate an extension
   */
  async deactivateExtension(extensionId: string) {
    const extension = this.extensions.get(extensionId);

    if (!extension || !extension.isActive) {
      return;
    }

    console.error(`[ExtensionManager] Deactivating: ${extensionId}`);

    try {
      // Call deactivate if available
      const mainPath = path.join(extension.extensionPath, extension.manifest.main!);
      const extensionModule = require(mainPath);

      if (typeof extensionModule.deactivate === 'function') {
        await extensionModule.deactivate();
      }

      // Dispose subscriptions
      if (extension.context?.subscriptions) {
        for (const subscription of extension.context.subscriptions) {
          if (subscription.dispose) {
            subscription.dispose();
          }
        }
      }

      extension.isActive = false;
      console.error(`[ExtensionManager] Deactivated: ${extensionId}`);
    } catch (error) {
      console.error(`[ExtensionManager] Failed to deactivate ${extensionId}:`, error);
    }
  }

  /**
   * Deactivate all extensions
   */
  async deactivateAll() {
    const activeExtensions = Array.from(this.extensions.values())
      .filter(ext => ext.isActive);

    for (const extension of activeExtensions) {
      await this.deactivateExtension(extension.id);
    }
  }

  /**
   * Get all loaded extensions
   */
  getAllExtensions(): LoadedExtension[] {
    return Array.from(this.extensions.values());
  }

  /**
   * Get a specific extension
   */
  getExtension(extensionId: string): LoadedExtension | undefined {
    return this.extensions.get(extensionId);
  }
}
