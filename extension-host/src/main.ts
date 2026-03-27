/**
 * DSCode Extension Host
 *
 * This is the Node.js process that runs VSCode extensions.
 * It communicates with the main Tauri app via IPC.
 */

import { ExtensionHostBridge } from './bridge';
import { ExtensionManager } from './extensions/manager';

class ExtensionHost {
  private bridge: ExtensionHostBridge;
  private extensionManager?: ExtensionManager;

  constructor() {
    this.bridge = new ExtensionHostBridge();
  }

  async start() {
    console.error('[ExtensionHost] Starting...');

    // Initialize IPC bridge first
    await this.bridge.connect();
    console.error('[ExtensionHost] Bridge connected');

    // Now create the ExtensionManager after bridge is connected
    this.extensionManager = new ExtensionManager(this.bridge);
    console.error('[ExtensionHost] Extension manager initialized');

    // Set up message handlers
    this.setupMessageHandlers();

    // Notify Tauri that we're ready to accept activation requests
    // Send this BEFORE loading extensions so Rust knows we're connected
    console.error('[ExtensionHost] Sending ready signal...');
    try {
      await this.bridge.send('extension-host-ready', { ready: true });
      console.error('[ExtensionHost] Ready signal sent');
    } catch (error) {
      console.error('[ExtensionHost] Failed to send ready signal:', error);
    }

    // Load installed extensions
    await this.extensionManager.loadExtensions();
    console.log('[ExtensionHost] Extensions loaded');

    // Signal startup finished to trigger onStartupFinished activations
    // This should happen after initial extensions are loaded
    await this.extensionManager.signalStartupFinished();
    console.log('[ExtensionHost] Startup finished signal sent');

    console.log('[ExtensionHost] Ready');
  }

  private setupMessageHandlers() {
    this.bridge.on('activate-extension', async (payload: any) => {
      if (this.extensionManager) {
        const extensionId = payload.extensionId || payload;
        await this.extensionManager.activateExtension(extensionId);
      }
    });

    this.bridge.on('deactivate-extension', async (payload: any) => {
      if (this.extensionManager) {
        const extensionId = payload.extensionId || payload;
        await this.extensionManager.deactivateExtension(extensionId);
      }
    });

    // Extension listing - return all loaded extensions
    this.bridge.on('list-extensions', async (payload: any, respond: Function) => {
      if (!this.extensionManager) {
        respond({ extensions: [] });
        return;
      }

      const extensions = this.extensionManager.getAllExtensions().map((ext) => ({
        id: ext.id,
        name: ext.manifest.name,
        displayName: ext.manifest.displayName || ext.manifest.name,
        version: ext.manifest.version,
        publisher: ext.manifest.publisher,
        description: ext.manifest.description || null,
        path: ext.extensionPath,
        isActive: ext.isActive,
        activationEvents: ext.manifest.activationEvents || [],
        contributes: ext.manifest.contributes || {},
      }));

      respond({ extensions });
    });

    // Reload extensions (after installation)
    this.bridge.on('reload-extensions', async (payload: any, respond: Function) => {
      if (!this.extensionManager) {
        respond({ success: false, error: 'Extension manager not initialized' });
        return;
      }

      try {
        await this.extensionManager.reloadExtensions();
        respond({ success: true });
      } catch (error: any) {
        respond({ success: false, error: error.message });
      }
    });

    // Uninstall extension
    this.bridge.on('uninstall-extension', async (payload: any, respond: Function) => {
      if (!this.extensionManager) {
        respond({ success: false, error: 'Extension manager not initialized' });
        return;
      }

      const { extensionId } = payload;
      if (!extensionId) {
        respond({ success: false, error: 'Extension ID is required' });
        return;
      }

      try {
        await this.extensionManager.uninstallExtension(extensionId);
        respond({ success: true });
      } catch (error: any) {
        respond({ success: false, error: error.message });
      }
    });

    // ==================== Activation Event Handlers ====================

    // Signal startup finished - triggers onStartupFinished extensions
    this.bridge.on('signal-startup-finished', async () => {
      if (this.extensionManager) {
        await this.extensionManager.signalStartupFinished();
      }
    });

    // Trigger onLanguage activation
    this.bridge.on('trigger-on-language', async (languageId: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnLanguage(languageId);
      }
    });

    // Trigger onCommand activation
    this.bridge.on('trigger-on-command', async (commandId: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnCommand(commandId);
      }
    });

    // Trigger onView activation
    this.bridge.on('trigger-on-view', async (viewId: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnView(viewId);
      }
    });

    // Trigger onDebug activation
    this.bridge.on('trigger-on-debug', async (debugType?: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnDebug(debugType);
      }
    });

    // Trigger onUri activation
    this.bridge.on('trigger-on-uri', async (scheme?: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnUri(scheme);
      }
    });

    // Trigger onFileSystem activation
    this.bridge.on('trigger-on-filesystem', async (scheme: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnFileSystem(scheme);
      }
    });

    // Trigger onWebviewPanel activation
    this.bridge.on('trigger-on-webview-panel', async (viewType: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnWebviewPanel(viewType);
      }
    });

    // Trigger onCustomEditor activation
    this.bridge.on('trigger-on-custom-editor', async (viewType: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnCustomEditor(viewType);
      }
    });

    // Trigger onNotebook activation
    this.bridge.on('trigger-on-notebook', async (notebookType: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnNotebook(notebookType);
      }
    });

    // Trigger onAuthenticationRequest activation
    this.bridge.on('trigger-on-authentication', async (providerId: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnAuthenticationRequest(providerId);
      }
    });

    // Trigger onTerminalProfile activation
    this.bridge.on('trigger-on-terminal-profile', async (profileId: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerOnTerminalProfile(profileId);
      }
    });

    // Trigger workspaceContains activation
    this.bridge.on('trigger-workspace-contains', async (pattern: string) => {
      if (this.extensionManager) {
        await this.extensionManager.triggerWorkspaceContains(pattern);
      }
    });

    // Get pending activations (for debugging)
    this.bridge.on('get-pending-activations', async (_: any, respond: Function) => {
      if (!this.extensionManager) {
        respond({ activations: {} });
        return;
      }

      const pending = this.extensionManager.getPendingActivations();
      const result: Record<string, string[]> = {};
      for (const [key, extensions] of pending) {
        result[key] = extensions;
      }
      respond({ activations: result });
    });
  }

  async shutdown() {
    console.error('[ExtensionHost] Shutting down...');
    if (this.extensionManager) {
      await this.extensionManager.deactivateAll();
    }
    await this.bridge.disconnect();
  }
}

// Entry point
const host = new ExtensionHost();

process.on('SIGINT', async () => {
  await host.shutdown();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await host.shutdown();
  process.exit(0);
});

// Start the extension host
host.start().catch((error) => {
  console.error('[Extension Host] Fatal error:', error);
  process.exit(1);
});
