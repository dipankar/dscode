/**
 * IPC Bridge between Extension Host and Main App
 *
 * Uses stdio for cross-platform IPC (same as VS Code).
 * Messages are sent via stdout and received via stdin.
 */

import { EventEmitter } from 'events';
import * as readline from 'readline';

export interface IPCMessage {
  id: string;
  type: string;
  payload: any;
}

export class ExtensionHostBridge extends EventEmitter {
  private messageId = 0;
  private pendingRequests = new Map<string, { resolve: Function; reject: Function }>();
  private isConnected = false;
  private rl?: readline.Interface;

  constructor() {
    super();
  }

  async connect() {
    try {
      console.error('[Bridge] Connecting via stdio...');

      // Create readline interface for stdin
      this.rl = readline.createInterface({
        input: process.stdin,
        output: undefined, // Don't echo to stdout
        terminal: false,
      });

      // Listen for messages from main app
      this.rl.on('line', (line: string) => {
        try {
          const message: IPCMessage = JSON.parse(line);
          this.handleMessage(message);
        } catch (error) {
          console.error('[Bridge] Error parsing message:', error);
        }
      });

      this.isConnected = true;
      console.error('[Bridge] Connected successfully');

      // Send ready signal
      await this.send('ready', {});
    } catch (error) {
      console.error('[Bridge] Connection failed:', error);
      throw error;
    }
  }

  async disconnect() {
    this.isConnected = false;

    if (this.rl) {
      this.rl.close();
      this.rl = undefined;
    }

    console.error('[Bridge] Disconnected');
  }

  /**
   * Send a one-way message to the main app
   */
  async send(type: string, payload: any): Promise<void> {
    const message: IPCMessage = {
      id: `msg_${this.messageId++}`,
      type,
      payload,
    };

    this.sendMessage(message);
  }

  /**
   * Send a request and wait for response
   */
  async request(type: string, payload: any): Promise<any> {
    if (!this.isConnected) {
      throw new Error('Bridge not connected');
    }

    const id = `req_${this.messageId++}`;

    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timeout: ${type}`));
      }, 30000);

      this.pendingRequests.set(id, {
        resolve: (result: any) => {
          clearTimeout(timeoutId);
          this.pendingRequests.delete(id);
          resolve(result);
        },
        reject: (error: any) => {
          clearTimeout(timeoutId);
          this.pendingRequests.delete(id);
          reject(error);
        },
      });

      const message: IPCMessage = { id, type, payload };
      this.sendMessage(message);
    });
  }

  /**
   * Handle incoming messages
   */
  private handleMessage(message: IPCMessage) {
    // Check if this is a response to a pending request
    const pending = this.pendingRequests.get(message.id);
    if (pending) {
      if (message.type.endsWith('-error')) {
        pending.reject(new Error(message.payload.error || 'Unknown error'));
      } else {
        pending.resolve(message.payload);
      }
      return;
    }

    // Otherwise, emit as event for handlers
    this.emit(message.type, message.payload, (response: any) => {
      // Send response back
      const responseMessage: IPCMessage = {
        id: message.id,
        type: `${message.type}-response`,
        payload: response,
      };
      this.sendMessage(responseMessage);
    });
  }

  /**
   * Send a message via stdout
   */
  private sendMessage(message: IPCMessage) {
    const json = JSON.stringify(message);
    process.stdout.write(json + '\n');
  }
}
