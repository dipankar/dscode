/**
 * Common Types and Utility Classes
 *
 * MarkdownString, ThemeColor, ThemeIcon, and other shared utilities
 */

import { Range, Position } from './textDocument';
import { Uri } from './uri';
import { TextEdit, WorkspaceEdit } from './textEditor';
import { Event, Disposable } from './events';

export class MarkdownString {
  value: string;
  isTrusted?: boolean | { enabledCommands: string[] };
  supportThemeIcons?: boolean;
  supportHtml?: boolean;
  baseUri?: Uri;

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

  appendLink(target: string | Uri, label: string): MarkdownString {
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
  WorkspaceFolder = 3,
}

export enum ExtensionMode {
  Production = 1,
  Development = 2,
  Test = 3,
}

export interface CancellationToken {
  isCancellationRequested: boolean;
  onCancellationRequested: Event<boolean>;
}

export class CancellationTokenSource {
  token: CancellationToken;
  private _isCancelled = false;

  constructor() {
    this.token = {
      isCancellationRequested: false,
      onCancellationRequested: () => ({ dispose: () => {} }),
    };
  }

  cancel(): void {
    this._isCancelled = true;
    this.token.isCancellationRequested = true;
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
  iconPath: Uri | { light: Uri; dark: Uri } | ThemeIcon;
  tooltip?: string;
}

export enum QuickPickItemKind {
  Separator = -1,
  Default = 0,
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
  Notification = 15,
}

export interface Progress<T> {
  report(value: T): void;
}

export interface Memento {
  get<T>(key: string): T | undefined;
  get<T>(key: string, defaultValue: T): T;
  update(key: string, value: unknown): Promise<void>;
  keys(): readonly string[];
}

export class MementoImpl implements Memento {
  private storage = new Map<string, unknown>();

  get<T>(key: string, defaultValue?: T): T | undefined {
    return this.storage.has(key) ? (this.storage.get(key) as T) : defaultValue;
  }

  async update(key: string, value: unknown): Promise<void> {
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
  SymbolicLink = 64,
}

export interface FileStat {
  type: FileType;
  ctime: number;
  mtime: number;
  size: number;
}

export enum FilePermission {
  Readonly = 1,
}

export class FileSystemError extends Error {
  static FileExists(messageOrUri?: string | Uri): FileSystemError {
    return new FileSystemError('EntryExists', messageOrUri);
  }

  static FileNotFound(messageOrUri?: string | Uri): FileSystemError {
    return new FileSystemError('EntryNotFound', messageOrUri);
  }

  static FileNotADirectory(messageOrUri?: string | Uri): FileSystemError {
    return new FileSystemError('EntryNotADirectory', messageOrUri);
  }

  static FileIsADirectory(messageOrUri?: string | Uri): FileSystemError {
    return new FileSystemError('EntryIsADirectory', messageOrUri);
  }

  static NoPermissions(messageOrUri?: string | Uri): FileSystemError {
    return new FileSystemError('NoPermissions', messageOrUri);
  }

  static Unavailable(messageOrUri?: string | Uri): FileSystemError {
    return new FileSystemError('Unavailable', messageOrUri);
  }

  constructor(
    public code: string,
    messageOrUri?: string | Uri
  ) {
    super(typeof messageOrUri === 'string' ? messageOrUri : code);
    this.name = code;
  }
}

// EnvironmentVariableCollection
export enum EnvironmentVariableMutatorType {
  Replace = 1,
  Append = 2,
  Prepend = 3,
}

export class EnvironmentVariableCollection {
  readonly persistent!: boolean;
  readonly description!: string | { value: string } | undefined;

  replace(variable: string, value: string): void {}
  append(variable: string, value: string): void {}
  prepend(variable: string, value: string): void {}
  get(variable: string): { type: EnvironmentVariableMutatorType; value: string } | undefined {
    return undefined;
  }
  forEach(
    callback: (
      variable: string,
      mutator: { type: EnvironmentVariableMutatorType; value: string },
      collection: EnvironmentVariableCollection
    ) => void,
    thisArg?: unknown
  ): void {}
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
  range: Range;
  command?: {
    title: string;
    command: string;
    tooltip?: string;
    arguments?: unknown[];
  };
  isResolved: boolean = false;

  constructor(
    range: Range,
    command?: { title: string; command: string; tooltip?: string; arguments?: unknown[] }
  ) {
    this.range = range;
    this.command = command;
    this.isResolved = !!command;
  }
}

// l10n API for localization
export const l10n = {
  t(message: string, ...args: unknown[]): string {
    if (args.length === 0) return message;
    if (args.length === 1 && typeof args[0] === 'object' && args[0] !== null) {
      const obj = args[0] as Record<string, unknown>;
      return message.replace(/\{(\w+)\}/g, (_, key) => String(obj[key] ?? `{${key}}`));
    }
    return message.replace(/\{(\d+)\}/g, (_, index) =>
      String(args[parseInt(index)] ?? `{${index}}`)
    );
  },
  bundle: undefined as string | undefined,
  uri: undefined as Uri | undefined,
};

// DocumentLink class
export class DocumentLink {
  range: Range;
  target?: Uri;

  constructor(range: Range, target?: Uri) {
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

// InlayHintLabelPart class
export class InlayHintLabelPart {
  value: string;
  tooltip?: string | MarkdownString | { value: string };
  location?: { uri: Uri; range: Range };
  command?: { title: string; command: string; arguments?: unknown[] };

  constructor(value: string) {
    this.value = value;
  }
}

// InlayHint class
export class InlayHint {
  position: Position;
  label: string | InlayHintLabelPart[];
  kind?: InlayHintKind;
  textEdits?: TextEdit[];
  tooltip?: string | MarkdownString | { value: string };
  paddingLeft?: boolean;
  paddingRight?: boolean;

  constructor(position: Position, label: string | InlayHintLabelPart[], kind?: InlayHintKind) {
    this.position = position;
    this.label = label;
    this.kind = kind;
  }
}

export enum InlayHintKind {
  Type = 1,
  Parameter = 2,
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
  Region = 3,
}

// SelectionRange class
export class SelectionRange {
  range: Range;
  parent?: SelectionRange;

  constructor(range: Range, parent?: SelectionRange) {
    this.range = range;
    this.parent = parent;
  }
}

// CallHierarchyItem class
export class CallHierarchyItem {
  name: string;
  kind: SymbolKind;
  tags?: SymbolTag[];
  detail?: string;
  uri: Uri;
  range: Range;
  selectionRange: Range;

  constructor(
    kind: SymbolKind,
    name: string,
    detail: string,
    uri: Uri,
    range: Range,
    selectionRange: Range
  ) {
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
  kind: SymbolKind;
  tags?: SymbolTag[];
  detail?: string;
  uri: Uri;
  range: Range;
  selectionRange: Range;

  constructor(
    kind: SymbolKind,
    name: string,
    detail: string,
    uri: Uri,
    range: Range,
    selectionRange: Range
  ) {
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
  range: Range;
  color: Color;

  constructor(range: Range, color: Color) {
    this.range = range;
    this.color = color;
  }
}

// ColorPresentation class
export class ColorPresentation {
  label: string;
  textEdit?: TextEdit;
  additionalTextEdits?: TextEdit[];

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

  push(
    line: number,
    char: number,
    length: number,
    tokenType: number,
    tokenModifiers?: number
  ): void {
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
  Nine = 9,
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
  range?: Range;
  command?: { title: string; command: string; arguments?: unknown[] };

  constructor(
    insertText: string | { value: string },
    range?: Range,
    command?: { title: string; command: string; arguments?: unknown[] }
  ) {
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
  readonly ranges: Range[];
  readonly wordPattern?: RegExp;

  constructor(ranges: Range[], wordPattern?: RegExp) {
    this.ranges = ranges;
    this.wordPattern = wordPattern;
  }
}

// SymbolKind enum
export enum SymbolKind {
  File = 0,
  Module = 1,
  Namespace = 2,
  Package = 3,
  Class = 4,
  Method = 5,
  Property = 6,
  Field = 7,
  Constructor = 8,
  Enum = 9,
  Interface = 10,
  Function = 11,
  Variable = 12,
  Constant = 13,
  String = 14,
  Number = 15,
  Boolean = 16,
  Array = 17,
  Object = 18,
  Key = 19,
  Null = 20,
  EnumMember = 21,
  Struct = 22,
  Event = 23,
  Operator = 24,
  TypeParameter = 25,
}

// SymbolInformation class (deprecated but still used)
export class SymbolInformation {
  name: string;
  kind: SymbolKind;
  containerName?: string;
  location: { uri: Uri; range: Range };
  tags?: SymbolTag[];

  constructor(
    name: string,
    kind: SymbolKind,
    containerName: string,
    location: { uri: Uri; range: Range }
  );
  constructor(name: string, kind: SymbolKind, range: Range, uri?: Uri, containerName?: string);
  constructor(
    name: string,
    kind: SymbolKind,
    containerNameOrRange: string | Range,
    locationOrUri?: { uri: Uri; range: Range } | Uri,
    containerName?: string
  ) {
    this.name = name;
    this.kind = kind;

    if (typeof containerNameOrRange === 'string') {
      this.containerName = containerNameOrRange;
      this.location = locationOrUri as { uri: Uri; range: Range };
    } else {
      this.containerName = containerName;
      this.location = {
        uri: locationOrUri as Uri,
        range: containerNameOrRange as Range,
      };
    }
  }
}

// WorkspaceSymbol class
export class WorkspaceSymbol {
  name: string;
  kind: SymbolKind;
  containerName?: string;
  location: { uri: Uri; range: Range };
  tags?: SymbolTag[];

  constructor(
    name: string,
    kind: SymbolKind,
    containerName: string,
    location: { uri: Uri; range: Range }
  ) {
    this.name = name;
    this.kind = kind;
    this.containerName = containerName;
    this.location = location;
  }
}

// SymbolTag enum
export enum SymbolTag {
  Deprecated = 1,
}

// DocumentHighlight class
export class DocumentHighlight {
  range: Range;
  kind?: DocumentHighlightKind;

  constructor(range: Range, kind?: DocumentHighlightKind) {
    this.range = range;
    this.kind = kind;
  }
}

export enum DocumentHighlightKind {
  Text = 0,
  Read = 1,
  Write = 2,
}

// Rename types
export class RenameEdit {
  entries(): IterableIterator<[Uri, TextEdit[]]> {
    return new Map<Uri, TextEdit[]>().entries();
  }
}

export enum LanguageStatusSeverity {
  Information = 0,
  Warning = 1,
  Error = 2,
}

export class LanguageStatusItem {
  id: string;
  name: string;
  severity: LanguageStatusSeverity;
  text: string;
  detail?: string;
  busy: boolean;
  command?: { title: string; command: string; arguments?: unknown[] };

  constructor(id: string, name: string, severity?: LanguageStatusSeverity) {
    this.id = id;
    this.name = name;
    this.severity = severity ?? LanguageStatusSeverity.Information;
    this.text = '';
    this.busy = false;
  }

  dispose(): void {}
}

// RelativePattern for glob matching
export class RelativePattern {
  base: string;
  pattern: string;

  constructor(base: string | Uri, pattern: string) {
    if (typeof base === 'string') {
      this.base = base;
    } else {
      this.base = base.fsPath || base.path;
    }
    this.pattern = pattern;
  }
}
