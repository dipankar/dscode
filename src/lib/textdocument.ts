import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface TextDocument {
  uri: string;
  language_id: string;
  version: number;
  line_count: number;
  is_untitled: boolean;
  is_dirty: boolean;
}

export interface TextEditor {
  id: string;
  document: TextDocument;
  selections: Selection[];
  visible_ranges: Range[];
  options: TextEditorOptions;
}

export interface TextEditorOptions {
  tab_size: number;
  insert_spaces: boolean;
  cursor_style: CursorStyle;
  line_numbers: LineNumbersStyle;
}

export enum CursorStyle {
  Line = 'line',
  Block = 'block',
  Underline = 'underline',
}

export enum LineNumbersStyle {
  Off = 'off',
  On = 'on',
  Relative = 'relative',
}

export interface Position {
  line: number;
  character: number;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface Selection {
  anchor: Position;
  active: Position;
}

export interface TextEdit {
  range: Range;
  new_text: string;
}

export interface TextDocumentChangeEvent {
  document: TextDocument;
  content_changes: TextDocumentContentChange[];
}

export interface TextDocumentContentChange {
  range?: Range;
  range_offset?: number;
  range_length?: number;
  text: string;
}

export interface TextEditorDecoration {
  id: string;
  owner: string;
  decoration_type: string;
  ranges: Range[];
}

export interface DecorationRenderOptions {
  background_color?: string;
  border?: string;
  border_color?: string;
  border_radius?: string;
  border_width?: string;
  color?: string;
  cursor?: string;
  text_decoration?: string;
  outline?: string;
  outline_color?: string;
  outline_width?: string;
}

export interface DecorationType {
  id: string;
  owner: string;
  options: DecorationRenderOptions;
}

/**
 * Text Document and Editor API manager
 */
export class TextDocumentManager {
  private documents: Map<string, TextDocument> = new Map();
  private editors: Map<string, TextEditor> = new Map();
  private decorationTypes: Map<string, DecorationType> = new Map();

  private onDidOpenTextDocumentCallbacks: Array<(document: TextDocument) => void> = [];
  private onDidCloseTextDocumentCallbacks: Array<(document: TextDocument) => void> = [];
  private onDidChangeTextDocumentCallbacks: Array<(event: TextDocumentChangeEvent) => void> = [];
  private onDidSaveTextDocumentCallbacks: Array<(document: TextDocument) => void> = [];

  private onDidChangeTextEditorSelectionCallbacks: Array<(editor: TextEditor) => void> = [];
  private onDidChangeTextEditorVisibleRangesCallbacks: Array<(editor: TextEditor) => void> = [];

  constructor() {}

  /**
   * Initialize text document manager
   */
  async initialize(): Promise<void> {
    // Listen for document opened events
    await listen<TextDocument>('text-document-opened', (event) => {
      this.documents.set(event.payload.uri, event.payload);
      this.notifyDocumentOpened(event.payload);
    });

    // Listen for document closed events
    await listen<TextDocument>('text-document-closed', (event) => {
      this.documents.delete(event.payload.uri);
      this.notifyDocumentClosed(event.payload);
    });

    // Listen for document changed events
    await listen<TextDocumentChangeEvent>('text-document-changed', (event) => {
      if (event.payload.document) {
        this.documents.set(event.payload.document.uri, event.payload.document);
      }
      this.notifyDocumentChanged(event.payload);
    });

    // Listen for document saved events
    await listen<TextDocument>('text-document-saved', (event) => {
      this.documents.set(event.payload.uri, event.payload);
      this.notifyDocumentSaved(event.payload);
    });

    // Listen for editor opened events
    await listen<TextEditor>('text-editor-opened', (event) => {
      this.editors.set(event.payload.id, event.payload);
    });

    // Listen for editor selection changed events
    await listen<TextEditor>('text-editor-selection-changed', (event) => {
      this.editors.set(event.payload.id, event.payload);
      this.notifySelectionChanged(event.payload);
    });

    // Listen for editor visible ranges changed events
    await listen<TextEditor>('text-editor-visible-ranges-changed', (event) => {
      this.editors.set(event.payload.id, event.payload);
      this.notifyVisibleRangesChanged(event.payload);
    });

    console.log('[TextDocument] Initialized');
  }

  // ===== Text Document Operations =====

  /**
   * Register a text document
   */
  async registerTextDocument(document: TextDocument): Promise<void> {
    await invoke('register_text_document', { document });
  }

  /**
   * Unregister a text document
   */
  async unregisterTextDocument(uri: string): Promise<void> {
    await invoke('unregister_text_document', { uri });
  }

  /**
   * Update text document
   */
  async updateTextDocument(
    uri: string,
    version: number,
    contentChanges: TextDocumentContentChange[]
  ): Promise<void> {
    await invoke('update_text_document', {
      uri,
      version,
      contentChanges,
    });
  }

  /**
   * Mark document as saved
   */
  async markDocumentSaved(uri: string): Promise<void> {
    await invoke('mark_document_saved', { uri });
  }

  /**
   * Get text document
   */
  async getTextDocument(uri: string): Promise<TextDocument | null> {
    try {
      return await invoke<TextDocument>('get_text_document', { uri });
    } catch {
      return null;
    }
  }

  /**
   * Get all text documents
   */
  async getAllTextDocuments(): Promise<TextDocument[]> {
    return await invoke<TextDocument[]>('get_all_text_documents');
  }

  // ===== Text Editor Operations =====

  /**
   * Register a text editor
   */
  async registerTextEditor(editor: TextEditor): Promise<void> {
    await invoke('register_text_editor', { editor });
  }

  /**
   * Unregister a text editor
   */
  async unregisterTextEditor(editorId: string): Promise<void> {
    await invoke('unregister_text_editor', { editorId });
  }

  /**
   * Update editor selections
   */
  async updateTextEditorSelections(editorId: string, selections: Selection[]): Promise<void> {
    await invoke('update_text_editor_selections', {
      editorId,
      selections,
    });
  }

  /**
   * Update editor visible ranges
   */
  async updateTextEditorVisibleRanges(editorId: string, visibleRanges: Range[]): Promise<void> {
    await invoke('update_text_editor_visible_ranges', {
      editorId,
      visibleRanges,
    });
  }

  /**
   * Get text editor
   */
  async getTextEditor(editorId: string): Promise<TextEditor | null> {
    try {
      return await invoke<TextEditor>('get_text_editor', { editorId });
    } catch {
      return null;
    }
  }

  /**
   * Get all text editors
   */
  async getAllTextEditors(): Promise<TextEditor[]> {
    return await invoke<TextEditor[]>('get_all_text_editors');
  }

  // ===== Text Editor Decorations =====

  /**
   * Create decoration type
   */
  async createTextEditorDecorationType(
    decorationTypeId: string,
    owner: string,
    options: DecorationRenderOptions
  ): Promise<string> {
    const decorationType: DecorationType = {
      id: decorationTypeId,
      owner,
      options,
    };

    const id = await invoke<string>('create_text_editor_decoration_type', {
      decorationType,
    });

    this.decorationTypes.set(id, decorationType);

    return id;
  }

  /**
   * Dispose decoration type
   */
  async disposeTextEditorDecorationType(decorationTypeId: string): Promise<void> {
    await invoke('dispose_text_editor_decoration_type', { decorationTypeId });
    this.decorationTypes.delete(decorationTypeId);
  }

  /**
   * Set editor decorations
   */
  async setTextEditorDecorations(
    editorId: string,
    decorationTypeId: string,
    ranges: Range[],
    owner: string
  ): Promise<void> {
    await invoke('set_text_editor_decorations', {
      editorId,
      decorationTypeId,
      ranges,
      owner,
    });
  }

  /**
   * Get editor decorations
   */
  async getTextEditorDecorations(editorId: string): Promise<TextEditorDecoration[]> {
    return await invoke<TextEditorDecoration[]>('get_text_editor_decorations', {
      editorId,
    });
  }

  /**
   * Subscribe to decoration changes for an editor
   */
  onDidChangeDecorations(editorId: string, callback: () => void): () => void {
    const eventName = `editor-decorations-changed:${editorId}`;
    const unlisten = listen(eventName, () => callback());

    // Return unsubscribe function
    return () => {
      unlisten.then((fn) => fn());
    };
  }

  // ===== Text Edits =====

  /**
   * Apply text edits to document
   */
  async applyTextEdits(uri: string, edits: TextEdit[]): Promise<void> {
    await invoke('apply_text_edits', { uri, edits });
  }

  /**
   * Create a text edit
   */
  createTextEdit(range: Range, newText: string): TextEdit {
    return { range, new_text: newText };
  }

  // ===== Event Subscriptions =====

  /**
   * Subscribe to document opened events
   */
  onDidOpenTextDocument(callback: (document: TextDocument) => void): () => void {
    this.onDidOpenTextDocumentCallbacks.push(callback);

    return () => {
      const index = this.onDidOpenTextDocumentCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidOpenTextDocumentCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to document closed events
   */
  onDidCloseTextDocument(callback: (document: TextDocument) => void): () => void {
    this.onDidCloseTextDocumentCallbacks.push(callback);

    return () => {
      const index = this.onDidCloseTextDocumentCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidCloseTextDocumentCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to document changed events
   */
  onDidChangeTextDocument(callback: (event: TextDocumentChangeEvent) => void): () => void {
    this.onDidChangeTextDocumentCallbacks.push(callback);

    return () => {
      const index = this.onDidChangeTextDocumentCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeTextDocumentCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to document saved events
   */
  onDidSaveTextDocument(callback: (document: TextDocument) => void): () => void {
    this.onDidSaveTextDocumentCallbacks.push(callback);

    return () => {
      const index = this.onDidSaveTextDocumentCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidSaveTextDocumentCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to editor selection changed events
   */
  onDidChangeTextEditorSelection(callback: (editor: TextEditor) => void): () => void {
    this.onDidChangeTextEditorSelectionCallbacks.push(callback);

    return () => {
      const index = this.onDidChangeTextEditorSelectionCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeTextEditorSelectionCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to editor visible ranges changed events
   */
  onDidChangeTextEditorVisibleRanges(callback: (editor: TextEditor) => void): () => void {
    this.onDidChangeTextEditorVisibleRangesCallbacks.push(callback);

    return () => {
      const index = this.onDidChangeTextEditorVisibleRangesCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeTextEditorVisibleRangesCallbacks.splice(index, 1);
      }
    };
  }

  // ===== Utility Methods =====

  /**
   * Clear text document data for owner
   */
  async clearTextDocumentData(owner: string): Promise<void> {
    await invoke('clear_textdocument_data', { owner });
  }

  /**
   * Get cached documents (local state)
   */
  get cachedDocuments(): TextDocument[] {
    return Array.from(this.documents.values());
  }

  /**
   * Get cached editors (local state)
   */
  get cachedEditors(): TextEditor[] {
    return Array.from(this.editors.values());
  }

  // ===== Private Methods =====

  /**
   * Notify document opened listeners
   */
  private notifyDocumentOpened(document: TextDocument): void {
    for (const callback of this.onDidOpenTextDocumentCallbacks) {
      try {
        callback(document);
      } catch (error) {
        console.error('[TextDocument] Error in document opened callback:', error);
      }
    }
  }

  /**
   * Notify document closed listeners
   */
  private notifyDocumentClosed(document: TextDocument): void {
    for (const callback of this.onDidCloseTextDocumentCallbacks) {
      try {
        callback(document);
      } catch (error) {
        console.error('[TextDocument] Error in document closed callback:', error);
      }
    }
  }

  /**
   * Notify document changed listeners
   */
  private notifyDocumentChanged(event: TextDocumentChangeEvent): void {
    for (const callback of this.onDidChangeTextDocumentCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[TextDocument] Error in document changed callback:', error);
      }
    }
  }

  /**
   * Notify document saved listeners
   */
  private notifyDocumentSaved(document: TextDocument): void {
    for (const callback of this.onDidSaveTextDocumentCallbacks) {
      try {
        callback(document);
      } catch (error) {
        console.error('[TextDocument] Error in document saved callback:', error);
      }
    }
  }

  /**
   * Notify selection changed listeners
   */
  private notifySelectionChanged(editor: TextEditor): void {
    for (const callback of this.onDidChangeTextEditorSelectionCallbacks) {
      try {
        callback(editor);
      } catch (error) {
        console.error('[TextDocument] Error in selection changed callback:', error);
      }
    }
  }

  /**
   * Notify visible ranges changed listeners
   */
  private notifyVisibleRangesChanged(editor: TextEditor): void {
    for (const callback of this.onDidChangeTextEditorVisibleRangesCallbacks) {
      try {
        callback(editor);
      } catch (error) {
        console.error('[TextDocument] Error in visible ranges changed callback:', error);
      }
    }
  }
}

// Export singleton instance
export const textDocumentManager = new TextDocumentManager();
