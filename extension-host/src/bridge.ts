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

/**
 * Input validation utilities
 */
class InputValidator {
  // Extension ID format: publisher.name (alphanumeric, dots, dashes)
  private static readonly EXTENSION_ID_PATTERN = /^[a-zA-Z0-9][a-zA-Z0-9._-]*[a-zA-Z0-9]$/;
  private static readonly MAX_EXTENSION_ID_LENGTH = 256;
  private static readonly MAX_STRING_LENGTH = 4096;

  /**
   * Validates an extension ID
   */
  static validateExtensionId(id: unknown): string {
    if (typeof id !== 'string') {
      throw new Error('Extension ID must be a string');
    }

    if (id.length === 0) {
      throw new Error('Extension ID cannot be empty');
    }

    if (id.length > this.MAX_EXTENSION_ID_LENGTH) {
      throw new Error(`Extension ID cannot exceed ${this.MAX_EXTENSION_ID_LENGTH} characters`);
    }

    // Allow simple IDs like single characters for testing
    if (id.length === 1 && /^[a-zA-Z0-9]$/.test(id)) {
      return id;
    }

    if (!this.EXTENSION_ID_PATTERN.test(id)) {
      throw new Error(
        'Extension ID contains invalid characters. Use only alphanumeric, dots, dashes, and underscores'
      );
    }

    // Check for path traversal attempts
    if (id.includes('..') || id.includes('/') || id.includes('\\')) {
      throw new Error('Extension ID cannot contain path components');
    }

    return id;
  }

  /**
   * Validates a command ID
   */
  static validateCommandId(id: unknown): string {
    if (typeof id !== 'string') {
      throw new Error('Command ID must be a string');
    }

    if (id.length === 0) {
      throw new Error('Command ID cannot be empty');
    }

    if (id.length > this.MAX_STRING_LENGTH) {
      throw new Error(`Command ID cannot exceed ${this.MAX_STRING_LENGTH} characters`);
    }

    // Command IDs can contain dots, colons (namespacing), alphanumeric, dashes, underscores
    if (!/^[a-zA-Z0-9][a-zA-Z0-9.:_-]*$/.test(id)) {
      throw new Error('Command ID contains invalid characters');
    }

    return id;
  }

  /**
   * Validates a view ID
   */
  static validateViewId(id: unknown): string {
    if (typeof id !== 'string') {
      throw new Error('View ID must be a string');
    }

    if (id.length === 0) {
      throw new Error('View ID cannot be empty');
    }

    if (id.length > this.MAX_STRING_LENGTH) {
      throw new Error(`View ID cannot exceed ${this.MAX_STRING_LENGTH} characters`);
    }

    // View IDs can contain dots, colons, alphanumeric, dashes, underscores
    if (!/^[a-zA-Z0-9][a-zA-Z0-9.:_-]*$/.test(id)) {
      throw new Error('View ID contains invalid characters');
    }

    return id;
  }

  /**
   * Validates a generic string with max length
   */
  static validateString(
    value: unknown,
    fieldName: string,
    maxLength: number = this.MAX_STRING_LENGTH
  ): string {
    if (typeof value !== 'string') {
      throw new Error(`${fieldName} must be a string`);
    }

    if (value.length > maxLength) {
      throw new Error(`${fieldName} cannot exceed ${maxLength} characters`);
    }

    return value;
  }

  /**
   * Validates payload is an object
   */
  static validatePayload(payload: unknown): Record<string, unknown> {
    if (payload === null || payload === undefined) {
      return {};
    }

    if (typeof payload !== 'object' || Array.isArray(payload)) {
      throw new Error('Payload must be an object');
    }

    return payload as Record<string, unknown>;
  }
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

      // Connect to Tauri for sending requests (REQ socket) with retries
      const maxAttempts = 10;
      for (let attempt = 1; attempt <= maxAttempts; attempt++) {
        try {
          this.nng.connect(this.incomingIpcUrl);
          console.error('[Bridge] Connected to incoming socket');
          break;
        } catch (error) {
          if (attempt === maxAttempts) {
            throw new Error(`Failed to connect after ${maxAttempts} attempts: ${error}`);
          }
          const delay = attempt * 200;
          console.error(`[Bridge] Connection attempt ${attempt} failed, retrying in ${delay}ms...`);
          await new Promise((resolve) => setTimeout(resolve, delay));
        }
      }

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
    this.nng.on('activate-extension', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const extensionId = InputValidator.validateExtensionId(validatedPayload.extensionId);

      await this.emitAsync('activate-extension', extensionId);
      return { success: true };
    });

    // Extension lifecycle: deactivate
    this.nng.on('deactivate-extension', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const extensionId = InputValidator.validateExtensionId(validatedPayload.extensionId);

      await this.emitAsync('deactivate-extension', extensionId);
      return { success: true };
    });

    // Extension management: list installed extensions
    this.nng.on('list-extensions', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      return new Promise((resolve, reject) => {
        const respond = (response: unknown) => resolve(response);
        try {
          const handled = this.emit('list-extensions', validatedPayload, respond);
          if (!handled) {
            reject(new Error('No listeners registered for list-extensions'));
          }
        } catch (error) {
          reject(error);
        }
      });
    });

    // Extension management: reload extensions after install/uninstall
    this.nng.on('reload-extensions', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      return new Promise((resolve, reject) => {
        const respond = (response: unknown) => resolve(response);
        try {
          const handled = this.emit('reload-extensions', validatedPayload, respond);
          if (!handled) {
            reject(new Error('No listeners registered for reload-extensions'));
          }
        } catch (error) {
          reject(error);
        }
      });
    });

    // Extension management: uninstall extension
    this.nng.on('uninstall-extension', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);

      // Validate extension ID if present
      if (validatedPayload.extensionId !== undefined) {
        InputValidator.validateExtensionId(validatedPayload.extensionId);
      }

      return new Promise((resolve, reject) => {
        const respond = (response: unknown) => resolve(response);
        try {
          const handled = this.emit('uninstall-extension', validatedPayload, respond);
          if (!handled) {
            reject(new Error('No listeners registered for uninstall-extension'));
          }
        } catch (error) {
          reject(error);
        }
      });
    });

    // Handle tree view requests
    this.nng.on('treeView:getChildren', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);

      // Validate view ID if present
      if (validatedPayload.viewId !== undefined) {
        InputValidator.validateViewId(validatedPayload.viewId);
      }

      console.error('[Bridge] Received treeView:getChildren request:', validatedPayload);

      return new Promise((resolve) => {
        // Emit event for extension manager to handle
        this.emit('treeView:getChildren', validatedPayload, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('treeView:event', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);

      // Validate view ID if present
      if (validatedPayload.viewId !== undefined) {
        InputValidator.validateViewId(validatedPayload.viewId);
      }

      console.error('[Bridge] Received treeView:event:', validatedPayload);

      return new Promise((resolve) => {
        this.emit('treeView:event', validatedPayload, (response: unknown) => {
          resolve(response ?? { success: true });
        });
      });
    });

    // Handle command execution requests
    this.nng.on('executeCommand', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);

      // Validate command ID
      if (validatedPayload.command !== undefined) {
        InputValidator.validateCommandId(validatedPayload.command);
      }

      console.error('[Bridge] Received executeCommand request:', validatedPayload);

      return new Promise((resolve) => {
        // Emit event for extension manager to handle
        this.emit('executeCommand', validatedPayload, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('configuration-changed', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('configurationChanged', validatedPayload);
        resolve({ success: true });
      });
    });

    this.nng.on('fsWatcher:event', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('fsWatcher:event', validatedPayload);
        resolve({ success: true });
      });
    });

    // ==================== Activation Event Handlers ====================

    // Signal startup finished - triggers onStartupFinished extensions
    this.nng.on('signal-startup-finished', async () => {
      await this.emitAsync('signal-startup-finished');
      return { success: true };
    });

    // Trigger onLanguage activation event
    this.nng.on('trigger-on-language', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const languageId = InputValidator.validateString(
        validatedPayload.languageId,
        'languageId',
        128
      );
      await this.emitAsync('trigger-on-language', languageId);
      return { success: true };
    });

    // Trigger onCommand activation event
    this.nng.on('trigger-on-command', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const commandId = InputValidator.validateCommandId(validatedPayload.commandId);
      await this.emitAsync('trigger-on-command', commandId);
      return { success: true };
    });

    // Trigger onView activation event
    this.nng.on('trigger-on-view', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const viewId = InputValidator.validateViewId(validatedPayload.viewId);
      await this.emitAsync('trigger-on-view', viewId);
      return { success: true };
    });

    // Trigger onDebug activation event
    this.nng.on('trigger-on-debug', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const debugType = validatedPayload.debugType
        ? InputValidator.validateString(validatedPayload.debugType, 'debugType', 128)
        : undefined;
      await this.emitAsync('trigger-on-debug', debugType);
      return { success: true };
    });

    // Trigger onUri activation event
    this.nng.on('trigger-on-uri', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const scheme = validatedPayload.scheme
        ? InputValidator.validateString(validatedPayload.scheme, 'scheme', 64)
        : undefined;
      await this.emitAsync('trigger-on-uri', scheme);
      return { success: true };
    });

    // Trigger onFileSystem activation event
    this.nng.on('trigger-on-filesystem', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const scheme = InputValidator.validateString(validatedPayload.scheme, 'scheme', 64);
      await this.emitAsync('trigger-on-filesystem', scheme);
      return { success: true };
    });

    // Trigger onWebviewPanel activation event
    this.nng.on('trigger-on-webview-panel', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const viewType = InputValidator.validateString(validatedPayload.viewType, 'viewType', 256);
      await this.emitAsync('trigger-on-webview-panel', viewType);
      return { success: true };
    });

    // Trigger onCustomEditor activation event
    this.nng.on('trigger-on-custom-editor', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const viewType = InputValidator.validateString(validatedPayload.viewType, 'viewType', 256);
      await this.emitAsync('trigger-on-custom-editor', viewType);
      return { success: true };
    });

    // Trigger onNotebook activation event
    this.nng.on('trigger-on-notebook', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const notebookType = InputValidator.validateString(
        validatedPayload.notebookType,
        'notebookType',
        256
      );
      await this.emitAsync('trigger-on-notebook', notebookType);
      return { success: true };
    });

    // Trigger onAuthenticationRequest activation event
    this.nng.on('trigger-on-authentication', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const providerId = InputValidator.validateString(
        validatedPayload.providerId,
        'providerId',
        256
      );
      await this.emitAsync('trigger-on-authentication', providerId);
      return { success: true };
    });

    // Trigger onTerminalProfile activation event
    this.nng.on('trigger-on-terminal-profile', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const profileId = InputValidator.validateString(validatedPayload.profileId, 'profileId', 256);
      await this.emitAsync('trigger-on-terminal-profile', profileId);
      return { success: true };
    });

    // Trigger workspaceContains activation event
    this.nng.on('trigger-workspace-contains', async (payload: unknown) => {
      const validatedPayload = InputValidator.validatePayload(payload);
      const pattern = InputValidator.validateString(validatedPayload.pattern, 'pattern', 1024);
      await this.emitAsync('trigger-workspace-contains', pattern);
      return { success: true };
    });

    // Get pending activations (for debugging)
    this.nng.on('get-pending-activations', async () => {
      return new Promise((resolve) => {
        this.emit('get-pending-activations', {}, (response: unknown) => {
          resolve(response);
        });
      });
    });

    // ==================== Language Provider Handlers ====================

    // Hover provider request
    this.nng.on('provideHover', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideHover', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    // Definition provider request
    this.nng.on('provideDefinition', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideDefinition', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideImplementation', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideImplementation', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideTypeDefinition', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideTypeDefinition', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideDeclaration', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideDeclaration', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    // References provider request
    this.nng.on('provideReferences', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideReferences', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    // Code actions provider request
    this.nng.on('provideCodeActions', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideCodeActions', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    // Document symbols provider request
    this.nng.on('provideDocumentSymbols', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideDocumentSymbols', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    // Document formatting provider request
    this.nng.on('provideDocumentFormatting', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideDocumentFormatting', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    // Completion provider request
    this.nng.on('provideCompletion', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideCompletion', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideSignatureHelp', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideSignatureHelp', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideRename', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideRename', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('prepareRename', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('prepareRename', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideCodeLenses', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideCodeLenses', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideDocumentLinks', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideDocumentLinks', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideDocumentHighlights', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideDocumentHighlights', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideFoldingRanges', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideFoldingRanges', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideSelectionRanges', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideSelectionRanges', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideWorkspaceSymbols', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideWorkspaceSymbols', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideRangeFormatting', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideRangeFormatting', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideOnTypeFormatting', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideOnTypeFormatting', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideInlineCompletionItems', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideInlineCompletionItems', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideSemanticTokens', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideSemanticTokens', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideDocumentColors', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideDocumentColors', data, (response: unknown) => {
          resolve(response);
        });
      });
    });

    this.nng.on('provideColorPresentations', async (payload: unknown) => {
      const data = InputValidator.validatePayload(payload);
      return new Promise((resolve) => {
        this.emit('provideColorPresentations', data, (response: unknown) => {
          resolve(response);
        });
      });
    });
  }

  /**
   * Send a one-way message to Tauri (fire and forget)
   * For one-way messages, we still use request() but ignore the response
   */
  async send(type: string, payload: unknown): Promise<void> {
    if (!this.isConnected) {
      throw new Error('Bridge not connected');
    }

    // Validate type
    InputValidator.validateString(type, 'Message type', 256);

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
  private async emitAsync(event: string, ...args: unknown[]): Promise<void> {
    const listeners = this.listeners(event);
    if (listeners.length === 0) {
      throw new Error(`No listeners registered for ${event}`);
    }
    for (const listener of listeners) {
      const result = (listener as (...innerArgs: unknown[]) => unknown)(...args);
      if (result instanceof Promise) {
        await result;
      }
    }
  }

  /**
   * Send a request to Tauri and wait for response
   */
  async request(type: string, payload: unknown): Promise<unknown> {
    if (!this.isConnected) {
      throw new Error('Bridge not connected');
    }

    // Validate type
    InputValidator.validateString(type, 'Request type', 256);

    try {
      const response = await this.nng.request(type, payload);
      return response;
    } catch (error) {
      console.error('[Bridge] Request failed:', error);
      throw error;
    }
  }
}
