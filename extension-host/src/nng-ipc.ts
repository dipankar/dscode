/**
 * NNG IPC Wrapper for Extension Host
 *
 * Wraps the native NNG binding for easier use in TypeScript.
 */

// @ts-ignore - Native module
const nngNative = require('../build/Release/nng_native.node');

export interface IPCMessage {
  id: string;
  type: string;
  payload: any;
}

export class NngIPC {
  private listening = false;
  private connected = false;
  private messageHandlers: Map<string, (payload: any) => Promise<any>> = new Map();
  private messageId = 0;

  /**
   * Start listening on the IPC endpoint (REP socket)
   */
  listen(url: string): void {
    if (this.listening) {
      throw new Error('Already listening');
    }

    nngNative.listen(url);
    this.listening = true;
    console.log(`[NNG IPC] Listening on ${url}`);

    // Start message loop
    this.startMessageLoop();
  }

  /**
   * Connect to IPC endpoint (REQ socket)
   */
  connect(url: string): void {
    if (this.connected) {
      throw new Error('Already connected');
    }

    nngNative.connect(url);
    this.connected = true;
    console.log(`[NNG IPC] Connected to ${url}`);
  }

  /**
   * Register a message handler (for incoming requests)
   */
  on(messageType: string, handler: (payload: any) => Promise<any>): void {
    this.messageHandlers.set(messageType, handler);
  }

  /**
   * Send a request to Tauri and wait for response (REQ socket)
   */
  async request(msgType: string, payload: any): Promise<any> {
    if (!this.connected) {
      throw new Error('Not connected. Call connect() first.');
    }

    // Generate unique message ID
    this.messageId++;
    const id = `req_${this.messageId}`;

    const message: IPCMessage = {
      id,
      type: msgType,
      payload,
    };

    // Serialize and send
    const messageStr = JSON.stringify(message);
    const responseStr = nngNative.request(messageStr);

    // Parse response
    const response: IPCMessage = JSON.parse(responseStr);

    // Check for error response
    if (response.type.endsWith('-error')) {
      const error = response.payload.error || 'Unknown error';
      throw new Error(error);
    }

    return response.payload;
  }

  /**
   * Message processing loop (runs in background)
   */
  private async startMessageLoop(): Promise<void> {
    while (this.listening) {
      try {
        // Receive message (blocking)
        const messageStr = nngNative.receive();
        const message: IPCMessage = JSON.parse(messageStr);

        console.log(`[NNG IPC] Received message: ${message.type} (id: ${message.id})`);

        // Process message
        const response = await this.handleMessage(message);

        // Send response
        const responseStr = JSON.stringify(response);
        nngNative.send(responseStr);

      } catch (error) {
        console.error('[NNG IPC] Error in message loop:', error);

        // Try to send error response
        try {
          const errorResponse: IPCMessage = {
            id: 'error',
            type: 'error',
            payload: {
              error: error instanceof Error ? error.message : String(error),
            },
          };
          nngNative.send(JSON.stringify(errorResponse));
        } catch (sendError) {
          console.error('[NNG IPC] Failed to send error response:', sendError);
        }
      }
    }
  }

  /**
   * Handle a received message
   */
  private async handleMessage(message: IPCMessage): Promise<IPCMessage> {
    const handler = this.messageHandlers.get(message.type);

    if (!handler) {
      return {
        id: message.id,
        type: `${message.type}-error`,
        payload: {
          error: `No handler registered for message type: ${message.type}`,
        },
      };
    }

    try {
      const result = await handler(message.payload);
      return {
        id: message.id,
        type: `${message.type}-response`,
        payload: result,
      };
    } catch (error) {
      return {
        id: message.id,
        type: `${message.type}-error`,
        payload: {
          error: error instanceof Error ? error.message : String(error),
        },
      };
    }
  }

  /**
   * Close the IPC connections
   */
  close(): void {
    if (this.listening) {
      this.listening = false;
      nngNative.close();
    }
    if (this.connected) {
      this.connected = false;
      nngNative.closeReq();
    }
    console.log('[NNG IPC] Connections closed');
  }
}
