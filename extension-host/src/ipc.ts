import * as net from 'net';
import { EventEmitter } from 'events';

const MAX_MESSAGE_SIZE = 50 * 1024 * 1024;

export interface IPCMessage {
  id: string;
  type: string;
  payload: unknown;
}

/**
 * STATE MACHINE: IpcConnection
 *
 * Tracks the connection state of the bidirectional Unix domain socket IPC
 * between the extension host and the Tauri backend.
 *
 * State Diagram:
 *
 *   Disconnected ──────► Connecting ──────► Connected
 *       ▲                    │                  │
 *       │        (max retry) │    (socket error)│
 *       │                    ▼                  ▼
 *       └────────────────────┴─────── Reconnecting
 *                                         │
 *                    (max retry failed)   │
 *       Disconnected ◄───────────────────┘
 *
 * Transitions:
 *   Disconnected -> Connecting    (connect() called)
 *   Connecting   -> Connected     (both sockets established successfully)
 *   Connecting   -> Disconnected  (connection failed after max retries)
 *   Connected    -> Reconnecting  (socket error/close detected while connected)
 *   Connected    -> Disconnected  (close() called intentionally)
 *   Reconnecting -> Connected     (reconnection succeeded)
 *   Reconnecting -> Disconnected  (reconnection failed after retries)
 *
 * Concurrency Invariant:
 *   State transitions happen synchronously in Node.js event handlers (socket
 *   'close', 'error' events). The `request()` method checks state before
 *   sending. Socket close handlers transition state BEFORE rejecting pending
 *   requests, ensuring no new requests are accepted during cleanup.
 *
 * Interruption Table:
 * ┌──────────────┬──────────────────────────────────────────────────────────┐
 * │ State        │ What happens on socket error, process crash, or close   │
 * ├──────────────┼──────────────────────────────────────────────────────────┤
 * │ Disconnected │ No-op. Already disconnected. No resources to clean up.  │
 * ├──────────────┼──────────────────────────────────────────────────────────┤
 * │ Connecting   │ Retry with linear backoff (10 attempts, 200ms * N).     │
 * │              │ After exhaustion: -> Disconnected. All connection        │
 * │              │ promises rejected. No pending requests exist yet.       │
 * ├──────────────┼──────────────────────────────────────────────────────────┤
 * │ Connected    │ All pending requests rejected with 'connection closed'  │
 * │              │ error. Timers cleared for each pending request.         │
 * │              │ -> Disconnected. Caller must reconnect explicitly.      │
 * │              │ Backend (Rust) will detect socket close in its read     │
 * │              │ loop and set alive=false on ExtensionIpc.               │
 * ├──────────────┼──────────────────────────────────────────────────────────┤
 * │ Reconnecting │ Same as Connecting but entered from Connected state.    │
 * │              │ If fails: -> Disconnected with all pending rejected.    │
 * └──────────────┴──────────────────────────────────────────────────────────┘
 *
 * Pending Request Lifecycle:
 *   Created -> timer started -> ONE of:
 *     1. Response received: clearTimeout(timer), resolve promise
 *     2. Timeout fires (30s): delete from map, reject with timeout error
 *     3. Socket closes: clearTimeout(timer), reject with connection error
 *   Exactly ONE of these three outcomes occurs per request.
 */
type IpcConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Reconnecting';

function writeMessage(socket: net.Socket, msg: IPCMessage): void {
  const body = Buffer.from(JSON.stringify(msg), 'utf8');
  const header = Buffer.alloc(4);
  header.writeUInt32BE(body.length, 0);
  const packet = Buffer.concat([header, body]);
  socket.write(packet);
}

function concatUint8Array(a: Uint8Array, b: Uint8Array): Uint8Array {
  const result = new Uint8Array(a.length + b.length);
  result.set(a, 0);
  result.set(b, a.length);
  return result;
}

function parseMessagesFromBuffer(data: Uint8Array): {
  messages: IPCMessage[];
  remaining: Uint8Array;
} {
  const messages: IPCMessage[] = [];
  let offset = 0;

  while (offset + 4 <= data.length) {
    const len =
      (data[offset] << 24) | (data[offset + 1] << 16) | (data[offset + 2] << 8) | data[offset + 3];
    if (len === 0 || offset + 4 + len > data.length) {
      break;
    }

    if (len > MAX_MESSAGE_SIZE) {
      console.error(`[SocketIPC] Message too large: ${len} bytes (max ${MAX_MESSAGE_SIZE})`);
      offset += 4 + len;
      continue;
    }

    const bodyBytes = data.subarray(offset + 4, offset + 4 + len);
    try {
      messages.push(JSON.parse(new TextDecoder().decode(bodyBytes)));
    } catch {}
    offset += 4 + len;
  }

  const remaining = data.subarray(offset);
  return { messages, remaining: new Uint8Array(remaining) };
}

export class SocketIPC extends EventEmitter {
  private incomingSocket: net.Socket | null = null;
  private outgoingServer: net.Server | null = null;
  private outgoingSocket: net.Socket | null = null;
  private messageHandlers: Map<
    string,
    (payload: unknown, respond?: (response: unknown) => void) => Promise<unknown>
  > = new Map();
  private messageId = 0;
  private pendingRequests: Map<
    string,
    {
      resolve: (value: unknown) => void;
      reject: (reason: unknown) => void;
      timer: ReturnType<typeof setTimeout>;
    }
  > = new Map();
  private outgoingBuffer: Uint8Array = new Uint8Array(0);
  private incomingBuffer: Uint8Array = new Uint8Array(0);
  private state: IpcConnectionState = 'Disconnected';

  async connect(outgoingUrl: string, incomingUrl: string): Promise<void> {
    this.state = 'Connecting';
    const outgoingPath = this.extractPath(outgoingUrl);
    const incomingPath = this.extractPath(incomingUrl);

    try {
      await this.startOutgoingServer(outgoingPath);
      await this.connectIncoming(incomingPath);

      this.state = 'Connected';
      console.error(`[SocketIPC] Connected (outgoing=${outgoingPath}, incoming=${incomingPath})`);
    } catch (error) {
      this.state = 'Disconnected';
      throw error;
    }
  }

  private extractPath(url: string): string {
    if (url.startsWith('ipc://')) {
      return url.slice('ipc://'.length);
    }
    return url;
  }

  private startOutgoingServer(socketPath: string): Promise<void> {
    return new Promise((resolve, reject) => {
      if (process.platform !== 'win32' && require('fs').existsSync(socketPath)) {
        try {
          require('fs').unlinkSync(socketPath);
        } catch {}
      }

      this.outgoingServer = net.createServer((socket) => {
        this.outgoingSocket = socket;
        console.error('[SocketIPC] Tauri connected to outgoing socket');

        socket.on('data', (data: Buffer) => {
          this.outgoingBuffer = concatUint8Array(this.outgoingBuffer, data);
          const { messages, remaining } = parseMessagesFromBuffer(this.outgoingBuffer);
          this.outgoingBuffer = remaining;
          for (const msg of messages) {
            this.handleIncomingRequest(msg, socket);
          }
        });

        socket.on('close', () => {
          this.outgoingSocket = null;
          console.error('[SocketIPC] Outgoing socket closed');
        });

        socket.on('error', (err) => {
          console.error('[SocketIPC] Outgoing socket error:', err.message);
        });

        resolve();
      });

      this.outgoingServer.on('error', (err) => {
        console.error('[SocketIPC] Outgoing server error:', err.message);
        reject(err);
      });

      if (process.platform === 'win32') {
        this.outgoingServer.listen(socketPath);
      } else {
        this.outgoingServer.listen(socketPath);
      }
    });
  }

  private connectIncoming(socketPath: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const maxAttempts = 10;
      let attempt = 0;
      let resolved = false;

      const tryConnect = () => {
        attempt++;
        const socket = net.connect(socketPath, () => {
          this.incomingSocket = socket;
          resolved = true;
          console.error('[SocketIPC] Connected to incoming socket');
          resolve();
        });

        socket.on('data', (data: Buffer) => {
          this.incomingBuffer = concatUint8Array(this.incomingBuffer, data);
          const { messages, remaining } = parseMessagesFromBuffer(this.incomingBuffer);
          this.incomingBuffer = remaining;
          for (const msg of messages) {
            if (msg.type.endsWith('-error')) {
              const pending = this.pendingRequests.get(msg.id);
              if (pending) {
                this.pendingRequests.delete(msg.id);
                clearTimeout(pending.timer);
                const errMsg = (msg.payload as { error?: string })?.error || 'Unknown error';
                pending.reject(new Error(errMsg));
              }
            } else {
              const pending = this.pendingRequests.get(msg.id);
              if (pending) {
                this.pendingRequests.delete(msg.id);
                clearTimeout(pending.timer);
                pending.resolve(msg.payload);
              }
            }
          }
        });

        socket.on('error', (err) => {
          if (!resolved) {
            if (attempt < maxAttempts) {
              console.error(
                `[Bridge] Connection attempt ${attempt} failed, retrying in ${attempt * 200}ms...`
              );
              setTimeout(tryConnect, attempt * 200);
            } else {
              reject(new Error(`Failed to connect after ${maxAttempts} attempts: ${err.message}`));
            }
          } else {
            console.error('[SocketIPC] Incoming socket error after connection:', err.message);
          }
        });

        socket.on('close', () => {
          if (resolved) {
            this.incomingSocket = null;
            this.state = 'Disconnected';
            console.error('[SocketIPC] Incoming socket closed');
            this.rejectAllPending('IPC connection closed');
          }
        });

        socket.setNoDelay(true);
      };

      tryConnect();
    });
  }

  private async handleIncomingRequest(msg: IPCMessage, socket: net.Socket): Promise<void> {
    const handler = this.messageHandlers.get(msg.type);

    let response: IPCMessage;
    if (!handler) {
      response = {
        id: msg.id,
        type: `${msg.type}-error`,
        payload: { error: `No handler registered for message type: ${msg.type}` },
      };
    } else {
      try {
        const respond = (data: unknown) => {
          const partialResponse: IPCMessage = {
            id: msg.id,
            type: `${msg.type}-partial`,
            payload: data,
          };
          this.emit('partial-response', partialResponse);
        };

        const result = await handler(msg.payload, respond);
        response = {
          id: msg.id,
          type: `${msg.type}-response`,
          payload: result,
        };
      } catch (error) {
        response = {
          id: msg.id,
          type: `${msg.type}-error`,
          payload: { error: error instanceof Error ? error.message : String(error) },
        };
      }
    }

    if (socket.writable) {
      writeMessage(socket, response);
    }
  }

  on(
    messageType: string,
    handler: (payload: unknown, respond?: (data: unknown) => void) => Promise<unknown>
  ): this {
    this.messageHandlers.set(messageType, handler);
    return this;
  }

  async request(msgType: string, payload: unknown): Promise<unknown> {
    if (this.state !== 'Connected') {
      throw new Error('IPC not connected (state: ' + this.state + ')');
    }

    if (!this.incomingSocket || !this.incomingSocket.writable) {
      throw new Error('Not connected. Call connect() first.');
    }

    const id = `req_${++this.messageId}`;
    const message: IPCMessage = { id, type: msgType, payload };

    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error(`Request '${msgType}' timed out after 30s (id: ${id})`));
        }
      }, 30000);
      this.pendingRequests.set(id, { resolve, reject, timer });
      writeMessage(this.incomingSocket!, message);
    });
  }

  async send(msgType: string, payload: unknown): Promise<void> {
    await this.request(msgType, payload);
  }

  private rejectAllPending(reason: string): void {
    for (const [id, pending] of this.pendingRequests) {
      clearTimeout(pending.timer);
      pending.reject(new Error(reason));
    }
    this.pendingRequests.clear();
  }

  close(): void {
    this.rejectAllPending('IPC shutting down');

    if (this.incomingSocket) {
      this.incomingSocket.destroy();
      this.incomingSocket = null;
    }

    if (this.outgoingSocket) {
      this.outgoingSocket.destroy();
      this.outgoingSocket = null;
    }

    if (this.outgoingServer) {
      this.outgoingServer.close();
      this.outgoingServer = null;
    }

    this.state = 'Disconnected';
    console.error('[SocketIPC] Connections closed');
  }

  isConnected(): boolean {
    return this.state === 'Connected';
  }
}
