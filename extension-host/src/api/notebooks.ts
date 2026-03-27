/**
 * Notebook API
 *
 * Support for interactive notebooks (Jupyter, etc.)
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';
import { Uri } from './uri';
import { CancellationToken } from './common';
import type { TextDocument } from './textDocument';

export enum NotebookCellKind {
  Markup = 1,
  Code = 2,
}

export enum NotebookCellExecutionState {
  Idle = 1,
  Pending = 2,
  Executing = 3,
}

export class NotebookCellData {
  constructor(
    public kind: NotebookCellKind,
    public value: string,
    public languageId: string
  ) {}

  outputs?: NotebookCellOutput[];
  metadata?: Record<string, unknown>;
  executionSummary?: NotebookCellExecutionSummary;
}

export interface NotebookCellExecutionSummary {
  executionOrder?: number;
  success?: boolean;
  timing?: { startTime: number; endTime: number };
}

export class NotebookCellOutput {
  constructor(
    public items: NotebookCellOutputItem[],
    public metadata?: Record<string, unknown>
  ) {}
}

export class NotebookCellOutputItem {
  constructor(
    public data: Uint8Array,
    public mime: string
  ) {}

  static text(value: string, mime?: string): NotebookCellOutputItem {
    const encoder = new TextEncoder();
    return new NotebookCellOutputItem(encoder.encode(value), mime || 'text/plain');
  }

  static json(value: unknown, mime?: string): NotebookCellOutputItem {
    const encoder = new TextEncoder();
    return new NotebookCellOutputItem(
      encoder.encode(JSON.stringify(value)),
      mime || 'application/json'
    );
  }

  static error(value: Error): NotebookCellOutputItem {
    const encoder = new TextEncoder();
    return new NotebookCellOutputItem(
      encoder.encode(value.stack || value.message),
      'application/vnd.code.notebook.error'
    );
  }

  static stdout(value: string): NotebookCellOutputItem {
    const encoder = new TextEncoder();
    return new NotebookCellOutputItem(
      encoder.encode(value),
      'application/vnd.code.notebook.stdout'
    );
  }

  static stderr(value: string): NotebookCellOutputItem {
    const encoder = new TextEncoder();
    return new NotebookCellOutputItem(
      encoder.encode(value),
      'application/vnd.code.notebook.stderr'
    );
  }
}

export class NotebookCell {
  readonly index!: number;
  readonly kind!: NotebookCellKind;
  readonly document!: TextDocument;
  readonly metadata!: Record<string, unknown>;
  readonly outputs!: readonly NotebookCellOutput[];
  readonly executionSummary!: NotebookCellExecutionSummary | undefined;
}

export interface NotebookCellRange {
  start: number;
  end: number;
}

export class NotebookDocument {
  readonly uri!: Uri;
  readonly notebookType!: string;
  readonly version!: number;
  readonly isDirty!: boolean;
  readonly isUntitled!: boolean;
  readonly isClosed!: boolean;
  readonly metadata!: Record<string, unknown>;
  readonly cellCount!: number;
  cellAt(index: number): NotebookCell {
    throw new Error('Not implemented');
  }
  getCells(range?: NotebookCellRange): NotebookCell[] {
    return [];
  }
  save(): Promise<boolean> {
    return Promise.resolve(false);
  }
}

export interface NotebookData {
  cells: NotebookCellData[];
  metadata?: Record<string, unknown>;
}

export interface NotebookDocumentContentProvider {
  onDidChangeNotebookContentOptions?: Event<unknown>;
  readonly options?: Record<string, unknown>;
  openNotebook(uri: Uri, openContext: unknown): Promise<NotebookData>;
  saveNotebook?(document: NotebookDocument, token: CancellationToken): Promise<void>;
  saveNotebookAs?(
    targetResource: Uri,
    document: NotebookDocument,
    token: CancellationToken
  ): Promise<void>;
  backupNotebook?(
    document: NotebookDocument,
    context: unknown,
    token: CancellationToken
  ): Promise<unknown>;
}

export enum NotebookControllerAffinity {
  Default = 1,
  Preferred = 2,
}

export interface NotebookController {
  readonly id: string;
  readonly notebookType: string;
  readonly supportedLanguages?: string[];
  label: string;
  description?: string;
  detail?: string;
  supportsExecutionOrder?: boolean;
  readonly onDidChangeSelectedNotebooks: Event<{ notebook: NotebookDocument; selected: boolean }>;
  createNotebookCellExecution(cell: NotebookCell): NotebookCellExecution;
  dispose(): void;
  updateNotebookAffinity(notebook: NotebookDocument, affinity: NotebookControllerAffinity): void;
}

export interface NotebookCellExecution {
  readonly cell: NotebookCell;
  readonly token: CancellationToken;
  executionOrder: number | undefined;
  start(startTime?: number): void;
  end(success: boolean | undefined, endTime?: number): void;
  clearOutput(cell?: NotebookCell): Promise<void>;
  replaceOutput(
    output: NotebookCellOutput | readonly NotebookCellOutput[],
    cell?: NotebookCell
  ): Promise<void>;
  appendOutput(
    output: NotebookCellOutput | readonly NotebookCellOutput[],
    cell?: NotebookCell
  ): Promise<void>;
  replaceOutputItems(
    items: NotebookCellOutputItem | readonly NotebookCellOutputItem[],
    output: NotebookCellOutput
  ): Promise<void>;
  appendOutputItems(
    items: NotebookCellOutputItem | readonly NotebookCellOutputItem[],
    output: NotebookCellOutput
  ): Promise<void>;
}

class NotebookCellExecutionImpl implements NotebookCellExecution {
  private _executionOrder?: number;

  constructor(
    private bridge: ExtensionHostBridge,
    public readonly cell: NotebookCell,
    public readonly token: CancellationToken
  ) {}

  get executionOrder(): number | undefined {
    return this._executionOrder;
  }

  set executionOrder(value: number | undefined) {
    this._executionOrder = value;
  }

  start(startTime?: number): void {
    this.bridge.send('notebookCellExecutionStart', {
      cell: this.cell,
      startTime,
    });
  }

  end(success: boolean | undefined, endTime?: number): void {
    this.bridge.send('notebookCellExecutionEnd', {
      cell: this.cell,
      success,
      endTime,
    });
  }

  async clearOutput(cell?: NotebookCell): Promise<void> {
    await this.bridge.request('notebookCellClearOutput', {
      cell: cell || this.cell,
    });
  }

  async replaceOutput(
    output: NotebookCellOutput | readonly NotebookCellOutput[],
    cell?: NotebookCell
  ): Promise<void> {
    await this.bridge.request('notebookCellReplaceOutput', {
      cell: cell || this.cell,
      output,
    });
  }

  async appendOutput(
    output: NotebookCellOutput | readonly NotebookCellOutput[],
    cell?: NotebookCell
  ): Promise<void> {
    await this.bridge.request('notebookCellAppendOutput', {
      cell: cell || this.cell,
      output,
    });
  }

  async replaceOutputItems(
    items: NotebookCellOutputItem | readonly NotebookCellOutputItem[],
    output: NotebookCellOutput
  ): Promise<void> {
    await this.bridge.request('notebookCellReplaceOutputItems', {
      output,
      items,
    });
  }

  async appendOutputItems(
    items: NotebookCellOutputItem | readonly NotebookCellOutputItem[],
    output: NotebookCellOutput
  ): Promise<void> {
    await this.bridge.request('notebookCellAppendOutputItems', {
      output,
      items,
    });
  }
}

class NotebookControllerImpl implements NotebookController {
  private _label: string;
  private _description?: string;
  private _detail?: string;
  private _supportsExecutionOrder?: boolean;
  private _onDidChangeSelectedNotebooks = new EventEmitter<{
    notebook: NotebookDocument;
    selected: boolean;
  }>();

  readonly onDidChangeSelectedNotebooks = this._onDidChangeSelectedNotebooks.event;

  constructor(
    private bridge: ExtensionHostBridge,
    public readonly id: string,
    public readonly notebookType: string,
    label: string,
    public readonly supportedLanguages?: string[]
  ) {
    this._label = label;
  }

  get label(): string {
    return this._label;
  }

  set label(value: string) {
    this._label = value;
  }

  get description(): string | undefined {
    return this._description;
  }

  set description(value: string | undefined) {
    this._description = value;
  }

  get detail(): string | undefined {
    return this._detail;
  }

  set detail(value: string | undefined) {
    this._detail = value;
  }

  get supportsExecutionOrder(): boolean | undefined {
    return this._supportsExecutionOrder;
  }

  set supportsExecutionOrder(value: boolean | undefined) {
    this._supportsExecutionOrder = value;
  }

  createNotebookCellExecution(cell: NotebookCell): NotebookCellExecution {
    return new NotebookCellExecutionImpl(this.bridge, cell, {
      isCancellationRequested: false,
      onCancellationRequested: () => ({ dispose: () => {} }),
    });
  }

  updateNotebookAffinity(notebook: NotebookDocument, affinity: NotebookControllerAffinity): void {
    this.bridge.send('updateNotebookAffinity', {
      controllerId: this.id,
      notebook,
      affinity,
    });
  }

  dispose(): void {
    this._onDidChangeSelectedNotebooks.dispose();
    this.bridge.send('disposeNotebookController', { id: this.id });
  }
}

export class NotebooksAPI {
  private _onDidOpenNotebookDocument = new EventEmitter<NotebookDocument>();
  private _onDidCloseNotebookDocument = new EventEmitter<NotebookDocument>();
  private _onDidSaveNotebookDocument = new EventEmitter<NotebookDocument>();

  readonly onDidOpenNotebookDocument = this._onDidOpenNotebookDocument.event;
  readonly onDidCloseNotebookDocument = this._onDidCloseNotebookDocument.event;
  readonly onDidSaveNotebookDocument = this._onDidSaveNotebookDocument.event;

  constructor(private bridge: ExtensionHostBridge) {
    this.setupListeners();
  }

  private setupListeners(): void {
    this.bridge.on('notebookOpened', (data: { document: NotebookDocument }) => {
      this._onDidOpenNotebookDocument.fire(data.document);
    });

    this.bridge.on('notebookClosed', (data: { document: NotebookDocument }) => {
      this._onDidCloseNotebookDocument.fire(data.document);
    });

    this.bridge.on('notebookSaved', (data: { document: NotebookDocument }) => {
      this._onDidSaveNotebookDocument.fire(data.document);
    });
  }

  registerNotebookContentProvider(
    notebookType: string,
    provider: NotebookDocumentContentProvider,
    options?: Record<string, unknown>
  ): Disposable {
    this.bridge.send('registerNotebookContentProvider', {
      notebookType,
      options,
    });

    return {
      dispose: () => {
        this.bridge.send('unregisterNotebookContentProvider', { notebookType });
      },
    };
  }

  createNotebookController(
    id: string,
    notebookType: string,
    label: string,
    handler?: (
      cells: NotebookCell[],
      notebook: NotebookDocument,
      controller: NotebookController
    ) => void | Promise<void>,
    rendererScripts?: unknown[]
  ): NotebookController {
    const controller = new NotebookControllerImpl(this.bridge, id, notebookType, label);

    this.bridge.send('createNotebookController', {
      id,
      notebookType,
      label,
    });

    return controller;
  }
}
