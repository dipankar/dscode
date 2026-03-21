/**
 * TextEditor API
 *
 * Represents an editor that is attached to a document
 */

import { ExtensionHostBridge } from '../bridge';
import { TextDocument, Position, Range } from './textDocument';

export interface Selection extends Range {
  anchor: Position;
  active: Position;
  isReversed: boolean;
}

export interface TextEditorOptions {
  tabSize?: number;
  insertSpaces?: boolean;
  cursorStyle?: number;
  lineNumbers?: number;
}

export interface TextEdit {
  range: Range;
  newText: string;
}

export interface WorkspaceEdit {
  entries(): [{ path: string }, TextEdit[]][];
  set(uri: { path: string }, edits: TextEdit[]): void;
  get(uri: { path: string }): TextEdit[] | undefined;
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
      });
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
      });
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
      range: { start: location, end: location },
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
      return { start: location.start, end: location.end };
    }
    return { start: location, end: location };
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
    const editor = new TextEditor(
      this.bridge,
      document,
      {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 0 },
        anchor: { line: 0, character: 0 },
        active: { line: 0, character: 0 },
        isReversed: false
      },
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
