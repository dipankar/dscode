/**
 * VS Code Commands API
 *
 * Handles command registration and execution
 */

import { ExtensionHostBridge } from '../bridge';

type CommandHandler = (...args: any[]) => any;

export class CommandsAPI {
  private commands = new Map<string, { owner: string; handler: CommandHandler }>();
  private currentExtensionId: string | null = null;

  constructor(private bridge: ExtensionHostBridge) {
    // Listen for command execution requests from main app
    this.bridge.on('executeCommand', async (payload: any, respond: Function) => {
      const { command, args } = payload;

      const record = this.commands.get(command);
      if (record) {
        try {
          const result = await this.runWithExtension(record.owner, async () => record.handler(...(args || [])));
          respond({ success: true, result });
        } catch (error: any) {
          console.error(`[Commands] Error executing ${command}:`, error);
          respond({ success: false, error: error.message });
        }
      } else {
        console.warn(`[Commands] Command not found: ${command}`);
        respond({ success: false, error: `Command not found: ${command}` });
      }
    });
  }

  /**
   * Register a command handler
   */
  registerCommand(command: string, callback: CommandHandler): { dispose: () => void } {
    console.error(`[Commands] Registering: ${command}`);

    const owner = this.currentExtensionId ?? '__core__';

    if (this.commands.has(command)) {
      console.warn(`[Commands] Command already registered: ${command}`);
    }

    this.commands.set(command, { owner, handler: callback });

    // Notify main app that command is available
    this.bridge.send('command-registered', { command, owner });

    return {
      dispose: () => {
        if (this.commands.delete(command)) {
          this.bridge.send('command-unregistered', { command, owner });
        }
      },
    };
  }

  /**
   * Register a text editor command
   */
  registerTextEditorCommand(
    command: string,
    callback: (textEditor: any, edit: any, ...args: any[]) => void
  ): { dispose: () => void } {
    // For now, treat it the same as regular command
    // In the future, we'll pass actual TextEditor and TextEditorEdit objects
    return this.registerCommand(command, callback);
  }

  /**
   * Execute a command
   */
  async executeCommand<T = any>(command: string, ...args: any[]): Promise<T> {
    console.error(`[Commands] Executing: ${command}`);

    // Check if it's a local command first
    const record = this.commands.get(command);
    if (record) {
      return this.runWithExtension(record.owner, async () => record.handler(...args));
    }

    // Otherwise, ask main app to execute it
    const result = await this.bridge.request('executeCommandRequest', {
      command,
      args,
    });

    if (!result.success) {
      throw new Error(result.error || `Failed to execute command: ${command}`);
    }

    return result.result;
  }

  /**
   * Get all registered commands
   */
  getCommands(filterInternal?: boolean): Promise<string[]> {
    return Promise.resolve(Array.from(this.commands.keys()));
  }

  async runWithExtension<T>(extensionId: string, callback: () => T | Promise<T>): Promise<T> {
    const previous = this.currentExtensionId;
    this.currentExtensionId = extensionId;
    try {
      return await callback();
    } finally {
      this.currentExtensionId = previous;
    }
  }
}
