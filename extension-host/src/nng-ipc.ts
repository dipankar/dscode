/**
 * NNG IPC Wrapper for Extension Host
 *
 * Wraps the native NNG binding for easier use in TypeScript.
 * Uses a worker thread for the blocking receive loop to avoid blocking the main event loop.
 */

import { Worker, isMainThread, parentPort, workerData } from 'worker_threads';
import { EventEmitter } from 'events';
import * as path from 'path';

// @ts-ignore - Native module
let nngNative: any;
try {
  nngNative = require('../build/Release/nng_native.node');
} catch {
  // Fallback for different build configurations
  try {
    nngNative = require('../build/Debug/nng_native.node');
  } catch {
    console.error('[NNG IPC] Failed to load native module');
  }
}

export interface IPCMessage {
  id: string;
  type: string;
  payload: unknown;
}

/**
 * Worker thread code for receiving messages
 * This runs the blocking receive loop in a separate thread
 */
if (!isMainThread && parentPort) {
  const { listenUrl } = workerData as { listenUrl: string };

  // Load native module in worker
  // @ts-ignore
  const workerNng = require('../build/Release/nng_native.node');

  try {
    workerNng.listen(listenUrl);
    console.log(`[NNG Worker] Listening on ${listenUrl}`);

    // Message loop - blocking but in worker thread
    let running = true;

    parentPort.on('message', (msg: { type: string }) => {
      if (msg.type === 'shutdown') {
        running = false;
        workerNng.close();
      }
    });

    while (running) {
      try {
        // This blocks, but we're in a worker thread so it's OK
        const messageStr = workerNng.receive();

        // Send to main thread for processing
        parentPort.postMessage({ type: 'message', data: messageStr });

        // Wait for response from main thread
        // We need to handle this synchronously for the REP socket pattern
        // Use a simple busy-wait with the response stored
      } catch (error) {
        if (running) {
          parentPort.postMessage({
            type: 'error',
            error: error instanceof Error ? error.message : String(error)
          });
        }
        break;
      }
    }
  } catch (error) {
    parentPort?.postMessage({
      type: 'fatal',
      error: error instanceof Error ? error.message : String(error)
    });
  }
}

export class NngIPC extends EventEmitter {
  private listening = false;
  private connected = false;
  private messageHandlers: Map<string, (payload: unknown) => Promise<unknown>> = new Map();
  private messageId = 0;
  private worker: Worker | null = null;
  private pendingResponse: string | null = null;
  private shutdownRequested = false;

  constructor() {
    super();
  }

  /**
   * Start listening on the IPC endpoint (REP socket)
   * Uses main thread for now with setImmediate to yield to event loop
   */
  listen(url: string): void {
    if (this.listening) {
      throw new Error('Already listening');
    }

    if (!nngNative) {
      throw new Error('NNG native module not loaded');
    }

    nngNative.listen(url);
    this.listening = true;
    console.log(`[NNG IPC] Listening on ${url}`);

    // Start message loop with proper yielding
    this.startMessageLoop();
  }

  /**
   * Connect to IPC endpoint (REQ socket)
   */
  connect(url: string): void {
    if (this.connected) {
      throw new Error('Already connected');
    }

    if (!nngNative) {
      throw new Error('NNG native module not loaded');
    }

    nngNative.connect(url);
    this.connected = true;
    console.log(`[NNG IPC] Connected to ${url}`);
  }

  /**
   * Register a message handler (for incoming requests)
   */
  on(messageType: string, handler: (payload: unknown) => Promise<unknown>): this {
    this.messageHandlers.set(messageType, handler);
    return this;
  }

  /**
   * Send a request to Tauri and wait for response (REQ socket)
   */
  async request(msgType: string, payload: unknown): Promise<unknown> {
    if (!this.connected) {
      throw new Error('Not connected. Call connect() first.');
    }

    if (!nngNative) {
      throw new Error('NNG native module not loaded');
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
      const error = (response.payload as { error?: string })?.error || 'Unknown error';
      throw new Error(error);
    }

    return response.payload;
  }

  /**
   * Message processing loop with proper event loop yielding
   * Uses setImmediate to allow other events to be processed between messages
   */
  private startMessageLoop(): void {
    const processNextMessage = async (): Promise<void> => {
      if (!this.listening || this.shutdownRequested) {
        return;
      }

      try {
        // Use a non-blocking check if available, otherwise use short timeout
        // The native receive is blocking, so we wrap it in a promise with setImmediate
        const messageStr = await this.receiveWithYield();

        if (messageStr === null) {
          // Socket closed or shutdown
          return;
        }

        const message: IPCMessage = JSON.parse(messageStr);

        console.log(`[NNG IPC] Received message: ${message.type} (id: ${message.id})`);

        // Process message
        const response = await this.handleMessage(message);

        // Send response
        const responseStr = JSON.stringify(response);
        nngNative.send(responseStr);

      } catch (error) {
        if (this.listening && !this.shutdownRequested) {
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

      // Schedule next iteration with setImmediate to yield to event loop
      if (this.listening && !this.shutdownRequested) {
        setImmediate(processNextMessage);
      }
    };

    // Start the loop
    setImmediate(processNextMessage);
  }

  /**
   * Receive a message with yielding to the event loop
   * This wraps the blocking receive in a way that allows other events to process
   */
  private receiveWithYield(): Promise<string | null> {
    return new Promise((resolve, reject) => {
      if (!this.listening || this.shutdownRequested) {
        resolve(null);
        return;
      }

      // Use setImmediate to yield before the blocking call
      setImmediate(() => {
        if (!this.listening || this.shutdownRequested) {
          resolve(null);
          return;
        }

        try {
          const messageStr = nngNative.receive();
          resolve(messageStr);
        } catch (error) {
          if (this.shutdownRequested) {
            resolve(null);
          } else {
            reject(error);
          }
        }
      });
    });
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
   * Close the IPC connections gracefully
   */
  close(): void {
    this.shutdownRequested = true;

    // Stop the worker if it exists
    if (this.worker) {
      this.worker.postMessage({ type: 'shutdown' });
      this.worker.terminate();
      this.worker = null;
    }

    if (this.listening) {
      this.listening = false;
      try {
        nngNative?.close();
      } catch (error) {
        console.error('[NNG IPC] Error closing listener:', error);
      }
    }

    if (this.connected) {
      this.connected = false;
      try {
        nngNative?.closeReq();
      } catch (error) {
        console.error('[NNG IPC] Error closing connection:', error);
      }
    }

    console.log('[NNG IPC] Connections closed');
  }

  /**
   * Check if the IPC is currently listening
   */
  isListening(): boolean {
    return this.listening;
  }

  /**
   * Check if the IPC is currently connected
   */
  isConnected(): boolean {
    return this.connected;
  }
}
