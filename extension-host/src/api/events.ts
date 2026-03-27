/**
 * Event System
 *
 * VS Code compatible event emitter and event handling
 */

import type { TextDocument, Range } from './textDocument';
import type { Selection, TextEditorOptions } from './textEditor';
import type { Uri } from './uri';

export interface TextEditorEventInfo {
  document: TextDocument;
  selections: Selection[];
  visibleRanges: Range[];
  options: TextEditorOptions;
  viewColumn?: number;
}

export interface Event<T> {
  (listener: (e: T) => any, thisArgs?: any, disposables?: Disposable[]): Disposable;
}

export class Disposable {
  constructor(private callOnDispose?: () => void) {}

  dispose(): void {
    if (this.callOnDispose) {
      this.callOnDispose();
    }
  }

  static from(...disposables: { dispose(): void }[]): Disposable {
    return new Disposable(() => {
      for (const disposable of disposables) {
        if (disposable && typeof disposable.dispose === 'function') {
          disposable.dispose();
        }
      }
    });
  }
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
        },
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
  document: TextDocument;
  contentChanges: Array<{
    range: Range;
    rangeOffset: number;
    rangeLength: number;
    text: string;
  }>;
}

export interface TextDocumentWillSaveEvent {
  document: TextDocument;
  reason: number; // SaveReason
  waitUntil(thenable: Promise<unknown>): void;
}

export interface TextEditorSelectionChangeEvent {
  textEditor: TextEditorEventInfo;
  selections: Selection[];
  kind?: number; // TextEditorSelectionChangeKind
}

export interface TextEditorVisibleRangesChangeEvent {
  textEditor: TextEditorEventInfo;
  visibleRanges: Range[];
}

export interface TextEditorOptionsChangeEvent {
  textEditor: TextEditorEventInfo;
  options: TextEditorOptions;
}

export interface TextEditorViewColumnChangeEvent {
  textEditor: TextEditorEventInfo;
  viewColumn: number;
}

export interface ConfigurationChangeEvent {
  affectsConfiguration(section: string, scope?: unknown): boolean;
}

export interface FileSystemWatcherEvent {
  uri: { fsPath: string; path: string };
}

export interface WorkspaceFoldersChangeEvent {
  added: Array<{ uri: { fsPath: string; scheme: string }; name: string; index: number }>;
  removed: Array<{ uri: { fsPath: string; scheme: string }; name: string; index: number }>;
}

export interface FileWillCreateEvent {
  files: Uri[];
  waitUntil(thenable: Promise<unknown>): void;
}

export interface FileWillDeleteEvent {
  files: Uri[];
  waitUntil(thenable: Promise<unknown>): void;
}

export interface FileWillRenameEvent {
  files: Array<{ oldUri: Uri; newUri: Uri }>;
  waitUntil(thenable: Promise<unknown>): void;
}

export interface TerminalCloseEvent {
  terminal: { name: string; processId: Promise<number | undefined> };
}

export interface DebugSessionCustomEvent {
  session: { id: string; type: string; name: string };
  event: string;
  body?: unknown;
}

// Enums
export enum SaveReason {
  Manual = 1,
  AfterDelay = 2,
  FocusOut = 3,
}

export enum TextEditorSelectionChangeKind {
  Keyboard = 1,
  Mouse = 2,
  Command = 3,
}
