/**
 * TextEditor API
 *
 * Represents an editor that is attached to a document
 */

import { ExtensionHostBridge } from '../bridge';
import { TextDocument, Position, Range } from './textDocument';

export class Selection extends Range {
  public readonly anchor: Position;
  public readonly active: Position;

  constructor(anchor: Position, active: Position);
  constructor(anchorLine: number, anchorCharacter: number, activeLine: number, activeCharacter: number);
  constructor(
    anchorOrAnchorLine: Position | number,
    activeOrAnchorCharacter: Position | number,
    activeLine?: number,
    activeCharacter?: number
  ) {
    let anchor: Position;
    let active: Position;

    if (anchorOrAnchorLine instanceof Position && activeOrAnchorCharacter instanceof Position) {
      anchor = anchorOrAnchorLine;
      active = activeOrAnchorCharacter;
    } else if (typeof anchorOrAnchorLine === 'number' && typeof activeOrAnchorCharacter === 'number') {
      anchor = new Position(anchorOrAnchorLine, activeOrAnchorCharacter);
      active = new Position(activeLine!, activeCharacter!);
    } else {
      throw new Error('Invalid Selection constructor arguments');
    }

    super(anchor.isBefore(active) ? anchor : active, anchor.isAfter(active) ? anchor : active);
    this.anchor = anchor;
    this.active = active;
  }

  get isReversed(): boolean {
    return this.anchor.isAfter(this.active);
  }
}

export interface TextEditorOptions {
  tabSize?: number;
  insertSpaces?: boolean;
  cursorStyle?: number;
  lineNumbers?: number;
}

export class TextEdit {
  constructor(
    public range: Range,
    public newText: string
  ) {}

  static replace(range: Range, newText: string): TextEdit {
    return new TextEdit(range, newText);
  }

  static insert(position: Position, newText: string): TextEdit {
    return new TextEdit(new Range(position, position), newText);
  }

  static delete(range: Range): TextEdit {
    return new TextEdit(range, '');
  }

  static setEndOfLine(eol: number): TextEdit {
    return new TextEdit(new Range(new Position(0, 0), new Position(0, 0)), '');
  }
}

export class WorkspaceEdit {
  private _edits = new Map<string, TextEdit[]>();

  entries(): [{ path: string }, TextEdit[]][] {
    const result: [{ path: string }, TextEdit[]][] = [];
    for (const [path, edits] of this._edits.entries()) {
      result.push([{ path }, edits]);
    }
    return result;
  }

  set(uri: { path: string }, edits: TextEdit[]): void {
    this._edits.set(uri.path, edits);
  }

  get(uri: { path: string }): TextEdit[] | undefined {
    return this._edits.get(uri.path);
  }

  has(uri: { path: string }): boolean {
    return this._edits.has(uri.path);
  }

  delete(uri: { path: string }): boolean {
    return this._edits.delete(uri.path);
  }

  replace(uri: { path: string }, range: Range, newText: string): void {
    const edits = this._edits.get(uri.path) || [];
    edits.push(new TextEdit(range, newText));
    this._edits.set(uri.path, edits);
  }

  insert(uri: { path: string }, position: Position, newText: string): void {
    this.replace(uri, new Range(position, position), newText);
  }

  deleteEdit(uri: { path: string }, range: Range): void {
    this.replace(uri, range, '');
  }

  get size(): number {
    return this._edits.size;
  }
}

export class SnippetString {
  value: string;

  constructor(value?: string) {
    this.value = value || '';
  }

  appendText(string: string): SnippetString {
    this.value += string.replace(/\$|}/g, '\\$&').replace(/\\/g, '\\\\');
    return this;
  }

  appendTabstop(number: number = 0): SnippetString {
    this.value += '$' + number;
    return this;
  }

  appendPlaceholder(
    value: string | ((snippet: SnippetString) => any),
    number: number = 0
  ): SnippetString {
    if (typeof value === 'function') {
      const nested = new SnippetString();
      value(nested);
      this.value += '${' + number + ':' + nested.value + '}';
    } else {
      this.value += '${' + number + ':' + value + '}';
    }
    return this;
  }

  appendChoice(values: string[], number: number = 0): SnippetString {
    this.value += '${' + number + '|' + values.join(',') + '|}';
    return this;
  }

  appendVariable(
    name: string,
    defaultValue: string | ((snippet: SnippetString) => any)
  ): SnippetString {
    if (typeof defaultValue === 'function') {
      const nested = new SnippetString();
      defaultValue(nested);
      this.value += '${' + name + ':' + nested.value + '}';
    } else {
      this.value += '${' + name + ':' + defaultValue + '}';
    }
    return this;
  }
}

export enum TextEditorRevealType {
  Default = 0,
  InCenter = 1,
  InCenterIfOutsideViewport = 2,
  AtTop = 3
}

export enum TextEditorCursorStyle {
  Line = 1,
  Block = 2,
  Underline = 3,
  LineThin = 4,
  BlockOutline = 5,
  UnderlineThin = 6
}

export enum TextEditorLineNumbersStyle {
  Off = 0,
  On = 1,
  Relative = 2
}

export interface TextEditorDecorationType {
  key: string;
  dispose(): void;
}

class TextEditorDecorationTypeImpl implements TextEditorDecorationType {
  constructor(
    private bridge: ExtensionHostBridge,
    public readonly key: string,
    private options: any
  ) {
    void this.bridge.send('registerDecorationType', {
      key: this.key,
      options: this.options || {}
    });
  }

  dispose(): void {
    void this.bridge.send('disposeDecorationType', { key: this.key });
  }
}

export interface DecorationOptions {
  range: Range;
  hoverMessage?: string | { value: string }[];
  renderOptions?: any;
}

export class TextEditor {
  private bridge: ExtensionHostBridge;

  constructor(
    bridge: ExtensionHostBridge,
    public readonly document: TextDocument,
    public selection: Selection,
    public selections: Selection[],
    public visibleRanges: Range[],
    public options: TextEditorOptions
  ) {
    this.bridge = bridge;
  }

  async edit(callback: (editBuilder: TextEditorEdit) => void): Promise<boolean> {
    const editBuilder = new TextEditorEdit();
    callback(editBuilder);

    try {
      const result = await this.bridge.request('applyEdits', {
        uri: this.document.uri.fsPath,
        edits: editBuilder.getEdits(),
        endOfLine: editBuilder.getEndOfLine()
      }) as { content?: string; version?: number } | null;
      if (result?.content) {
        this.document.updateContent(result.content, result?.version);
      }
      return true;
    } catch (error) {
      console.error('[TextEditor] Edit failed:', error);
      return false;
    }
  }

  async insertSnippet(
    snippet: string | SnippetString,
    location?: Position | Range | Position[] | Range[]
  ): Promise<boolean> {
    try {
      const snippetText = typeof snippet === 'string' ? snippet : snippet.value;
      const result = await this.bridge.request('insertSnippet', {
        uri: this.document.uri.fsPath,
        snippet: snippetText,
        location
      }) as { content?: string; version?: number } | null;
      if (result?.content) {
        this.document.updateContent(result.content, result?.version);
      }
      return true;
    } catch (error) {
      console.error('[TextEditor] Insert snippet failed:', error);
      return false;
    }
  }

  setDecorations(decorationType: any, rangesOrOptions: Range[] | any[]): void {
    const key = decorationType?.key;
    if (!key) {
      console.warn('[TextEditor] setDecorations called without a valid decoration type');
      return;
    }

    void this.bridge.send('setDecorations', {
      uri: this.document.uri.fsPath,
      key,
      decorations: rangesOrOptions
    });
  }

  revealRange(range: Range, revealType?: number): void {
    this.bridge.send('revealRange', {
      uri: this.document.uri.fsPath,
      range,
      revealType
    });
  }

  hide(): void {
    this.bridge.send('hideEditor', {
      uri: this.document.uri.fsPath
    });
  }

  show(column?: number): void {
    this.bridge.send('showEditor', {
      uri: this.document.uri.fsPath,
      column
    });
  }
}

export class TextEditorEdit {
  private edits: TextEdit[] = [];
  private endOfLine?: number;

  replace(location: Position | Range | Selection, value: string): void {
    const range = this.toRange(location);
    this.edits.push({ range, newText: value });
  }

  insert(location: Position, value: string): void {
    this.edits.push({
      range: new Range(location, location),
      newText: value
    });
  }

  delete(location: Range | Selection): void {
    const range = this.toRange(location);
    this.edits.push({ range, newText: '' });
  }

  setEndOfLine(endOfLine: number): void {
    this.endOfLine = endOfLine;
  }

  getEdits(): TextEdit[] {
    return this.edits;
  }

  getEndOfLine(): number | undefined {
    return this.endOfLine;
  }

  private toRange(location: Position | Range | Selection): Range {
    if ('start' in location && 'end' in location) {
      return new Range(location.start, location.end);
    }
    return new Range(location, location);
  }
}

/**
 * TextEditorAPI manages text editors
 */
export class TextEditorAPI {
  private editors = new Map<string, TextEditor>();
  private activeEditor: TextEditor | undefined;

  constructor(private bridge: ExtensionHostBridge) {
    this.setupListeners();
  }

  private setupListeners(): void {
    // Listen for editor changes from main process
    this.bridge.on('activeEditorChanged', (data: any) => {
      const editor = this.editors.get(data.uri);
      this.activeEditor = editor;
    });

    this.bridge.on('selectionChanged', (data: any) => {
      const editor = this.editors.get(data.uri);
      if (editor && data.selection) {
        editor.selection = data.selection;
        editor.selections = data.selections || [data.selection];
      }
    });

    this.bridge.on('visibleRangesChanged', (data: any) => {
      const editor = this.editors.get(data.uri);
      if (editor && Array.isArray(data.ranges)) {
        editor.visibleRanges = data.ranges;
      }
    });
  }

  createEditor(document: TextDocument): TextEditor {
    const zeroPos = new Position(0, 0);
    const editor = new TextEditor(
      this.bridge,
      document,
      new Selection(zeroPos, zeroPos),
      [],
      [],
      {
        tabSize: 4,
        insertSpaces: true,
        cursorStyle: 1,
        lineNumbers: 1
      }
    );

    this.editors.set(document.uri.fsPath, editor);
    return editor;
  }

  getActiveEditor(): TextEditor | undefined {
    return this.activeEditor;
  }

  getEditor(uri: string): TextEditor | undefined {
    return this.editors.get(uri);
  }

  getAllEditors(): TextEditor[] {
    return Array.from(this.editors.values());
  }

  setActiveEditor(editor: TextEditor): void {
    this.activeEditor = editor;
  }

  createDecorationType(options: any): TextEditorDecorationType {
    const key = `decoration_${Date.now()}_${Math.random().toString(36).slice(2)}`;
    return new TextEditorDecorationTypeImpl(this.bridge, key, options);
  }
}
