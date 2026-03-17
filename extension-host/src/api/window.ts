/**
 * VS Code Window API
 *
 * Handles UI interactions like messages, input boxes, quick picks, etc.
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter } from './events';

export enum MessageType {
  Info = 'info',
  Warning = 'warning',
  Error = 'error',
}

export class WindowAPI {
  // Event emitters
  private _onDidChangeActiveTextEditor = new EventEmitter<any>();
  private _onDidChangeVisibleTextEditors = new EventEmitter<any[]>();
  private _onDidChangeTextEditorSelection = new EventEmitter<any>();
  private _onDidChangeTextEditorVisibleRanges = new EventEmitter<any>();
  private _onDidChangeTextEditorOptions = new EventEmitter<any>();
  private _onDidChangeTextEditorViewColumn = new EventEmitter<any>();
  private _onDidChangeWindowState = new EventEmitter<any>();

  // Events
  readonly onDidChangeActiveTextEditor = this._onDidChangeActiveTextEditor.event;
  readonly onDidChangeVisibleTextEditors = this._onDidChangeVisibleTextEditors.event;
  readonly onDidChangeTextEditorSelection = this._onDidChangeTextEditorSelection.event;
  readonly onDidChangeTextEditorVisibleRanges = this._onDidChangeTextEditorVisibleRanges.event;
  readonly onDidChangeTextEditorOptions = this._onDidChangeTextEditorOptions.event;
  readonly onDidChangeTextEditorViewColumn = this._onDidChangeTextEditorViewColumn.event;
  readonly onDidChangeWindowState = this._onDidChangeWindowState.event;

  constructor(private bridge: ExtensionHostBridge) {
    this.setupEventListeners();
  }

  private setupEventListeners(): void {
    this.bridge.on('activeTextEditorChanged', (data: any) => {
      this._onDidChangeActiveTextEditor.fire(data);
    });

    this.bridge.on('visibleTextEditorsChanged', (data: any) => {
      this._onDidChangeVisibleTextEditors.fire(data);
    });

    this.bridge.on('textEditorSelectionChanged', (data: any) => {
      this._onDidChangeTextEditorSelection.fire(data);
    });

    this.bridge.on('textEditorVisibleRangesChanged', (data: any) => {
      this._onDidChangeTextEditorVisibleRanges.fire(data);
    });

    this.bridge.on('textEditorOptionsChanged', (data: any) => {
      this._onDidChangeTextEditorOptions.fire(data);
    });

    this.bridge.on('textEditorViewColumnChanged', (data: any) => {
      this._onDidChangeTextEditorViewColumn.fire(data);
    });

    this.bridge.on('windowStateChanged', (data: any) => {
      this._onDidChangeWindowState.fire(data);
    });
  }

  /**
   * Show an information message
   */
  async showInformationMessage(message: string, ...items: string[]): Promise<string | undefined> {
    console.error('[Window] Info:', message);

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
    console.error('[Window] Warning:', message);

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
