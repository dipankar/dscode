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

export class QuickPickItem {
  constructor(public label: string) {}

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

// EnvironmentVariableCollection
export enum EnvironmentVariableMutatorType {
  Replace = 1,
  Append = 2,
  Prepend = 3
}

export class EnvironmentVariableCollection {
  readonly persistent!: boolean;
  readonly description!: string | { value: string } | undefined;

  replace(variable: string, value: string): void {}
  append(variable: string, value: string): void {}
  prepend(variable: string, value: string): void {}
  get(variable: string): any { return undefined; }
  forEach(callback: (variable: string, mutator: any, collection: EnvironmentVariableCollection) => any, thisArg?: any): void {}
  delete(variable: string): void {}
  clear(): void {}
}

// CancellationError - thrown when an operation is cancelled
export class CancellationError extends Error {
  constructor() {
    super('Cancelled');
    this.name = 'CancellationError';
  }
}

// CodeLens class
export class CodeLens {
  range: any; // Range
  command?: {
    title: string;
    command: string;
    tooltip?: string;
    arguments?: any[];
  };
  isResolved: boolean = false;

  constructor(range: any, command?: { title: string; command: string; tooltip?: string; arguments?: any[] }) {
    this.range = range;
    this.command = command;
    this.isResolved = !!command;
  }
}

// l10n API for localization
export const l10n = {
  t(message: string, ...args: any[]): string {
    // Simple implementation - just return the message with basic substitution
    if (args.length === 0) return message;
    if (args.length === 1 && typeof args[0] === 'object') {
      // Named arguments
      return message.replace(/\{(\w+)\}/g, (_, key) => String(args[0][key] ?? `{${key}}`));
    }
    // Positional arguments
    return message.replace(/\{(\d+)\}/g, (_, index) => String(args[parseInt(index)] ?? `{${index}}`));
  },
  bundle: undefined as any,
  uri: undefined as any
};

// DocumentLink class
export class DocumentLink {
  range: any; // Range
  target?: any; // Uri

  constructor(range: any, target?: any) {
    this.range = range;
    this.target = target;
  }
}

// SignatureHelp class
export class SignatureHelp {
  signatures: SignatureInformation[] = [];
  activeSignature: number = 0;
  activeParameter: number = 0;
}

// SignatureInformation class
export class SignatureInformation {
  label: string;
  documentation?: string | { value: string };
  parameters: ParameterInformation[] = [];
  activeParameter?: number;

  constructor(label: string, documentation?: string | { value: string }) {
    this.label = label;
    this.documentation = documentation;
  }
}

// ParameterInformation class
export class ParameterInformation {
  label: string | [number, number];
  documentation?: string | { value: string };

  constructor(label: string | [number, number], documentation?: string | { value: string }) {
    this.label = label;
    this.documentation = documentation;
  }
}

// InlayHint class
export class InlayHint {
  position: any; // Position
  label: string | any[]; // string | InlayHintLabelPart[]
  kind?: InlayHintKind;
  textEdits?: any[]; // TextEdit[]
  tooltip?: string | { value: string };
  paddingLeft?: boolean;
  paddingRight?: boolean;

  constructor(position: any, label: string | any[], kind?: InlayHintKind) {
    this.position = position;
    this.label = label;
    this.kind = kind;
  }
}

export enum InlayHintKind {
  Type = 1,
  Parameter = 2
}

// FoldingRange class
export class FoldingRange {
  start: number;
  end: number;
  kind?: FoldingRangeKind;

  constructor(start: number, end: number, kind?: FoldingRangeKind) {
    this.start = start;
    this.end = end;
    this.kind = kind;
  }
}

export enum FoldingRangeKind {
  Comment = 1,
  Imports = 2,
  Region = 3
}

// SelectionRange class
export class SelectionRange {
  range: any; // Range
  parent?: SelectionRange;

  constructor(range: any, parent?: SelectionRange) {
    this.range = range;
    this.parent = parent;
  }
}

// CallHierarchyItem class
export class CallHierarchyItem {
  name: string;
  kind: any; // SymbolKind
  tags?: any[]; // SymbolTag[]
  detail?: string;
  uri: any; // Uri
  range: any; // Range
  selectionRange: any; // Range

  constructor(kind: any, name: string, detail: string, uri: any, range: any, selectionRange: any) {
    this.kind = kind;
    this.name = name;
    this.detail = detail;
    this.uri = uri;
    this.range = range;
    this.selectionRange = selectionRange;
  }
}

// TypeHierarchyItem class
export class TypeHierarchyItem {
  name: string;
  kind: any; // SymbolKind
  tags?: any[]; // SymbolTag[]
  detail?: string;
  uri: any; // Uri
  range: any; // Range
  selectionRange: any; // Range

  constructor(kind: any, name: string, detail: string, uri: any, range: any, selectionRange: any) {
    this.kind = kind;
    this.name = name;
    this.detail = detail;
    this.uri = uri;
    this.range = range;
    this.selectionRange = selectionRange;
  }
}

// Color class
export class Color {
  readonly red: number;
  readonly green: number;
  readonly blue: number;
  readonly alpha: number;

  constructor(red: number, green: number, blue: number, alpha: number) {
    this.red = red;
    this.green = green;
    this.blue = blue;
    this.alpha = alpha;
  }
}

// ColorInformation class
export class ColorInformation {
  range: any; // Range
  color: Color;

  constructor(range: any, color: Color) {
    this.range = range;
    this.color = color;
  }
}

// ColorPresentation class
export class ColorPresentation {
  label: string;
  textEdit?: any; // TextEdit
  additionalTextEdits?: any[]; // TextEdit[]

  constructor(label: string) {
    this.label = label;
  }
}

// SemanticTokens class
export class SemanticTokens {
  readonly resultId?: string;
  readonly data: Uint32Array;

  constructor(data: Uint32Array, resultId?: string) {
    this.data = data;
    this.resultId = resultId;
  }
}

// SemanticTokensBuilder class
export class SemanticTokensBuilder {
  private _data: number[] = [];

  push(line: number, char: number, length: number, tokenType: number, tokenModifiers?: number): void {
    this._data.push(line, char, length, tokenType, tokenModifiers || 0);
  }

  build(resultId?: string): SemanticTokens {
    return new SemanticTokens(new Uint32Array(this._data), resultId);
  }
}

// SemanticTokensLegend class
export class SemanticTokensLegend {
  readonly tokenTypes: string[];
  readonly tokenModifiers: string[];

  constructor(tokenTypes: string[], tokenModifiers?: string[]) {
    this.tokenTypes = tokenTypes;
    this.tokenModifiers = tokenModifiers || [];
  }
}

// ViewColumn enum
export enum ViewColumn {
  Active = -1,
  Beside = -2,
  One = 1,
  Two = 2,
  Three = 3,
  Four = 4,
  Five = 5,
  Six = 6,
  Seven = 7,
  Eight = 8,
  Nine = 9
}

// CodeActionKind class
export class CodeActionKind {
  static readonly Empty = new CodeActionKind('');
  static readonly QuickFix = new CodeActionKind('quickfix');
  static readonly Refactor = new CodeActionKind('refactor');
  static readonly RefactorExtract = new CodeActionKind('refactor.extract');
  static readonly RefactorInline = new CodeActionKind('refactor.inline');
  static readonly RefactorRewrite = new CodeActionKind('refactor.rewrite');
  static readonly Source = new CodeActionKind('source');
  static readonly SourceOrganizeImports = new CodeActionKind('source.organizeImports');
  static readonly SourceFixAll = new CodeActionKind('source.fixAll');

  readonly value: string;

  private constructor(value: string) {
    this.value = value;
  }

  append(parts: string): CodeActionKind {
    return new CodeActionKind(this.value ? `${this.value}.${parts}` : parts);
  }

  intersects(other: CodeActionKind): boolean {
    return this.value === other.value || other.value.startsWith(this.value + '.');
  }

  contains(other: CodeActionKind): boolean {
    return this.value === other.value || other.value.startsWith(this.value + '.');
  }
}

// InlineCompletionItem class
export class InlineCompletionItem {
  insertText: string | { value: string };
  filterText?: string;
  range?: any; // Range
  command?: any; // Command

  constructor(insertText: string | { value: string }, range?: any, command?: any) {
    this.insertText = insertText;
    this.range = range;
    this.command = command;
  }
}

// InlineCompletionList class
export class InlineCompletionList {
  items: InlineCompletionItem[];

  constructor(items: InlineCompletionItem[]) {
    this.items = items;
  }
}

// LinkedEditingRanges class
export class LinkedEditingRanges {
  readonly ranges: any[]; // Range[]
  readonly wordPattern?: RegExp;

  constructor(ranges: any[], wordPattern?: RegExp) {
    this.ranges = ranges;
    this.wordPattern = wordPattern;
  }
}

// SymbolInformation class (deprecated but still used)
export class SymbolInformation {
  name: string;
  kind: any; // SymbolKind
  containerName?: string;
  location: any; // Location
  tags?: any[]; // SymbolTag[]

  constructor(name: string, kind: any, containerName: string, location: any);
  constructor(name: string, kind: any, range: any, uri?: any, containerName?: string);
  constructor(
    name: string,
    kind: any,
    containerNameOrRange: string | any,
    locationOrUri?: any,
    containerName?: string
  ) {
    this.name = name;
    this.kind = kind;

    if (typeof containerNameOrRange === 'string') {
      // Old signature: (name, kind, containerName, location)
      this.containerName = containerNameOrRange;
      this.location = locationOrUri;
    } else {
      // New signature: (name, kind, range, uri, containerName)
      this.containerName = containerName;
      // Create a Location-like object
      this.location = {
        uri: locationOrUri,
        range: containerNameOrRange
      };
    }
  }
}

// WorkspaceSymbol class
export class WorkspaceSymbol {
  name: string;
  kind: any; // SymbolKind
  containerName?: string;
  location: any; // Location | { uri: Uri }
  tags?: any[]; // SymbolTag[]

  constructor(name: string, kind: any, containerName: string, location: any) {
    this.name = name;
    this.kind = kind;
    this.containerName = containerName;
    this.location = location;
  }
}

// SymbolTag enum
export enum SymbolTag {
  Deprecated = 1
}

// DocumentHighlight class
export class DocumentHighlight {
  range: any; // Range
  kind?: DocumentHighlightKind;

  constructor(range: any, kind?: DocumentHighlightKind) {
    this.range = range;
    this.kind = kind;
  }
}

export enum DocumentHighlightKind {
  Text = 0,
  Read = 1,
  Write = 2
}

// Rename types
export class RenameEdit {
  entries(): IterableIterator<[any, any[]]> {
    return (new Map()).entries();
  }
}

// RelativePattern for glob matching
export class RelativePattern {
  base: string;
  pattern: string;

  constructor(base: string | any, pattern: string) {
    if (typeof base === 'string') {
      this.base = base;
    } else if (base.uri) {
      this.base = base.uri.fsPath || base.uri.path;
    } else {
      this.base = base.fsPath || base.path || String(base);
    }
    this.pattern = pattern;
  }
}
