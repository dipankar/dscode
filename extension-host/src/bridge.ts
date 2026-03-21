/**
 * IPC Bridge between Extension Host and Main App
 *
 * Uses NNG (nanomsg-next-generation) for IPC.
 * Extension host listens on REP socket, Tauri connects with REQ socket.
 */

import { EventEmitter } from 'events';
import { NngIPC } from './nng-ipc';

export interface IPCMessage {
  id: string;
  type: string;
  payload: any;
}

export class ExtensionHostBridge extends EventEmitter {
  private nng: NngIPC;
  private isConnected = false;
  private ipcUrl: string;
  private incomingIpcUrl: string;

  constructor() {
    super();
    this.nng = new NngIPC();
    // Get IPC URLs from environment variables or use platform-safe defaults
    const windowsPipeRoot = '\\.\pipe\\';
    const defaultOutgoing =
      process.platform === 'win32'
        ? `ipc://${windowsPipeRoot}dscode-extension-host`
        : 'ipc:///tmp/dscode-extension-host.ipc';
    const defaultIncoming =
      process.platform === 'win32'
        ? `ipc://${windowsPipeRoot}dscode-incoming-extension-host`
        : 'ipc:///tmp/dscode-incoming-extension-host.ipc';
    this.ipcUrl = process.env.DSCODE_IPC_URL || defaultOutgoing;
    this.incomingIpcUrl = process.env.DSCODE_INCOMING_IPC_URL || defaultIncoming;
  }

  async connect() {
    try {
      console.error('[Bridge] Connecting via NNG...');
      console.error('[Bridge] Outgoing IPC URL (ExtHost listens):', this.ipcUrl);
      console.error('[Bridge] Incoming IPC URL (ExtHost connects):', this.incomingIpcUrl);

      // Start listening for requests from Tauri (REP socket)
      this.nng.listen(this.ipcUrl);

      // Connect to Tauri for sending requests (REQ socket)
      this.nng.connect(this.incomingIpcUrl);

      this.isConnected = true;

      // Register message handlers
      this.setupHandlers();

      console.error('[Bridge] Bidirectional connection established');
    } catch (error) {
      console.error('[Bridge] Connection failed:', error);
      throw error;
    }
  }

  async disconnect() {
    this.isConnected = false;
    this.nng.close();
    console.error('[Bridge] Disconnected');
  }

  /**
   * Setup NNG message handlers
   */
  private setupHandlers() {
    // Extension lifecycle: activate
    this.nng.on('activate-extension', async (payload: any) => {
      const extensionId = payload?.extensionId;
      if (!extensionId) {
        throw new Error('Missing extensionId in activate-extension payload');
      }

      await this.emitAsync('activate-extension', extensionId);
      return { success: true };
    });

    // Extension lifecycle: deactivate
    this.nng.on('deactivate-extension', async (payload: any) => {
      const extensionId = payload?.extensionId;
      if (!extensionId) {
        throw new Error('Missing extensionId in deactivate-extension payload');
      }

      await this.emitAsync('deactivate-extension', extensionId);
      return { success: true };
    });

    // Extension management: list installed extensions
    this.nng.on('list-extensions', async (payload: any) => {
      return new Promise((resolve, reject) => {
        const respond = (response: any) => resolve(response);
        try {
          const handled = this.emit('list-extensions', payload, respond);
          if (!handled) {
            reject(new Error('No listeners registered for list-extensions'));
          }
        } catch (error) {
          reject(error);
        }
      });
    });

    // Extension management: reload extensions after install/uninstall
    this.nng.on('reload-extensions', async (payload: any) => {
      return new Promise((resolve, reject) => {
        const respond = (response: any) => resolve(response);
        try {
          const handled = this.emit('reload-extensions', payload, respond);
          if (!handled) {
            reject(new Error('No listeners registered for reload-extensions'));
          }
        } catch (error) {
          reject(error);
        }
      });
    });

    // Extension management: uninstall extension
    this.nng.on('uninstall-extension', async (payload: any) => {
      return new Promise((resolve, reject) => {
        const respond = (response: any) => resolve(response);
        try {
          const handled = this.emit('uninstall-extension', payload, respond);
          if (!handled) {
            reject(new Error('No listeners registered for uninstall-extension'));
          }
        } catch (error) {
          reject(error);
        }
      });
    });

    // Handle tree view requests
    this.nng.on('treeView:getChildren', async (payload: any) => {
      console.error('[Bridge] Received treeView:getChildren request:', payload);

      return new Promise((resolve) => {
        // Emit event for extension manager to handle
        this.emit('treeView:getChildren', payload, (response: any) => {
          resolve(response);
        });
      });
    });

    this.nng.on('treeView:event', async (payload: any) => {
      console.error('[Bridge] Received treeView:event:', payload);

      return new Promise((resolve) => {
        this.emit('treeView:event', payload, (response: any) => {
          resolve(response ?? { success: true });
        });
      });
    });

    // Handle command execution requests
    this.nng.on('executeCommand', async (payload: any) => {
      console.error('[Bridge] Received executeCommand request:', payload);

      return new Promise((resolve) => {
        // Emit event for extension manager to handle
        this.emit('executeCommand', payload, (response: any) => {
          resolve(response);
        });
      });
    });

    this.nng.on('configuration-changed', async (payload: any) => {
      return new Promise((resolve) => {
        this.emit('configurationChanged', payload);
        resolve({ success: true });
      });
    });

    this.nng.on('fsWatcher:event', async (payload: any) => {
      return new Promise((resolve) => {
        this.emit('fsWatcher:event', payload);
        resolve({ success: true });
      });
    });
  }

  /**
   * Send a one-way message to Tauri (fire and forget)
   * For one-way messages, we still use request() but ignore the response
   */
  async send(type: string, payload: any): Promise<void> {
    if (!this.isConnected) {
      throw new Error('Bridge not connected');
    }

    try {
      // For one-way messages, we still need to wait for ack due to REQ/REP pattern
      await this.nng.request(type, payload);
    } catch (error) {
      console.error('[Bridge] Failed to send message:', error);
      throw error;
    }
  }

  /**
   * Emit an event and await async listeners
   */
  private async emitAsync(event: string, ...args: any[]): Promise<void> {
    const listeners = this.listeners(event);
    if (listeners.length === 0) {
      throw new Error(`No listeners registered for ${event}`);
    }
    for (const listener of listeners) {
      const result = (listener as (...innerArgs: any[]) => unknown)(...args);
      if (result instanceof Promise) {
        await result;
      }
    }
  }

  /**
   * Send a request to Tauri and wait for response
   */
  async request(type: string, payload: any): Promise<any> {
    if (!this.isConnected) {
      throw new Error('Bridge not connected');
    }

    try {
      const response = await this.nng.request(type, payload);
      return response;
    } catch (error) {
      console.error('[Bridge] Request failed:', error);
      throw error;
    }
  }
}
