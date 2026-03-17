/**
 * VS Code Commands API
 *
 * Handles command registration and execution
 */

import { ExtensionHostBridge } from '../bridge';

type CommandHandler = (...args: any[]) => any;

export class CommandsAPI {
  private commands = new Map<string, CommandHandler>();

  constructor(private bridge: ExtensionHostBridge) {
    // Listen for command execution requests from main app
    this.bridge.on('execute-command', async (payload: any, respond: Function) => {
      const { command, args } = payload;

      const handler = this.commands.get(command);
      if (handler) {
        try {
          const result = await handler(...(args || []));
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

    if (this.commands.has(command)) {
      console.warn(`[Commands] Command already registered: ${command}`);
    }

    this.commands.set(command, callback);

    // Notify main app that command is available
    this.bridge.send('command-registered', { command });

    return {
      dispose: () => {
        this.commands.delete(command);
        this.bridge.send('command-unregistered', { command });
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
    const localHandler = this.commands.get(command);
    if (localHandler) {
      return await localHandler(...args);
    }

    // Otherwise, ask main app to execute it
    const result = await this.bridge.request('execute-command-request', {
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
}
