import * as net from 'net';
import { EventEmitter } from 'events';

const MAX_MESSAGE_SIZE = 50 * 1024 * 1024;

export interface IPCMessage {
  id: string;
  type: string;
  payload: unknown;
}

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
    { resolve: (value: unknown) => void; reject: (reason: unknown) => void }
  > = new Map();
  private outgoingBuffer: Uint8Array = new Uint8Array(0);
  private incomingBuffer: Uint8Array = new Uint8Array(0);
  private listening = false;
  private connected = false;

  async connect(outgoingUrl: string, incomingUrl: string): Promise<void> {
    const outgoingPath = this.extractPath(outgoingUrl);
    const incomingPath = this.extractPath(incomingUrl);

    await this.startOutgoingServer(outgoingPath);
    await this.connectIncoming(incomingPath);

    this.connected = true;
    console.error(`[SocketIPC] Connected (outgoing=${outgoingPath}, incoming=${incomingPath})`);
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
        this.listening = true;
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
          this.listening = false;
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

      const tryConnect = () => {
        attempt++;
        const socket = net.connect(socketPath, () => {
          this.incomingSocket = socket;
          this.connected = true;
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
                const errMsg = (msg.payload as { error?: string })?.error || 'Unknown error';
                pending.reject(new Error(errMsg));
              }
            } else {
              const pending = this.pendingRequests.get(msg.id);
              if (pending) {
                this.pendingRequests.delete(msg.id);
                pending.resolve(msg.payload);
              }
            }
          }
        });

        socket.on('error', (err) => {
          if (attempt < maxAttempts) {
            setTimeout(tryConnect, attempt * 200);
          } else {
            reject(new Error(`Failed to connect after ${maxAttempts} attempts: ${err.message}`));
          }
        });

        socket.on('close', () => {
          this.incomingSocket = null;
          console.error('[SocketIPC] Incoming socket closed');
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
    if (!this.incomingSocket || !this.incomingSocket.writable) {
      throw new Error('Not connected. Call connect() first.');
    }

    const id = `req_${++this.messageId}`;
    const message: IPCMessage = { id, type: msgType, payload };

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });
      writeMessage(this.incomingSocket!, message);

      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error(`Request ${id} timed out`));
        }
      }, 30000);
    });
  }

  async send(msgType: string, payload: unknown): Promise<void> {
    await this.request(msgType, payload);
  }

  close(): void {
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

    for (const [, pending] of this.pendingRequests) {
      pending.reject(new Error('IPC closed'));
    }
    this.pendingRequests.clear();

    this.listening = false;
    this.connected = false;
    console.error('[SocketIPC] Connections closed');
  }

  isListening(): boolean {
    return this.listening;
  }

  isConnected(): boolean {
    return this.connected;
  }
}
