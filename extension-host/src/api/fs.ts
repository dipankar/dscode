/**
 * FileSystem API
 *
 * File system watching and operations
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';

export interface FileSystemWatcher extends Disposable {
  readonly ignoreCreateEvents: boolean;
  readonly ignoreChangeEvents: boolean;
  readonly ignoreDeleteEvents: boolean;
  readonly onDidCreate: Event<any>;
  readonly onDidChange: Event<any>;
  readonly onDidDelete: Event<any>;
  dispose(): void;
}

class FileSystemWatcherImpl implements FileSystemWatcher {
  private _onDidCreate = new EventEmitter<any>();
  private _onDidChange = new EventEmitter<any>();
  private _onDidDelete = new EventEmitter<any>();

  readonly onDidCreate = this._onDidCreate.event;
  readonly onDidChange = this._onDidChange.event;
  readonly onDidDelete = this._onDidDelete.event;

  constructor(
    private bridge: ExtensionHostBridge,
    private watcherId: string,
    private globPattern: string,
    public readonly ignoreCreateEvents: boolean,
    public readonly ignoreChangeEvents: boolean,
    public readonly ignoreDeleteEvents: boolean
  ) {
    this.setupListeners();
  }

  private setupListeners(): void {
    this.bridge.on(`fileWatcher:${this.watcherId}:create`, (data: any) => {
      if (!this.ignoreCreateEvents) {
        this._onDidCreate.fire(data.uri);
      }
    });

    this.bridge.on(`fileWatcher:${this.watcherId}:change`, (data: any) => {
      if (!this.ignoreChangeEvents) {
        this._onDidChange.fire(data.uri);
      }
    });

    this.bridge.on(`fileWatcher:${this.watcherId}:delete`, (data: any) => {
      if (!this.ignoreDeleteEvents) {
        this._onDidDelete.fire(data.uri);
      }
    });
  }

  dispose(): void {
    this._onDidCreate.dispose();
    this._onDidChange.dispose();
    this._onDidDelete.dispose();
    this.bridge.send('disposeFileWatcher', { watcherId: this.watcherId });
  }
}

export class FileSystemAPI {
  constructor(private bridge: ExtensionHostBridge) {}

  createFileSystemWatcher(
    globPattern: string,
    ignoreCreateEvents?: boolean,
    ignoreChangeEvents?: boolean,
    ignoreDeleteEvents?: boolean
  ): FileSystemWatcher {
    const watcherId = `fsWatcher_${Date.now()}_${Math.random()}`;

    const watcher = new FileSystemWatcherImpl(
      this.bridge,
      watcherId,
      globPattern,
      ignoreCreateEvents || false,
      ignoreChangeEvents || false,
      ignoreDeleteEvents || false
    );

    this.bridge.send('createFileSystemWatcher', {
      watcherId,
      globPattern,
      ignoreCreateEvents,
      ignoreChangeEvents,
      ignoreDeleteEvents
    });

    return watcher;
  }
}
