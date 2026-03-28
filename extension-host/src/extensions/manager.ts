/**
 * Extension Manager
 *
 * Handles loading, activating, and deactivating extensions.
 * Supports VS Code-compatible lazy activation based on activation events.
 */

import * as path from 'path';
import * as fs from 'fs';
import { ExtensionHostBridge } from '../bridge';
import * as vscodeAPI from '../api/vscode';
import { PersistentMemento } from './storage';
import { ActivationEventManager, ParsedActivationEvent } from './activation';
import { SecretStorageImpl } from '../api/authentication';

/**
 * STATE MACHINE: ExtensionLifecycle
 *
 * Tracks the lifecycle of an individual extension from registration through
 * activation to eventual deactivation or failure.
 *
 * State Diagram:
 *
 *   Registered ──────► Activating ──────► Active
 *       ▲                  │                │
 *       │                  │ (error)        │
 *       │                  ▼                ▼
 *   Inactive ◄────── Failed          Deactivating
 *       ▲                ▲                │
 *       │                │ (error)        │
 *       │                └────────────────┘
 *       │                                 │
 *       └─────────────────────────────────┘
 *
 * Transitions:
 *   Registered   -> Activating    (activateExtension() called)
 *   Activating   -> Active        (activate() resolved successfully)
 *   Activating   -> Failed        (activate() threw, dependency failed, or module load error)
 *   Active       -> Deactivating  (deactivateExtension() called)
 *   Deactivating -> Inactive      (deactivate() + dispose() completed)
 *   Deactivating -> Failed        (deactivate() or dispose() threw)
 *   Failed       -> Activating    (retry activation)
 *   Inactive     -> Activating    (re-activation after deactivation)
 *
 * Concurrency Invariant:
 *   Node.js is single-threaded, but `await` yields the event loop. The state
 *   field itself acts as the concurrency guard: we transition to 'Activating'
 *   SYNCHRONOUSLY (before any await), so a re-entrant call sees 'Activating'
 *   and returns early. This replaces the old `activating` Set + `isActive`
 *   boolean pattern which had a TOCTOU window between checking isActive and
 *   adding to the activating Set.
 *
 * Interruption Table:
 * ┌──────────────┬─────────────────────────────────────────────────────────────┐
 * │ State        │ What happens if extension host process crashes             │
 * ├──────────────┼─────────────────────────────────────────────────────────────┤
 * │ Registered   │ Safe. No resources allocated. Backend unaware of this      │
 * │              │ extension. On host restart, extension re-registers.        │
 * ├──────────────┼─────────────────────────────────────────────────────────────┤
 * │ Activating   │ DANGER: activate() may have partially executed. Resources  │
 * │              │ (event listeners, file handles) may be allocated but not   │
 * │              │ tracked in subscriptions[]. Backend IPC request will       │
 * │              │ timeout after 30s → backend marks extension as inactive.   │
 * │              │ Frontend shows extension as inactive. On host restart,     │
 * │              │ extension re-registers as Registered. Partial resources    │
 * │              │ from the crashed process are freed by OS process cleanup.  │
 * ├──────────────┼─────────────────────────────────────────────────────────────┤
 * │ Active       │ All subscriptions and registered commands are lost.        │
 * │              │ Backend still thinks extension is active (no notification).│
 * │              │ Frontend still shows extension as active (stale state).    │
 * │              │ User commands targeting this extension will fail silently. │
 * │              │ On host restart, extension re-activates from scratch.      │
 * │              │ Recovery: backend should detect host crash and mark all    │
 * │              │ extensions inactive, then re-activate after restart.       │
 * ├──────────────┼─────────────────────────────────────────────────────────────┤
 * │ Deactivating │ dispose() may have partially run. Some subscriptions freed,│
 * │              │ others leaked (freed by OS on process exit). Backend       │
 * │              │ unload IPC request will timeout → extension stays marked   │
 * │              │ active in backend. Status bar items from this extension    │
 * │              │ are NOT cleaned up. Frontend shows stale items.            │
 * ├──────────────┼─────────────────────────────────────────────────────────────┤
 * │ Inactive     │ Safe. No resources held. Backend should already know.      │
 * ├──────────────┼─────────────────────────────────────────────────────────────┤
 * │ Failed       │ Safe. No resources held. May need manual retry.            │
 * └──────────────┴─────────────────────────────────────────────────────────────┘
 *
 * Cross-Layer Impact:
 *   Backend (Rust): Tracks extension.active in SessionState. Updated only
 *     AFTER successful IPC response from extension host. If host crashes,
 *     backend state becomes stale until manually reconciled.
 *   Frontend (Svelte): Receives SessionEvent::ExtensionLoaded/Unloaded.
 *     Shows whatever backend tells it. If backend is stale, frontend is stale.
 *   Recovery: On host restart, backend should call scan_extensions() +
 *     load_auto_start_extensions() which re-synchronizes all three layers.
 */
type ExtensionState = 'Registered' | 'Activating' | 'Active' | 'Deactivating' | 'Inactive' | 'Failed';

export interface ExtensionManifest {
  name: string;
  displayName?: string;
  description?: string;
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
  extensionDependencies?: string[];
  extensionPack?: string[];
  categories?: string[];
  repository?: string | { url?: string };
}

export interface LoadedExtension {
  id: string;
  manifest: ExtensionManifest;
  extensionPath: string;
  state: ExtensionState;
  context?: any;
  exports?: any;
}

export class ExtensionManager {
  private extensions = new Map<string, LoadedExtension>();
  private bridge: ExtensionHostBridge;
  private activationManager: ActivationEventManager;
  private startupComplete = false;

  constructor(bridge: ExtensionHostBridge) {
    this.bridge = bridge;
    this.activationManager = new ActivationEventManager();

    // Set up activation callback
    this.activationManager.setActivationCallback(async (extensionId, event) => {
      console.log(
        `[ExtensionManager] Activation requested for ${extensionId} due to ${event.type}${event.argument ? ':' + event.argument : ''}`
      );
      await this.activateExtension(extensionId);
    });

    // Initialize vscode API
    vscodeAPI.initializeAPI(bridge);

    // Inject vscode module into require cache
    // This allows extensions to do: const vscode = require('vscode')
    const Module = require('module');
    const originalRequire = Module.prototype.require;

    Module.prototype.require = function (id: string) {
      if (id === 'vscode') {
        // Wrap in Proxy to catch undefined property access
        return new Proxy(vscodeAPI, {
          get(target: any, prop: string | symbol) {
            if (prop === Symbol.unscopables) {
              return undefined;
            }
            const value = target[prop];
            if (value === undefined && typeof prop === 'string' && !prop.startsWith('_')) {
              console.warn(`[vscode API] Accessing undefined property: ${String(prop)}`);
            }
            return value;
          },
        });
      }
      return originalRequire.apply(this, arguments);
    };
  }

  /**
   * Load all installed extensions
   */
  async loadExtensions() {
    // Get extensions directory from main app
    console.log('[ExtensionManager] Requesting extensions directory...');
    const extensionsDir = (await this.bridge.request('get-extensions-dir', {})) as string;
    console.log('[ExtensionManager] Received extensions directory:', extensionsDir);

    if (!fs.existsSync(extensionsDir)) {
      console.error('[ExtensionManager] Extensions directory does not exist:', extensionsDir);
      return;
    }

    console.log('[ExtensionManager] Reading directory contents...');
    const extensionDirs = fs
      .readdirSync(extensionsDir, { withFileTypes: true })
      .filter((dirent) => dirent.isDirectory())
      .map((dirent) => dirent.name);

    console.error('[ExtensionManager] Found extension directories:', extensionDirs);

    for (const dir of extensionDirs) {
      try {
        console.log(`[ExtensionManager] Loading extension: ${dir}`);
        await this.loadExtension(path.join(extensionsDir, dir));
      } catch (error) {
        console.error(`[ExtensionManager] Failed to load extension ${dir}:`, error);
      }
    }

    console.log(`[ExtensionManager] Loaded ${this.extensions.size} extensions`);

    // Activate extensions with "*" activation event
    const immediateExtensions = this.activationManager.getImmediateActivations();
    console.log(`[ExtensionManager] Activating ${immediateExtensions.length} immediate extensions`);
    for (const extensionId of immediateExtensions) {
      try {
        await this.activateExtension(extensionId);
      } catch (error) {
        console.error(
          `[ExtensionManager] Failed to activate immediate extension ${extensionId}:`,
          error
        );
      }
    }
  }

  /**
   * Signal that startup is complete - activates onStartupFinished extensions
   */
  async signalStartupFinished(): Promise<void> {
    if (this.startupComplete) {
      return;
    }
    this.startupComplete = true;
    console.log('[ExtensionManager] Startup finished, triggering onStartupFinished activations');
    await this.activationManager.triggerStartupFinished();
  }

  // ==================== Activation Event Triggers ====================

  /**
   * Trigger activation for opening a file with a specific language
   */
  async triggerOnLanguage(languageId: string): Promise<void> {
    await this.activationManager.triggerOnLanguage(languageId);
  }

  /**
   * Trigger activation for executing a command
   */
  async triggerOnCommand(commandId: string): Promise<void> {
    await this.activationManager.triggerOnCommand(commandId);
  }

  /**
   * Trigger activation for a view becoming visible
   */
  async triggerOnView(viewId: string): Promise<void> {
    await this.activationManager.triggerOnView(viewId);
  }

  /**
   * Trigger activation for debugging
   */
  async triggerOnDebug(debugType?: string): Promise<void> {
    await this.activationManager.triggerOnDebug(debugType);
  }

  /**
   * Trigger activation for a URI scheme being opened
   */
  async triggerOnUri(scheme?: string): Promise<void> {
    await this.activationManager.triggerOnUri(scheme);
  }

  /**
   * Trigger activation for file system scheme
   */
  async triggerOnFileSystem(scheme: string): Promise<void> {
    await this.activationManager.triggerOnFileSystem(scheme);
  }

  /**
   * Trigger activation for authentication request
   */
  async triggerOnAuthenticationRequest(providerId: string): Promise<void> {
    await this.activationManager.triggerOnAuthenticationRequest(providerId);
  }

  /**
   * Trigger activation for webview panel
   */
  async triggerOnWebviewPanel(viewType: string): Promise<void> {
    await this.activationManager.triggerOnWebviewPanel(viewType);
  }

  /**
   * Trigger activation for custom editor
   */
  async triggerOnCustomEditor(viewType: string): Promise<void> {
    await this.activationManager.triggerOnCustomEditor(viewType);
  }

  /**
   * Trigger activation for notebook
   */
  async triggerOnNotebook(notebookType: string): Promise<void> {
    await this.activationManager.triggerOnNotebook(notebookType);
  }

  /**
   * Trigger activation for terminal profile
   */
  async triggerOnTerminalProfile(profileId: string): Promise<void> {
    await this.activationManager.triggerOnTerminalProfile(profileId);
  }

  /**
   * Trigger activation when workspace contains a pattern
   */
  async triggerWorkspaceContains(pattern: string): Promise<void> {
    await this.activationManager.triggerWorkspaceContains(pattern);
  }

  /**
   * Check if an extension is activated
   */
  isExtensionActivated(extensionId: string): boolean {
    return this.activationManager.isActivated(extensionId);
  }

  /**
   * Get pending activations for debugging
   */
  getPendingActivations(): Map<string, string[]> {
    return this.activationManager.getPendingActivations();
  }

  /**
   * Validates and performs a state transition for an extension.
   * Invalid transitions are logged but do not throw - graceful degradation.
   */
  private transitionExtension(id: string, to: ExtensionState): boolean {
    const extension = this.extensions.get(id);
    if (!extension) {
      console.error(`[ExtensionManager] Cannot transition unknown extension '${id}' to '${to}'`);
      return false;
    }

    const validTransitions: Record<ExtensionState, ExtensionState[]> = {
      'Registered': ['Activating'],
      'Activating': ['Active', 'Failed'],
      'Active': ['Deactivating'],
      'Deactivating': ['Inactive', 'Failed'],
      'Inactive': ['Activating'],
      'Failed': ['Activating'],
    };

    const allowed = validTransitions[extension.state];
    if (!allowed || !allowed.includes(to)) {
      console.error(
        `[ExtensionManager] Invalid state transition for '${id}': ${extension.state} -> ${to}. ` +
        `Allowed transitions from '${extension.state}': [${allowed?.join(', ') || 'none'}]`
      );
      return false;
    }

    console.log(`[ExtensionManager] ${id}: ${extension.state} -> ${to}`);
    extension.state = to;
    return true;
  }

  /**
   * Load a single extension
   */
  private async loadExtension(extensionPath: string) {
    const manifestPath = path.join(extensionPath, 'package.json');

    if (!fs.existsSync(manifestPath)) {
      throw new Error('Extension manifest not found');
    }

    const manifest: ExtensionManifest = JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));

    const extensionId = `${manifest.publisher}.${manifest.name}`;

    this.extensions.set(extensionId, {
      id: extensionId,
      manifest,
      extensionPath,
      state: 'Registered',
    });

    // Register with ExtensionsAPI so vscode.extensions.getExtension() can find it
    vscodeAPI.registerLoadedExtension(
      extensionId,
      extensionPath,
      manifest, // packageJSON
      false, // isActive
      undefined // exports (not yet activated)
    );

    console.log(`[ExtensionManager] Loaded: ${extensionId}`);

    // Register with activation manager for lazy loading
    const activationEvents = manifest.activationEvents || [];
    if (activationEvents.length > 0) {
      this.activationManager.registerExtension(extensionId, activationEvents);
    } else {
      // Extensions without activation events are activated immediately (VS Code behavior)
      console.log(
        `[ExtensionManager] No activation events for ${extensionId}, marking for immediate activation`
      );
      this.activationManager.registerExtension(extensionId, ['*']);
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

    if (extension.state === 'Active' || extension.state === 'Activating') {
      console.error(`[ExtensionManager] Extension already ${extension.state}: ${extensionId}`);
      return;
    }

    if (!this.transitionExtension(extensionId, 'Activating')) {
      return;
    }

    console.error(`[ExtensionManager] Activating: ${extensionId}`);

    try {
      const dependencies = extension.manifest.extensionDependencies || [];
      for (const dependencyId of dependencies) {
        if (!this.extensions.has(dependencyId)) {
          console.warn(
            `[ExtensionManager] Missing dependency ${dependencyId} required by ${extensionId}`
          );
          continue;
        }
        try {
          await this.activateExtension(dependencyId);
        } catch (depError) {
          console.warn(
            `[ExtensionManager] Failed to activate dependency ${dependencyId} for ${extensionId}:`,
            depError
          );
        }
      }

      // Load the extension's main file
      if (extension.manifest.main) {
        const mainPath = path.join(extension.extensionPath, extension.manifest.main);

        const storageInfo = await this.bridge.request('get-extension-storage', {
          extensionId,
        });
        const storagePaths = storageInfo as { global: string; workspace: string; logs: string };

        const globalState = new PersistentMemento(
          path.join(storagePaths.global, 'globalState.json')
        );
        const workspaceState = new PersistentMemento(
          path.join(storagePaths.workspace, 'workspaceState.json')
        );

        // Create extension context
        const context: vscodeAPI.ExtensionContext = {
          subscriptions: [],
          extensionPath: extension.extensionPath,
          extensionUri: vscodeAPI.Uri.file(extension.extensionPath),
          globalState: {
            get: (key: string, defaultValue?: unknown) => globalState.get(key, defaultValue),
            update: (key: string, value: any) => globalState.update(key, value),
            keys: () => globalState.keys(),
          },
          workspaceState: {
            get: (key: string, defaultValue?: unknown) => workspaceState.get(key, defaultValue),
            update: (key: string, value: any) => workspaceState.update(key, value),
            keys: () => workspaceState.keys(),
          },
          secrets: new SecretStorageImpl(this.bridge, extensionId),
          extensionMode: 1, // Production
          asAbsolutePath: (relativePath: string) => {
            const resolved = path.resolve(extension.extensionPath, relativePath);
            if (
              !resolved.startsWith(extension.extensionPath + path.sep) &&
              resolved !== extension.extensionPath
            ) {
              throw new Error(`Path escapes extension directory: ${relativePath}`);
            }
            return resolved;
          },
          environmentVariableCollection: new vscodeAPI.EnvironmentVariableCollection(),
          storageUri: vscodeAPI.Uri.file(storagePaths.workspace),
          globalStorageUri: vscodeAPI.Uri.file(storagePaths.global),
          logUri: vscodeAPI.Uri.file(storagePaths.logs),
        };

        // Load and activate the extension
        console.error(`[ExtensionManager] Loading module from: ${mainPath}`);
        console.error(`[ExtensionManager] Checking vscode module availability...`);
        try {
          const testVscode = require('vscode');
          console.error(`[ExtensionManager] Key vscode API types:`);
          console.error(`  Disposable: ${typeof testVscode.Disposable}`);
          console.error(`  EventEmitter: ${typeof testVscode.EventEmitter}`);
          console.error(`  CancellationTokenSource: ${typeof testVscode.CancellationTokenSource}`);
          console.error(`  Uri: ${typeof testVscode.Uri}`);
          console.error(`  Range: ${typeof testVscode.Range}`);
          console.error(`  Position: ${typeof testVscode.Position}`);
          console.error(`  Selection: ${typeof testVscode.Selection}`);
          console.error(`  CompletionItem: ${typeof testVscode.CompletionItem}`);
          console.error(`  CodeAction: ${typeof testVscode.CodeAction}`);
          console.error(`  Diagnostic: ${typeof testVscode.Diagnostic}`);
          console.error(`  TreeItem: ${typeof testVscode.TreeItem}`);
          console.error(`  QuickPickItem: ${typeof testVscode.QuickPickItem}`);
          console.error(`  Hover: ${typeof testVscode.Hover}`);
          console.error(`  Location: ${typeof testVscode.Location}`);
          console.error(`  DocumentSymbol: ${typeof testVscode.DocumentSymbol}`);
          console.error(`  MarkdownString: ${typeof testVscode.MarkdownString}`);
          console.error(`  ThemeIcon: ${typeof testVscode.ThemeIcon}`);
          console.error(`  ThemeColor: ${typeof testVscode.ThemeColor}`);
          console.error(`  TextEdit: ${typeof testVscode.TextEdit}`);
          console.error(`  WorkspaceEdit: ${typeof testVscode.WorkspaceEdit}`);
          console.error(`  SnippetString: ${typeof testVscode.SnippetString}`);
          console.error(`  OutputChannel: ${typeof testVscode.OutputChannel}`);
          console.error(`  StatusBarItem: ${typeof testVscode.StatusBarItem}`);
          console.error(`  Terminal: ${typeof testVscode.Terminal}`);
          console.error(`  QuickPick: ${typeof testVscode.QuickPick}`);
          console.error(`  InputBox: ${typeof testVscode.InputBox}`);
          console.error(`  TextDocument: ${typeof testVscode.TextDocument}`);
          console.error(`  TextEditor: ${typeof testVscode.TextEditor}`);
          console.error(`  FileSystemError: ${typeof testVscode.FileSystemError}`);
          console.error(`  NotebookCell: ${typeof testVscode.NotebookCell}`);
          console.error(`  NotebookCellData: ${typeof testVscode.NotebookCellData}`);
          console.error(`  NotebookCellOutput: ${typeof testVscode.NotebookCellOutput}`);
          console.error(`  NotebookCellOutputItem: ${typeof testVscode.NotebookCellOutputItem}`);
          console.error(`  NotebookDocument: ${typeof testVscode.NotebookDocument}`);
          console.error(`  Task: ${typeof testVscode.Task}`);
          console.error(`  DebugSession: ${typeof testVscode.DebugSession}`);
          console.error(`  Extension: ${typeof testVscode.Extension}`);
          console.error(`  SourceControl: ${typeof testVscode.SourceControl}`);
          console.error(`  TestController: ${typeof testVscode.TestController}`);
          console.error(`  CommentController: ${typeof testVscode.CommentController}`);
          console.error(
            `  EnvironmentVariableCollection: ${typeof testVscode.EnvironmentVariableCollection}`
          );
        } catch (e) {
          console.error(`[ExtensionManager] Failed to load vscode module:`, e);
        }

        const extensionModule = require(mainPath);

        await vscodeAPI.__withExtensionActivation(extensionId, async () => {
          if (typeof extensionModule.activate === 'function') {
            extension.exports = await extensionModule.activate(context);
            extension.context = context;
            this.transitionExtension(extensionId, 'Active');

            // Update ExtensionsAPI with the activated state and exports
            vscodeAPI.updateExtensionActivation(extensionId, extension.exports);

            console.error(`[ExtensionManager] Activated: ${extensionId}`);
          } else {
            console.warn(`[ExtensionManager] No activate function: ${extensionId}`);
          }
        });
      }
    } catch (error: any) {
      console.error(`[ExtensionManager] Failed to activate ${extensionId}:`, error);
      if (error.stack) {
        console.error(`[ExtensionManager] Stack trace:\n${error.stack}`);
      }
      this.transitionExtension(extensionId, 'Failed');
      (vscodeAPI as any)._updateExtensionExports(extensionId, extension.exports || {});
    }
  }

  /**
   * Deactivate an extension
   */
  async deactivateExtension(extensionId: string) {
    const extension = this.extensions.get(extensionId);

    if (!extension || extension.state !== 'Active') {
      return;
    }

    if (!this.transitionExtension(extensionId, 'Deactivating')) {
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

      this.transitionExtension(extensionId, 'Inactive');
      console.error(`[ExtensionManager] Deactivated: ${extensionId}`);
    } catch (error) {
      console.error(`[ExtensionManager] Failed to deactivate ${extensionId}:`, error);
      this.transitionExtension(extensionId, 'Failed');
    }
  }

  /**
   * Deactivate all extensions
   */
  async deactivateAll() {
    const activeExtensions = Array.from(this.extensions.values()).filter((ext) => ext.state === 'Active');

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

  /**
   * Reload extensions from disk (e.g., after installing a new one)
   */
  async reloadExtensions(): Promise<void> {
    console.error('[ExtensionManager] Reloading extensions...');
    await this.loadExtensions();
  }

  /**
   * Uninstall an extension (deactivate and remove from filesystem)
   */
  async uninstallExtension(extensionId: string): Promise<void> {
    const extension = this.extensions.get(extensionId);
    if (!extension) {
      throw new Error(`Extension ${extensionId} not found`);
    }

    // Deactivate if active
    if (extension.state === 'Active') {
      await this.deactivateExtension(extensionId);
    }

    // Unregister from activation manager
    this.activationManager.unregisterExtension(extensionId);

    // Remove from filesystem
    const fs = require('fs');
    const path = require('path');

    try {
      // Remove extension directory
      await fs.promises.rm(extension.extensionPath, { recursive: true, force: true });
      console.error(`[ExtensionManager] Removed extension directory: ${extension.extensionPath}`);

      // Remove from loaded extensions map
      this.extensions.delete(extensionId);

      console.error(`[ExtensionManager] Successfully uninstalled extension: ${extensionId}`);
    } catch (error) {
      console.error(`[ExtensionManager] Failed to uninstall extension: ${error}`);
      throw new Error(`Failed to uninstall extension: ${error}`);
    }
  }
}
