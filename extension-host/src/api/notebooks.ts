/**
 * Notebook API
 *
 * Support for interactive notebooks (Jupyter, etc.)
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';
import { Uri } from './uri';

export enum NotebookCellKind {
  Markup = 1,
  Code = 2
}

export enum NotebookCellExecutionState {
  Idle = 1,
  Pending = 2,
  Executing = 3
}

export interface NotebookCellData {
  kind: NotebookCellKind;
  value: string;
  languageId: string;
  outputs?: NotebookCellOutput[];
  metadata?: { [key: string]: any };
  executionSummary?: NotebookCellExecutionSummary;
}

export interface NotebookCellExecutionSummary {
  executionOrder?: number;
  success?: boolean;
  timing?: { startTime: number; endTime: number };
}

export interface NotebookCellOutput {
  items: NotebookCellOutputItem[];
  metadata?: { [key: string]: any };
}

export interface NotebookCellOutputItem {
  mime: string;
  data: Uint8Array;
}

export interface NotebookCell {
  readonly index: number;
  readonly kind: NotebookCellKind;
  readonly document: any; // TextDocument
  readonly metadata: { [key: string]: any };
  readonly outputs: readonly NotebookCellOutput[];
  readonly executionSummary: NotebookCellExecutionSummary | undefined;
}

export interface NotebookDocument {
  readonly uri: Uri;
  readonly notebookType: string;
  readonly version: number;
  readonly isDirty: boolean;
  readonly isUntitled: boolean;
  readonly isClosed: boolean;
  readonly metadata: { [key: string]: any };
  readonly cellCount: number;
  cellAt(index: number): NotebookCell;
  getCells(range?: any): NotebookCell[];
  save(): Promise<boolean>;
}

export interface NotebookData {
  cells: NotebookCellData[];
  metadata?: { [key: string]: any };
}

export interface NotebookDocumentContentProvider {
  onDidChangeNotebookContentOptions?: Event<any>;
  readonly options?: any;
  openNotebook(uri: Uri, openContext: any): Promise<NotebookData>;
  saveNotebook?(document: NotebookDocument, token: any): Promise<void>;
  saveNotebookAs?(targetResource: Uri, document: NotebookDocument, token: any): Promise<void>;
  backupNotebook?(document: NotebookDocument, context: any, token: any): Promise<any>;
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
  updateNotebookAffinity(notebook: NotebookDocument, affinity: any): void;
}

export interface NotebookCellExecution {
  readonly cell: NotebookCell;
  readonly token: any; // CancellationToken
  executionOrder: number | undefined;
  start(startTime?: number): void;
  end(success: boolean | undefined, endTime?: number): void;
  clearOutput(cell?: NotebookCell): Promise<void>;
  replaceOutput(output: NotebookCellOutput | readonly NotebookCellOutput[], cell?: NotebookCell): Promise<void>;
  appendOutput(output: NotebookCellOutput | readonly NotebookCellOutput[], cell?: NotebookCell): Promise<void>;
  replaceOutputItems(items: NotebookCellOutputItem | readonly NotebookCellOutputItem[], output: NotebookCellOutput): Promise<void>;
  appendOutputItems(items: NotebookCellOutputItem | readonly NotebookCellOutputItem[], output: NotebookCellOutput): Promise<void>;
}

class NotebookCellExecutionImpl implements NotebookCellExecution {
  private _executionOrder?: number;

  constructor(
    private bridge: ExtensionHostBridge,
    public readonly cell: NotebookCell,
    public readonly token: any
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
      startTime
    });
  }

  end(success: boolean | undefined, endTime?: number): void {
    this.bridge.send('notebookCellExecutionEnd', {
      cell: this.cell,
      success,
      endTime
    });
  }

  async clearOutput(cell?: NotebookCell): Promise<void> {
    await this.bridge.request('notebookCellClearOutput', {
      cell: cell || this.cell
    });
  }

  async replaceOutput(output: NotebookCellOutput | readonly NotebookCellOutput[], cell?: NotebookCell): Promise<void> {
    await this.bridge.request('notebookCellReplaceOutput', {
      cell: cell || this.cell,
      output
    });
  }

  async appendOutput(output: NotebookCellOutput | readonly NotebookCellOutput[], cell?: NotebookCell): Promise<void> {
    await this.bridge.request('notebookCellAppendOutput', {
      cell: cell || this.cell,
      output
    });
  }

  async replaceOutputItems(items: NotebookCellOutputItem | readonly NotebookCellOutputItem[], output: NotebookCellOutput): Promise<void> {
    await this.bridge.request('notebookCellReplaceOutputItems', {
      output,
      items
    });
  }

  async appendOutputItems(items: NotebookCellOutputItem | readonly NotebookCellOutputItem[], output: NotebookCellOutput): Promise<void> {
    await this.bridge.request('notebookCellAppendOutputItems', {
      output,
      items
    });
  }
}

class NotebookControllerImpl implements NotebookController {
  private _label: string;
  private _description?: string;
  private _detail?: string;
  private _supportsExecutionOrder?: boolean;
  private _onDidChangeSelectedNotebooks = new EventEmitter<{ notebook: NotebookDocument; selected: boolean }>();

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
    return new NotebookCellExecutionImpl(this.bridge, cell, {});
  }

  updateNotebookAffinity(notebook: NotebookDocument, affinity: any): void {
    this.bridge.send('updateNotebookAffinity', {
      controllerId: this.id,
      notebook,
      affinity
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
    this.bridge.on('notebookOpened', (data: any) => {
      this._onDidOpenNotebookDocument.fire(data.document);
    });

    this.bridge.on('notebookClosed', (data: any) => {
      this._onDidCloseNotebookDocument.fire(data.document);
    });

    this.bridge.on('notebookSaved', (data: any) => {
      this._onDidSaveNotebookDocument.fire(data.document);
    });
  }

  registerNotebookContentProvider(
    notebookType: string,
    provider: NotebookDocumentContentProvider,
    options?: any
  ): Disposable {
    this.bridge.send('registerNotebookContentProvider', {
      notebookType,
      options
    });

    return {
      dispose: () => {
        this.bridge.send('unregisterNotebookContentProvider', { notebookType });
      }
    };
  }

  createNotebookController(
    id: string,
    notebookType: string,
    label: string,
    handler?: (cells: NotebookCell[], notebook: NotebookDocument, controller: NotebookController) => void | Promise<void>,
    rendererScripts?: any[]
  ): NotebookController {
    const controller = new NotebookControllerImpl(this.bridge, id, notebookType, label);

    this.bridge.send('createNotebookController', {
      id,
      notebookType,
      label
    });

    return controller;
  }
}
