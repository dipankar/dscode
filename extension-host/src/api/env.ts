/**
 * Environment and Extensions API
 *
 * System environment information and extension management
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter } from './events';

// Environment
export interface EnvironmentAPI {
  readonly appName: string;
  readonly appRoot: string;
  readonly appHost: string;
  readonly language: string;
  readonly clipboard: Clipboard;
  readonly machineId: string;
  readonly sessionId: string;
  readonly remoteName: string | undefined;
  readonly shell: string;
  readonly uiKind: UIKind;
  readonly uriScheme: string;
  readonly isNewAppInstall: boolean;
  readonly isTelemetryEnabled: boolean;
  readonly onDidChangeTelemetryEnabled: Event<boolean>;
  readonly logLevel: LogLevel;
  readonly onDidChangeLogLevel: Event<LogLevel>;
  asExternalUri(target: any): Promise<any>;
  openExternal(target: any): Promise<boolean>;
}

export interface Clipboard {
  readText(): Promise<string>;
  writeText(value: string): Promise<void>;
}

export enum UIKind {
  Desktop = 1,
  Web = 2
}

export enum LogLevel {
  Off = 0,
  Trace = 1,
  Debug = 2,
  Info = 3,
  Warning = 4,
  Error = 5
}

class ClipboardImpl implements Clipboard {
  constructor(private bridge: ExtensionHostBridge) {}

  async readText(): Promise<string> {
    const result = await this.bridge.request('clipboardReadText', {});
    return result.text || '';
  }

  async writeText(value: string): Promise<void> {
    await this.bridge.request('clipboardWriteText', { text: value });
  }
}

export class EnvironmentAPIImpl implements EnvironmentAPI {
  private _onDidChangeTelemetryEnabled = new EventEmitter<boolean>();
  private _onDidChangeLogLevel = new EventEmitter<LogLevel>();
  private _clipboard: Clipboard;

  readonly onDidChangeTelemetryEnabled = this._onDidChangeTelemetryEnabled.event;
  readonly onDidChangeLogLevel = this._onDidChangeLogLevel.event;

  readonly appName = 'DSCode';
  readonly appRoot = '/';
  readonly appHost = 'desktop';
  readonly language = 'en';
  readonly machineId = 'machine-' + Date.now();
  readonly sessionId = 'session-' + Date.now();
  readonly remoteName = undefined;
  readonly shell = '/bin/bash';
  readonly uiKind = UIKind.Desktop;
  readonly uriScheme = 'vscode';
  readonly isNewAppInstall = false;
  readonly isTelemetryEnabled = false;
  readonly logLevel = LogLevel.Info;

  constructor(private bridge: ExtensionHostBridge) {
    this._clipboard = new ClipboardImpl(bridge);
  }

  get clipboard(): Clipboard {
    return this._clipboard;
  }

  async asExternalUri(target: any): Promise<any> {
    return target;
  }

  async openExternal(target: any): Promise<boolean> {
    this.bridge.send('openExternal', { uri: target });
    return true;
  }
}

// Extensions
export interface Extension<T> {
  readonly id: string;
  readonly extensionUri: any;
  readonly extensionPath: string;
  readonly isActive: boolean;
  readonly packageJSON: any;
  readonly extensionKind: ExtensionKind;
  readonly exports: T;
  activate(): Promise<T>;
}

export enum ExtensionKind {
  UI = 1,
  Workspace = 2
}

export interface ExtensionMode {
  development: 1;
  test: 2;
  production: 3;
}

class ExtensionImpl<T> implements Extension<T> {
  constructor(
    public readonly id: string,
    public readonly extensionUri: any,
    public readonly extensionPath: string,
    public readonly packageJSON: any,
    public readonly extensionKind: ExtensionKind,
    private _isActive: boolean,
    private _exports: T
  ) {}

  get isActive(): boolean {
    return this._isActive;
  }

  get exports(): T {
    return this._exports;
  }

  async activate(): Promise<T> {
    this._isActive = true;
    return this._exports;
  }
}

export class ExtensionsAPI {
  private _onDidChange = new EventEmitter<void>();
  private _extensions: Extension<any>[] = [];

  readonly onDidChange = this._onDidChange.event;

  constructor(private bridge: ExtensionHostBridge) {}

  get all(): readonly Extension<any>[] {
    return this._extensions;
  }

  getExtension<T = any>(extensionId: string): Extension<T> | undefined {
    return this._extensions.find(ext => ext.id === extensionId);
  }

  registerExtension<T>(
    id: string,
    path: string,
    packageJSON: any,
    exports: T
  ): Extension<T> {
    const ext = new ExtensionImpl(
      id,
      { path },
      path,
      packageJSON,
      ExtensionKind.Workspace,
      false,
      exports
    );
    this._extensions.push(ext);
    this._onDidChange.fire();
    return ext;
  }
}
