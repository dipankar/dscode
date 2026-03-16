/**
 * Event System
 *
 * VS Code compatible event emitter and event handling
 */

export interface Event<T> {
  (listener: (e: T) => any, thisArgs?: any, disposables?: Disposable[]): Disposable;
}

export interface Disposable {
  dispose(): void;
}

export class EventEmitter<T> {
  private listeners: Array<{ listener: (e: T) => any; thisArgs?: any }> = [];

  get event(): Event<T> {
    return (listener: (e: T) => any, thisArgs?: any, disposables?: Disposable[]): Disposable => {
      const listenerObj = { listener, thisArgs };
      this.listeners.push(listenerObj);

      const disposable = {
        dispose: () => {
          const index = this.listeners.indexOf(listenerObj);
          if (index !== -1) {
            this.listeners.splice(index, 1);
          }
        }
      };

      if (disposables) {
        disposables.push(disposable);
      }

      return disposable;
    };
  }

  fire(data: T): void {
    for (const { listener, thisArgs } of this.listeners) {
      try {
        listener.call(thisArgs, data);
      } catch (error) {
        console.error('[EventEmitter] Error in event listener:', error);
      }
    }
  }

  dispose(): void {
    this.listeners = [];
  }
}

// Common event types
export interface TextDocumentChangeEvent {
  document: any; // TextDocument
  contentChanges: Array<{
    range: any; // Range
    rangeOffset: number;
    rangeLength: number;
    text: string;
  }>;
}

export interface TextDocumentWillSaveEvent {
  document: any; // TextDocument
  reason: number; // SaveReason
  waitUntil(thenable: Promise<any>): void;
}

export interface TextEditorSelectionChangeEvent {
  textEditor: any; // TextEditor
  selections: any[]; // Selection[]
  kind?: number; // TextEditorSelectionChangeKind
}

export interface TextEditorVisibleRangesChangeEvent {
  textEditor: any; // TextEditor
  visibleRanges: any[]; // Range[]
}

export interface TextEditorOptionsChangeEvent {
  textEditor: any; // TextEditor
  options: any; // TextEditorOptions
}

export interface TextEditorViewColumnChangeEvent {
  textEditor: any; // TextEditor
  viewColumn: number;
}

export interface ConfigurationChangeEvent {
  affectsConfiguration(section: string, scope?: any): boolean;
}

export interface FileSystemWatcherEvent {
  uri: { fsPath: string; path: string };
}

export interface WorkspaceFoldersChangeEvent {
  added: any[]; // WorkspaceFolder[]
  removed: any[]; // WorkspaceFolder[]
}

export interface FileWillCreateEvent {
  files: any[]; // Uri[]
  waitUntil(thenable: Promise<any>): void;
}

export interface FileWillDeleteEvent {
  files: any[]; // Uri[]
  waitUntil(thenable: Promise<any>): void;
}

export interface FileWillRenameEvent {
  files: Array<{ oldUri: any; newUri: any }>; // { oldUri: Uri; newUri: Uri }[]
  waitUntil(thenable: Promise<any>): void;
}

export interface TerminalCloseEvent {
  terminal: any; // Terminal
}

export interface DebugSessionCustomEvent {
  session: any; // DebugSession
  event: string;
  body?: any;
}

// Enums
export enum SaveReason {
  Manual = 1,
  AfterDelay = 2,
  FocusOut = 3
}

export enum TextEditorSelectionChangeKind {
  Keyboard = 1,
  Mouse = 2,
  Command = 3
}
