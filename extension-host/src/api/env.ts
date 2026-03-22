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
  createTelemetryLogger(sender: TelemetrySender, options?: TelemetryLoggerOptions): TelemetryLogger;
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
    const result = await this.bridge.request('clipboardReadText', {}) as { text?: string };
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

  createTelemetryLogger(sender: TelemetrySender, options?: TelemetryLoggerOptions): TelemetryLogger {
    return new TelemetryLoggerImpl(sender, options);
  }
}

// Telemetry API types
export interface TelemetrySender {
  sendEventData(eventName: string, data?: Record<string, any>): void;
  sendErrorData(error: Error, data?: Record<string, any>): void;
  flush?(): void | Promise<void>;
}

export interface TelemetryLoggerOptions {
  ignoreBuiltInCommonProperties?: boolean;
  ignoreUnhandledErrors?: boolean;
  additionalCommonProperties?: Record<string, any>;
}

export interface TelemetryLogger {
  readonly onDidChangeEnableStates: Event<TelemetryLogger>;
  readonly isUsageEnabled: boolean;
  readonly isErrorsEnabled: boolean;
  logUsage(eventName: string, data?: Record<string, any>): void;
  logError(eventNameOrError: string | Error, data?: Record<string, any>): void;
  dispose(): void;
}

class TelemetryLoggerImpl implements TelemetryLogger {
  private _onDidChangeEnableStates = new EventEmitter<TelemetryLogger>();
  readonly onDidChangeEnableStates = this._onDidChangeEnableStates.event;
  readonly isUsageEnabled = false; // Telemetry disabled by default
  readonly isErrorsEnabled = false;

  constructor(
    private sender: TelemetrySender,
    private options?: TelemetryLoggerOptions
  ) {}

  logUsage(eventName: string, data?: Record<string, any>): void {
    if (this.isUsageEnabled) {
      this.sender.sendEventData(eventName, data);
    }
  }

  logError(eventNameOrError: string | Error, data?: Record<string, any>): void {
    if (this.isErrorsEnabled) {
      if (eventNameOrError instanceof Error) {
        this.sender.sendErrorData(eventNameOrError, data);
      } else {
        this.sender.sendEventData(eventNameOrError, data);
      }
    }
  }

  dispose(): void {
    this.sender.flush?.();
    this._onDidChangeEnableStates.dispose();
  }
}

// Extensions
export class Extension<T> {
  readonly id!: string;
  readonly extensionUri!: any;
  readonly extensionPath!: string;
  get isActive(): boolean { return false; }
  readonly packageJSON!: any;
  readonly extensionKind!: ExtensionKind;
  get exports(): T { return undefined as any; }
  activate(): Promise<T> { return Promise.resolve(undefined as any); }
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

class ExtensionImpl<T> extends Extension<T> {
  private _isActive: boolean;
  private _exports: T;

  constructor(
    id: string,
    extensionUri: any,
    extensionPath: string,
    packageJSON: any,
    extensionKind: ExtensionKind,
    isActive: boolean,
    exports: T
  ) {
    super();
    (this as any).id = id;
    (this as any).extensionUri = extensionUri;
    (this as any).extensionPath = extensionPath;
    (this as any).packageJSON = packageJSON;
    (this as any).extensionKind = extensionKind;
    this._isActive = isActive;
    this._exports = exports;
  }

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
