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

  constructor() {
    super();
    this.nng = new NngIPC();
    // Get IPC URL from environment variable or use default
    this.ipcUrl = process.env.DSCODE_IPC_URL || 'ipc:///tmp/dscode-extension-host.ipc';
  }

  async connect() {
    try {
      console.error('[Bridge] Connecting via NNG...');
      console.error('[Bridge] IPC URL:', this.ipcUrl);

      // Start listening for requests from Tauri
      this.nng.listen(this.ipcUrl);
      this.isConnected = true;

      // Register message handlers
      this.setupHandlers();

      console.error('[Bridge] Connected successfully on', this.ipcUrl);
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
   * Send a one-way message to Tauri
   * Note: Not supported with current REQ/REP pattern.
   * TODO: Add second socket for extension-host-initiated messages
   */
  async send(type: string, payload: any): Promise<void> {
    console.error('[Bridge] Warning: send() not supported with NNG REQ/REP pattern');
    console.error('[Bridge] Attempted to send:', type, payload);
    // TODO: Implement with second socket (REQ from extension host side)
  }

  /**
   * Send a request to Tauri and wait for response
   * Note: Not supported with current REQ/REP pattern.
   * TODO: Add second socket for extension-host-initiated requests
   */
  async request(type: string, payload: any): Promise<any> {
    console.error('[Bridge] Warning: request() not supported with NNG REQ/REP pattern');
    console.error('[Bridge] Attempted to request:', type, payload);
    // TODO: Implement with second socket (REQ from extension host side)
    return null;
  }

  /**
   * Note: Current NNG REQ/REP pattern allows Tauri to initiate requests to extension host,
   * but not vice versa. For full bidirectional communication, we need to add a second
   * socket pair where extension host has REQ and Tauri has REP.
   */
}
