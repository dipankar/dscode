/**
 * Terminal API
 *
 * Integrated terminal support
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';
import { Uri } from './uri';
import { ThemeIcon } from './common';

// Terminal Options
export interface TerminalOptions {
  name?: string;
  shellPath?: string;
  shellArgs?: string[] | string;
  cwd?: string | { fsPath: string };
  env?: { [key: string]: string | null | undefined };
  strictEnv?: boolean;
  hideFromUser?: boolean;
  message?: string;
  iconPath?: Uri | { light: Uri; dark: Uri } | ThemeIcon;
  color?: ThemeIcon;
}

export interface ExtensionTerminalOptions {
  name: string;
  pty: Pseudoterminal;
  iconPath?: Uri | { light: Uri; dark: Uri } | ThemeIcon;
  color?: ThemeIcon;
}

export class Pseudoterminal {
  onDidWrite!: Event<string>;
  onDidOverrideDimensions?: Event<TerminalDimensions | undefined>;
  onDidClose?: Event<number | void>;
  onDidChangeName?: Event<string>;
  open(initialDimensions: TerminalDimensions | undefined): void {}
  close(): void {}
  handleInput?(data: string): void;
  setDimensions?(dimensions: TerminalDimensions): void;
}

export interface TerminalDimensions {
  readonly columns: number;
  readonly rows: number;
}

export interface TerminalExitStatus {
  readonly code: number | undefined;
  readonly reason: TerminalExitReason;
}

export enum TerminalExitReason {
  Unknown = 0,
  Shutdown = 1,
  Process = 2,
  User = 3,
  Extension = 4,
}

// Export Terminal as a class (not interface) so extensions can reference it at runtime
export class Terminal {
  readonly name!: string;
  get processId(): Promise<number | undefined> {
    return Promise.resolve(undefined);
  }
  readonly creationOptions!: Readonly<TerminalOptions | ExtensionTerminalOptions>;
  get exitStatus(): TerminalExitStatus | undefined {
    return undefined;
  }
  sendText(text: string, shouldExecute?: boolean): void {}
  show(preserveFocus?: boolean): void {}
  hide(): void {}
  dispose(): void {}
}

class TerminalImpl extends Terminal {
  private _exitStatus?: TerminalExitStatus;
  private _processId: Promise<number | undefined>;
  readonly name: string;
  readonly creationOptions: Readonly<TerminalOptions | ExtensionTerminalOptions>;

  constructor(
    private bridge: ExtensionHostBridge,
    private terminalId: string,
    name: string,
    creationOptions: Readonly<TerminalOptions | ExtensionTerminalOptions>
  ) {
    super();
    this.name = name;
    this.creationOptions = creationOptions;
    this._processId = this.getProcessId();
  }

  private async getProcessId(): Promise<number | undefined> {
    try {
      const result = (await this.bridge.request('getTerminalProcessId', {
        terminalId: this.terminalId,
      })) as { processId?: number };
      return result.processId;
    } catch {
      return undefined;
    }
  }

  get processId(): Promise<number | undefined> {
    return this._processId;
  }

  get exitStatus(): TerminalExitStatus | undefined {
    return this._exitStatus;
  }

  sendText(text: string, shouldExecute = true): void {
    this.bridge.send('terminalSendText', {
      terminalId: this.terminalId,
      text,
      shouldExecute,
    });
  }

  show(preserveFocus?: boolean): void {
    this.bridge.send('terminalShow', {
      terminalId: this.terminalId,
      preserveFocus,
    });
  }

  hide(): void {
    this.bridge.send('terminalHide', {
      terminalId: this.terminalId,
    });
  }

  dispose(): void {
    this.bridge.send('terminalDispose', {
      terminalId: this.terminalId,
    });
  }
}

export class TerminalAPI {
  private _onDidOpenTerminal = new EventEmitter<Terminal>();
  private _onDidCloseTerminal = new EventEmitter<Terminal>();
  private _onDidChangeActiveTerminal = new EventEmitter<Terminal | undefined>();
  private _onDidStartTerminalShellExecution = new EventEmitter<TerminalShellExecution>();
  private _onDidEndTerminalShellExecution = new EventEmitter<TerminalShellExecution>();
  private _terminals: Terminal[] = [];
  private _terminalIds = new Map<Terminal, string>();
  private _activeTerminal?: Terminal;

  readonly onDidOpenTerminal = this._onDidOpenTerminal.event;
  readonly onDidCloseTerminal = this._onDidCloseTerminal.event;
  readonly onDidChangeActiveTerminal = this._onDidChangeActiveTerminal.event;
  readonly onDidStartTerminalShellExecution = this._onDidStartTerminalShellExecution.event;
  readonly onDidEndTerminalShellExecution = this._onDidEndTerminalShellExecution.event;

  constructor(private bridge: ExtensionHostBridge) {
    this.setupListeners();
  }

  private setupListeners(): void {
    this.bridge.on(
      'terminalOpened',
      (data: { terminalId: string; name: string; options: TerminalOptions }) => {
        const terminal = new TerminalImpl(this.bridge, data.terminalId, data.name, data.options);
        this._terminalIds.set(terminal, data.terminalId);
        this._terminals.push(terminal);
        this._onDidOpenTerminal.fire(terminal);
      }
    );

    this.bridge.on('terminalClosed', (data: { terminalId: string }) => {
      const terminal = this._terminals.find((t) => this._terminalIds.get(t) === data.terminalId);
      if (terminal) {
        this._terminals = this._terminals.filter((t) => t !== terminal);
        this._onDidCloseTerminal.fire(terminal);
      }
    });
  }

  get terminals(): readonly Terminal[] {
    return this._terminals;
  }

  get activeTerminal(): Terminal | undefined {
    return this._activeTerminal;
  }

  createTerminal(options?: TerminalOptions): Terminal;
  createTerminal(name?: string, shellPath?: string, shellArgs?: string[] | string): Terminal;
  createTerminal(
    nameOrOptions?: string | TerminalOptions,
    shellPath?: string,
    shellArgs?: string[] | string
  ): Terminal {
    const terminalId = `terminal_${Date.now()}_${Math.random()}`;

    let options: TerminalOptions;
    if (typeof nameOrOptions === 'string') {
      options = { name: nameOrOptions, shellPath, shellArgs };
    } else {
      options = nameOrOptions || {};
    }

    const terminal = new TerminalImpl(this.bridge, terminalId, options.name || 'Terminal', options);

    this.bridge.send('createTerminal', {
      terminalId,
      options,
    });

    this._terminals.push(terminal);
    this._terminalIds.set(terminal, terminalId);
    this._onDidOpenTerminal.fire(terminal);

    return terminal;
  }

  createTerminalFromOptions(options: TerminalOptions | ExtensionTerminalOptions): Terminal {
    const terminalId = `terminal_${Date.now()}_${Math.random()}`;
    const name = 'name' in options ? options.name! : 'Terminal';

    const terminal = new TerminalImpl(this.bridge, terminalId, name, options);

    this.bridge.send('createTerminal', {
      terminalId,
      options,
    });

    this._terminals.push(terminal);
    this._terminalIds.set(terminal, terminalId);
    this._onDidOpenTerminal.fire(terminal);

    return terminal;
  }
}

export interface TerminalShellExecution {
  readonly terminal: Terminal;
  readonly commandLine: { value: string; isTrusted: boolean };
  readonly cwd: Uri | undefined;
  readonly executionId: string;
}
