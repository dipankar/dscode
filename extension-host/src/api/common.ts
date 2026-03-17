/**
 * Common Types and Utility Classes
 *
 * MarkdownString, ThemeColor, ThemeIcon, and other shared utilities
 */

export class MarkdownString {
  value: string;
  isTrusted?: boolean | { enabledCommands: string[] };
  supportThemeIcons?: boolean;
  supportHtml?: boolean;
  baseUri?: any;

  constructor(value?: string, supportThemeIcons?: boolean) {
    this.value = value || '';
    this.supportThemeIcons = supportThemeIcons;
  }

  appendText(value: string): MarkdownString {
    // Escape markdown special characters
    this.value += value
      .replace(/\\/g, '\\\\')
      .replace(/\*/g, '\\*')
      .replace(/_/g, '\\_')
      .replace(/\[/g, '\\[')
      .replace(/\]/g, '\\]')
      .replace(/`/g, '\\`');
    return this;
  }

  appendMarkdown(value: string): MarkdownString {
    this.value += value;
    return this;
  }

  appendCodeblock(value: string, language?: string): MarkdownString {
    this.value += '\n```' + (language || '') + '\n';
    this.value += value;
    this.value += '\n```\n';
    return this;
  }

  appendLink(target: string | any, label: string): MarkdownString {
    const href = typeof target === 'string' ? target : target.toString();
    this.value += `[${label}](${href})`;
    return this;
  }
}

export class ThemeColor {
  constructor(public readonly id: string) {}
}

export class ThemeIcon {
  static readonly File = new ThemeIcon('file');
  static readonly Folder = new ThemeIcon('folder');

  constructor(
    public readonly id: string,
    public readonly color?: ThemeColor
  ) {}
}

export enum ConfigurationTarget {
  Global = 1,
  Workspace = 2,
  WorkspaceFolder = 3
}

export enum ExtensionMode {
  Production = 1,
  Development = 2,
  Test = 3
}

export interface CancellationToken {
  isCancellationRequested: boolean;
  onCancellationRequested: any; // Event<any>
}

export class CancellationTokenSource {
  token: CancellationToken;
  private _isCancelled = false;

  constructor() {
    this.token = {
      isCancellationRequested: false,
      onCancellationRequested: () => ({ dispose: () => {} })
    };
  }

  cancel(): void {
    this._isCancelled = true;
    (this.token as any).isCancellationRequested = true;
  }

  dispose(): void {
    this.cancel();
  }
}

export interface QuickPickItem {
  label: string;
  description?: string;
  detail?: string;
  picked?: boolean;
  alwaysShow?: boolean;
  buttons?: QuickInputButton[];
}

export interface QuickInputButton {
  iconPath: any;
  tooltip?: string;
}

export enum QuickPickItemKind {
  Separator = -1,
  Default = 0
}

export interface InputBoxOptions {
  value?: string;
  valueSelection?: [number, number];
  prompt?: string;
  placeHolder?: string;
  password?: boolean;
  ignoreFocusOut?: boolean;
  validateInput?(value: string): string | undefined | null | Promise<string | undefined | null>;
}

export interface QuickPickOptions {
  title?: string;
  placeHolder?: string;
  matchOnDescription?: boolean;
  matchOnDetail?: boolean;
  canPickMany?: boolean;
  ignoreFocusOut?: boolean;
  onDidSelectItem?(item: QuickPickItem | string): void;
}

export interface ProgressOptions {
  location: ProgressLocation | { viewId: string };
  title?: string;
  cancellable?: boolean;
}

export enum ProgressLocation {
  SourceControl = 1,
  Window = 10,
  Notification = 15
}

export interface Progress<T> {
  report(value: T): void;
}

export interface Memento {
  get<T>(key: string): T | undefined;
  get<T>(key: string, defaultValue: T): T;
  update(key: string, value: any): Promise<void>;
  keys(): readonly string[];
}

export class MementoImpl implements Memento {
  private storage = new Map<string, any>();

  get<T>(key: string, defaultValue?: T): T | undefined {
    return this.storage.has(key) ? this.storage.get(key) : defaultValue;
  }

  async update(key: string, value: any): Promise<void> {
    if (value === undefined) {
      this.storage.delete(key);
    } else {
      this.storage.set(key, value);
    }
  }

  keys(): readonly string[] {
    return Array.from(this.storage.keys());
  }
}

export enum FileType {
  Unknown = 0,
  File = 1,
  Directory = 2,
  SymbolicLink = 64
}

export interface FileStat {
  type: FileType;
  ctime: number;
  mtime: number;
  size: number;
}

export enum FilePermission {
  Readonly = 1
}

export class FileSystemError extends Error {
  static FileExists(messageOrUri?: string | any): FileSystemError {
    return new FileSystemError('EntryExists', messageOrUri);
  }

  static FileNotFound(messageOrUri?: string | any): FileSystemError {
    return new FileSystemError('EntryNotFound', messageOrUri);
  }

  static FileNotADirectory(messageOrUri?: string | any): FileSystemError {
    return new FileSystemError('EntryNotADirectory', messageOrUri);
  }

  static FileIsADirectory(messageOrUri?: string | any): FileSystemError {
    return new FileSystemError('EntryIsADirectory', messageOrUri);
  }

  static NoPermissions(messageOrUri?: string | any): FileSystemError {
    return new FileSystemError('NoPermissions', messageOrUri);
  }

  static Unavailable(messageOrUri?: string | any): FileSystemError {
    return new FileSystemError('Unavailable', messageOrUri);
  }

  constructor(
    public code: string,
    messageOrUri?: string | any
  ) {
    super(typeof messageOrUri === 'string' ? messageOrUri : code);
    this.name = code;
  }
}
