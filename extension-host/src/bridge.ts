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
    // Get IPC URLs from environment variables or use defaults
    this.ipcUrl = process.env.DSCODE_IPC_URL || 'ipc:///tmp/dscode-extension-host.ipc';
    this.incomingIpcUrl = process.env.DSCODE_INCOMING_IPC_URL || 'ipc:///tmp/dscode-incoming-extension-host.ipc';
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
