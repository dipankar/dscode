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

    // Load installed extensions
    await this.extensionManager.loadExtensions();
    console.error('[ExtensionHost] Extensions loaded');

    console.error('[ExtensionHost] Ready');
  }

  private setupMessageHandlers() {
    this.bridge.on('activate-extension', async (extensionId: string) => {
      if (this.extensionManager) {
        await this.extensionManager.activateExtension(extensionId);
      }
    });

    this.bridge.on('deactivate-extension', async (extensionId: string) => {
      if (this.extensionManager) {
        await this.extensionManager.deactivateExtension(extensionId);
      }
    });

    this.bridge.on('execute-command', async (command: string, args: any[]) => {
      // Command execution will be handled by vscode.commands API
      console.error(`[ExtensionHost] Execute command: ${command}`, args);
    });

    // Extension listing - return all loaded extensions
    this.bridge.on('list-extensions', async (payload: any, respond: Function) => {
      if (!this.extensionManager) {
        respond({ extensions: [] });
        return;
      }

      const extensions = this.extensionManager.getAllExtensions().map(ext => ({
        id: ext.id,
        name: ext.manifest.name,
        displayName: ext.manifest.displayName || ext.manifest.name,
        version: ext.manifest.version,
        publisher: ext.manifest.publisher,
        description: ext.manifest.description || null,
        path: ext.extensionPath,
        isActive: ext.isActive,
        activationEvents: ext.manifest.activationEvents || [],
        contributes: ext.manifest.contributes || {}
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
