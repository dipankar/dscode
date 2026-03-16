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
  private extensionManager: ExtensionManager;

  constructor() {
    this.bridge = new ExtensionHostBridge();
    this.extensionManager = new ExtensionManager(this.bridge);
  }

  async start() {
    console.log('[Extension Host] Starting...');

    // Initialize IPC bridge
    await this.bridge.connect();
    console.log('[Extension Host] Bridge connected');

    // Set up message handlers
    this.setupMessageHandlers();

    // Load installed extensions
    await this.extensionManager.loadExtensions();
    console.log('[Extension Host] Extensions loaded');

    console.log('[Extension Host] Ready');
  }

  private setupMessageHandlers() {
    this.bridge.on('activate-extension', async (extensionId: string) => {
      await this.extensionManager.activateExtension(extensionId);
    });

    this.bridge.on('deactivate-extension', async (extensionId: string) => {
      await this.extensionManager.deactivateExtension(extensionId);
    });

    this.bridge.on('execute-command', async (command: string, args: any[]) => {
      // Command execution will be handled by vscode.commands API
      console.log(`[Extension Host] Execute command: ${command}`, args);
    });
  }

  async shutdown() {
    console.log('[Extension Host] Shutting down...');
    await this.extensionManager.deactivateAll();
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
