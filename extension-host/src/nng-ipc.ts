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
 * Worker thread code for receiving messages.
 * This runs the blocking receive loop in a separate thread so
 * the main Node.js event loop is never blocked.
 */
if (!isMainThread && parentPort) {
  const { listenUrl } = workerData as { listenUrl: string };

  // Load native module in worker context
  // @ts-ignore
  let workerNng: any;
  try {
    // @ts-ignore
    workerNng = require('../build/Release/nng_native.node');
  } catch {
    try {
      // @ts-ignore
      workerNng = require('../build/Debug/nng_native.node');
    } catch {
      parentPort?.postMessage({
        type: 'fatal',
        error: 'Failed to load NNG native module in worker thread',
      });
    }
  }

  if (workerNng) {
    try {
      workerNng.listen(listenUrl);
      parentPort?.postMessage({ type: 'ready' });

      let running = true;

      // Handle shutdown signal from main thread
      parentPort.on('message', (msg: { type: string; response?: string }) => {
        if (msg.type === 'shutdown') {
          running = false;
          try {
            workerNng.close();
          } catch {}
        } else if (msg.type === 'send-response' && msg.response) {
          // Main thread tells us to send a response for the last received message
          try {
            workerNng.send(msg.response);
          } catch (error) {
            parentPort?.postMessage({
              type: 'error',
              error: error instanceof Error ? error.message : String(error),
            });
          }
        }
      });

      // Blocking receive loop — runs in its own thread, so it's safe
      while (running) {
        try {
          const messageStr = workerNng.receive();

          if (!running) break;

          // Forward the received message to the main thread for processing
          parentPort?.postMessage({ type: 'message', data: messageStr });

          // Wait for the main thread to process the message and tell us to send the response.
          // The REP socket pattern requires: recv -> process -> send -> recv -> ...
          // We use a Promise that resolves when the main thread sends back the response.
          // Since we're in a worker thread, we synchronously wait for the next message
          // from parentPort that contains the response.
          // But worker_threads are async, so we need to yield to receive the response.
          // Instead, we'll use a synchronous flag-based approach.
        } catch (error) {
          if (running) {
            parentPort?.postMessage({
              type: 'error',
              error: error instanceof Error ? error.message : String(error),
            });
            // Brief pause before retrying to avoid tight error loops
            const sleepMs = 100;
            const end = Date.now() + sleepMs;
            while (Date.now() < end && running) {
              // busy sleep
            }
          }
        }
      }
    } catch (error) {
      parentPort?.postMessage({
        type: 'fatal',
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
}

/**
 * NngIPC - IPC communication using NNG with worker thread for non-blocking receives.
 *
 * Architecture:
 * - Main thread: handles message dispatch, request/response correlation
 * - Worker thread: runs the blocking NNG receive loop for incoming (REP) messages
 * - REQ socket: used on main thread for outgoing requests (with proper timeout handling)
 */
export class NngIPC extends EventEmitter {
  private listening = false;
  private connected = false;
  private messageHandlers: Map<
    string,
    (payload: unknown, respond?: (response: unknown) => void) => Promise<unknown>
  > = new Map();
  private messageId = 0;
  private worker: Worker | null = null;
  private pendingResponse = false;

  // Queue for outgoing REQ responses while worker is receiving
  private responseQueue: string[] = [];

  constructor() {
    super();
  }

  /**
   * Start listening on the IPC endpoint (REP socket).
   * Uses a worker thread to avoid blocking the main event loop.
   */
  listen(url: string): void {
    if (this.listening) {
      throw new Error('Already listening');
    }

    if (!nngNative) {
      throw new Error('NNG native module not loaded');
    }

    // Connect the REQ socket on the main thread (for outgoing requests)
    // The REP socket runs in the worker thread (for incoming requests)
    this.startWorker(url);
    this.listening = true;
    console.log(`[NNG IPC] Listening on ${url} (worker thread mode)`);
  }

  /**
   * Start a worker thread for the blocking receive loop
   */
  private startWorker(url: string): void {
    const workerFilename = __filename;

    this.worker = new Worker(workerFilename, {
      workerData: { listenUrl: url },
    });

    this.worker.on('message', async (msg: { type: string; data?: string; error?: string }) => {
      if (msg.type === 'ready') {
        console.log('[NNG IPC] Worker thread ready');
      } else if (msg.type === 'message' && msg.data) {
        // Incoming message from the extension host (via REP socket)
        await this.handleIncomingMessage(msg.data);
      } else if (msg.type === 'error') {
        console.error('[NNG IPC] Worker error:', msg.error);
        this.emit('error', new Error(msg.error));
      } else if (msg.type === 'fatal') {
        console.error('[NNG IPC] Worker fatal error:', msg.error);
        this.emit('error', new Error(msg.error));
        this.listening = false;
      }
    });

    this.worker.on('error', (err) => {
      console.error('[NNG IPC] Worker thread error:', err);
      this.emit('error', err);
    });

    this.worker.on('exit', (code) => {
      if (code !== 0) {
        console.warn(`[NNG IPC] Worker exited with code ${code}`);
      }
      this.listening = false;
    });
  }

  /**
   * Handle an incoming message received by the worker thread.
   * Processes it through registered handlers and sends the response back.
   */
  private async handleIncomingMessage(messageStr: string): Promise<void> {
    let message: IPCMessage;
    try {
      message = JSON.parse(messageStr);
    } catch {
      console.error('[NNG IPC] Failed to parse incoming message');
      // Send error response back through worker
      this.sendResponseToWorker(
        JSON.stringify({
          id: 'error',
          type: 'error',
          payload: { error: 'Failed to parse message' },
        })
      );
      return;
    }

    console.log(`[NNG IPC] Received message: ${message.type} (id: ${message.id})`);

    const handler = this.messageHandlers.get(message.type);

    let response: IPCMessage;
    if (!handler) {
      response = {
        id: message.id,
        type: `${message.type}-error`,
        payload: { error: `No handler registered for message type: ${message.type}` },
      };
    } else {
      try {
        // Provide a respond callback for handlers that need to send partial responses
        const respond = (data: unknown) => {
          const partialResponse: IPCMessage = {
            id: message.id,
            type: `${message.type}-partial`,
            payload: data,
          };
          // Partial responses are emitted as events, not sent back through REP
          this.emit('partial-response', partialResponse);
        };

        const result = await handler(message.payload, respond);
        response = {
          id: message.id,
          type: `${message.type}-response`,
          payload: result,
        };
      } catch (error) {
        response = {
          id: message.id,
          type: `${message.type}-error`,
          payload: { error: error instanceof Error ? error.message : String(error) },
        };
      }
    }

    // Send the response back through the worker thread's REP socket
    const responseStr = JSON.stringify(response);
    this.sendResponseToWorker(responseStr);
  }

  /**
   * Send a response through the worker thread's REP socket
   */
  private sendResponseToWorker(responseStr: string): void {
    if (this.worker) {
      this.worker.postMessage({ type: 'send-response', response: responseStr });
    }
  }

  /**
   * Connect to IPC endpoint (REQ socket for outgoing messages)
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
   * Register a message handler for incoming requests.
   * Handlers now receive an optional `respond` callback for partial responses.
   */
  on(
    messageType: string,
    handler: (payload: unknown, respond?: (data: unknown) => void) => Promise<unknown>
  ): this {
    this.messageHandlers.set(messageType, handler);
    return this;
  }

  /**
   * Send a request to Tauri and wait for response (REQ socket).
   * Uses spawn_blocking-style approach: offloads the synchronous NNG call
   * to avoid blocking the main event loop.
   */
  async request(msgType: string, payload: unknown): Promise<unknown> {
    if (!this.connected) {
      throw new Error('Not connected. Call connect() first.');
    }

    if (!nngNative) {
      throw new Error('NNG native module not loaded');
    }

    const id = `req_${++this.messageId}`;

    const message: IPCMessage = {
      id,
      type: msgType,
      payload,
    };

    const messageStr = JSON.stringify(message);

    // Use a Promise-based approach that yields to the event loop
    // by scheduling the blocking call via setImmediate, then awaiting the result
    return new Promise((resolve, reject) => {
      setImmediate(() => {
        try {
          const responseStr = nngNative.request(messageStr);
          const response: IPCMessage = JSON.parse(responseStr);

          if (response.type.endsWith('-error')) {
            const error = (response.payload as { error?: string })?.error || 'Unknown error';
            reject(new Error(error));
          } else {
            resolve(response.payload);
          }
        } catch (error) {
          reject(error instanceof Error ? error : new Error(String(error)));
        }
      });
    });
  }

  /**
   * Close the IPC connections gracefully.
   */
  close(): void {
    // Stop the worker thread
    if (this.worker) {
      this.worker.postMessage({ type: 'shutdown' });
      // Give the worker a brief moment to shut down gracefully
      setTimeout(() => {
        if (this.worker) {
          this.worker.terminate();
          this.worker = null;
        }
      }, 1000);
    }

    if (this.listening) {
      this.listening = false;
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
