/**
 * Environment and Extensions API
 *
 * System environment information and extension management
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter } from './events';
import { Uri } from './uri';

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
  Web = 2,
}

export enum LogLevel {
  Off = 0,
  Trace = 1,
  Debug = 2,
  Info = 3,
  Warning = 4,
  Error = 5,
}

class ClipboardImpl implements Clipboard {
  constructor(private bridge: ExtensionHostBridge) {}

  async readText(): Promise<string> {
    const result = (await this.bridge.request('clipboardReadText', {})) as { text?: string };
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

  asExternalUri(target: Uri): Promise<Uri> {
    return Promise.resolve(target);
  }

  async openExternal(target: Uri): Promise<boolean> {
    this.bridge.send('openExternal', { uri: target });
    return true;
  }

  createTelemetryLogger(
    sender: TelemetrySender,
    options?: TelemetryLoggerOptions
  ): TelemetryLogger {
    return new TelemetryLoggerImpl(sender, options);
  }
}

// Telemetry API types
export interface TelemetrySender {
  sendEventData(eventName: string, data?: Record<string, unknown>): void;
  sendErrorData(error: Error, data?: Record<string, unknown>): void;
  flush?(): void | Promise<void>;
}

export interface TelemetryLoggerOptions {
  ignoreBuiltInCommonProperties?: boolean;
  ignoreUnhandledErrors?: boolean;
  additionalCommonProperties?: Record<string, unknown>;
}

export interface TelemetryLogger {
  readonly onDidChangeEnableStates: Event<TelemetryLogger>;
  readonly isUsageEnabled: boolean;
  readonly isErrorsEnabled: boolean;
  logUsage(eventName: string, data?: Record<string, unknown>): void;
  logError(eventNameOrError: string | Error, data?: Record<string, unknown>): void;
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

  logUsage(eventName: string, data?: Record<string, unknown>): void {
    if (this.isUsageEnabled) {
      this.sender.sendEventData(eventName, data);
    }
  }

  logError(eventNameOrError: string | Error, data?: Record<string, unknown>): void {
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
  readonly extensionUri!: Uri;
  readonly extensionPath!: string;
  get isActive(): boolean {
    return false;
  }
  readonly packageJSON!: Record<string, unknown>;
  readonly extensionKind!: ExtensionKind;
  get exports(): T {
    return undefined as unknown as T;
  }
  activate(): Promise<T> {
    return Promise.resolve(undefined as unknown as T);
  }
}

export enum ExtensionKind {
  UI = 1,
  Workspace = 2,
}

export interface ExtensionMode {
  development: 1;
  test: 2;
  production: 3;
}

class ExtensionImpl<T> extends Extension<T> {
  private _isActive: boolean;
  private _exports: T;
  readonly id: string;
  readonly extensionUri: Uri;
  readonly extensionPath: string;
  readonly packageJSON: Record<string, unknown>;
  readonly extensionKind: ExtensionKind;

  constructor(
    id: string,
    extensionUri: Uri,
    extensionPath: string,
    packageJSON: Record<string, unknown>,
    extensionKind: ExtensionKind,
    isActive: boolean,
    exports: T
  ) {
    super();
    this.id = id;
    this.extensionUri = extensionUri;
    this.extensionPath = extensionPath;
    this.packageJSON = packageJSON;
    this.extensionKind = extensionKind;
    this._isActive = isActive;
    this._exports = exports;
  }

  get isActive(): boolean {
    return this._isActive;
  }

  get exports(): T {
    if (this._exports === undefined || this._exports === null) {
      const createStub = (): any =>
        new Proxy(function () {}, {
          get(_target: any, prop: string | symbol) {
            if (prop === Symbol.unscopables) return undefined;
            if (prop === 'then') return undefined;
            if (prop === 'toJSON') return () => ({});
            if (prop === 'constructor') return undefined;
            return createStub();
          },
          apply() {
            return undefined;
          },
          construct() {
            return createStub();
          },
        });
      return createStub() as unknown as T;
    }
    return this._exports;
  }

  async activate(): Promise<T> {
    this._isActive = true;
    return this.exports;
  }
}

export class ExtensionsAPI {
  private _onDidChange = new EventEmitter<void>();
  private _extensions: Extension<unknown>[] = [];

  readonly onDidChange = this._onDidChange.event;

  constructor(private bridge: ExtensionHostBridge) {}

  get all(): readonly Extension<unknown>[] {
    return this._extensions;
  }

  getExtension<T>(extensionId: string): Extension<T> | undefined {
    return this._extensions.find((ext) => ext.id === extensionId) as Extension<T> | undefined;
  }

  registerExtension<T>(
    id: string,
    path: string,
    packageJSON: Record<string, unknown>,
    exports: T
  ): Extension<T> {
    const ext = new ExtensionImpl(
      id,
      Uri.file(path),
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
