/**
 * VS Code Window API
 *
 * Handles UI interactions like messages, input boxes, quick picks, etc.
 */

import { ExtensionHostBridge } from '../bridge';

export enum MessageType {
  Info = 'info',
  Warning = 'warning',
  Error = 'error',
}

export class WindowAPI {
  constructor(private bridge: ExtensionHostBridge) {}

  /**
   * Show an information message
   */
  async showInformationMessage(message: string, ...items: string[]): Promise<string | undefined> {
    console.log('[Window] Info:', message);

    if (items.length === 0) {
      // Just show the message
      await this.bridge.send('window-show-message', {
        type: MessageType.Info,
        message,
      });
      return undefined;
    }

    // Show message with actions
    const result = await this.bridge.request('window-show-message-with-actions', {
      type: MessageType.Info,
      message,
      actions: items,
    });

    return result?.action;
  }

  /**
   * Show a warning message
   */
  async showWarningMessage(message: string, ...items: string[]): Promise<string | undefined> {
    console.log('[Window] Warning:', message);

    if (items.length === 0) {
      await this.bridge.send('window-show-message', {
        type: MessageType.Warning,
        message,
      });
      return undefined;
    }

    const result = await this.bridge.request('window-show-message-with-actions', {
      type: MessageType.Warning,
      message,
      actions: items,
    });

    return result?.action;
  }

  /**
   * Show an error message
   */
  async showErrorMessage(message: string, ...items: string[]): Promise<string | undefined> {
    console.error('[Window] Error:', message);

    if (items.length === 0) {
      await this.bridge.send('window-show-message', {
        type: MessageType.Error,
        message,
      });
      return undefined;
    }

    const result = await this.bridge.request('window-show-message-with-actions', {
      type: MessageType.Error,
      message,
      actions: items,
    });

    return result?.action;
  }

  /**
   * Show an input box
   */
  async showInputBox(options?: {
    prompt?: string;
    placeHolder?: string;
    value?: string;
    password?: boolean;
  }): Promise<string | undefined> {
    const result = await this.bridge.request('window-show-input-box', options || {});
    return result?.value;
  }

  /**
   * Show a quick pick menu
   */
  async showQuickPick(
    items: string[] | { label: string; description?: string; detail?: string }[],
    options?: {
      placeHolder?: string;
      canPickMany?: boolean;
    }
  ): Promise<any> {
    const result = await this.bridge.request('window-show-quick-pick', {
      items,
      options: options || {},
    });

    return result?.selected;
  }

  /**
   * Create an output channel
   */
  createOutputChannel(name: string): OutputChannel {
    return new OutputChannel(name, this.bridge);
  }

  /**
   * Set status bar message
   */
  setStatusBarMessage(text: string, hideAfterTimeout?: number): { dispose: () => void } {
    const id = Math.random().toString(36).substring(7);

    this.bridge.send('window-set-status-bar-message', {
      id,
      text,
      timeout: hideAfterTimeout,
    });

    return {
      dispose: () => {
        this.bridge.send('window-clear-status-bar-message', { id });
      },
    };
  }
}

/**
 * Output Channel
 */
export class OutputChannel {
  constructor(private name: string, private bridge: ExtensionHostBridge) {}

  append(value: string): void {
    this.bridge.send('output-channel-append', {
      channel: this.name,
      value,
    });
  }

  appendLine(value: string): void {
    this.append(value + '\n');
  }

  clear(): void {
    this.bridge.send('output-channel-clear', {
      channel: this.name,
    });
  }

  show(preserveFocus?: boolean): void {
    this.bridge.send('output-channel-show', {
      channel: this.name,
      preserveFocus,
    });
  }

  hide(): void {
    this.bridge.send('output-channel-hide', {
      channel: this.name,
    });
  }

  dispose(): void {
    this.bridge.send('output-channel-dispose', {
      channel: this.name,
    });
  }
}
