/**
 * UI Components API
 *
 * StatusBar, TreeView, Webview, and other UI components
 */

import { ExtensionHostBridge } from '../bridge';
import { Disposable, Event, EventEmitter } from './events';

// StatusBar
export enum StatusBarAlignment {
  Left = 1,
  Right = 2
}

export interface StatusBarItem {
  readonly alignment: StatusBarAlignment;
  readonly priority?: number;
  text: string;
  tooltip?: string | { value: string };
  color?: string;
  backgroundColor?: any;
  command?: string | { title: string; command: string; arguments?: any[] };
  accessibilityInformation?: { label: string; role?: string };
  name?: string;
  show(): void;
  hide(): void;
  dispose(): void;
}

class StatusBarItemImpl implements StatusBarItem {
  private _text = '';
  private _tooltip?: string | { value: string };
  private _color?: string;
  private _command?: string | { title: string; command: string; arguments?: any[] };
  private _visible = false;

  constructor(
    private bridge: ExtensionHostBridge,
    public readonly alignment: StatusBarAlignment,
    public readonly priority?: number,
    private id?: string
  ) {
    this.id = id || `statusbar_${Date.now()}_${Math.random()}`;
  }

  get text(): string {
    return this._text;
  }

  set text(value: string) {
    this._text = value;
    this.update();
  }

  get tooltip(): string | { value: string } | undefined {
    return this._tooltip;
  }

  set tooltip(value: string | { value: string } | undefined) {
    this._tooltip = value;
    this.update();
  }

  get color(): string | undefined {
    return this._color;
  }

  set color(value: string | undefined) {
    this._color = value;
    this.update();
  }

  get command(): string | { title: string; command: string; arguments?: any[] } | undefined {
    return this._command;
  }

  set command(value: string | { title: string; command: string; arguments?: any[] } | undefined) {
    this._command = value;
    this.update();
  }

  show(): void {
    this._visible = true;
    this.update();
  }

  hide(): void {
    this._visible = false;
    this.bridge.send('hideStatusBarItem', { id: this.id });
  }

  dispose(): void {
    this.bridge.send('disposeStatusBarItem', { id: this.id });
  }

  private update(): void {
    if (this._visible) {
      this.bridge.send('updateStatusBarItem', {
        id: this.id,
        alignment: this.alignment,
        priority: this.priority,
        text: this._text,
        tooltip: this._tooltip,
        color: this._color,
        command: this._command
      });
    }
  }
}

// TreeView
export interface TreeItem {
  label?: string | { label: string };
  id?: string;
  iconPath?: string | { light: string; dark: string } | any;
  description?: string | boolean;
  resourceUri?: any;
  tooltip?: string | { value: string };
  command?: { title: string; command: string; arguments?: any[] };
  collapsibleState?: TreeItemCollapsibleState;
  contextValue?: string;
  accessibilityInformation?: { label: string; role?: string };
}

export enum TreeItemCollapsibleState {
  None = 0,
  Collapsed = 1,
  Expanded = 2
}

export interface TreeDataProvider<T> {
  onDidChangeTreeData?: Event<T | undefined | null | void>;
  getTreeItem(element: T): TreeItem | Promise<TreeItem>;
  getChildren(element?: T): T[] | Promise<T[]>;
  getParent?(element: T): T | undefined | Promise<T | undefined>;
  resolveTreeItem?(item: TreeItem, element: T): TreeItem | Promise<TreeItem>;
}

export interface TreeView<T> extends Disposable {
  readonly onDidExpandElement: Event<{ element: T }>;
  readonly onDidCollapseElement: Event<{ element: T }>;
  readonly selection: T[];
  readonly onDidChangeSelection: Event<{ selection: T[] }>;
  readonly visible: boolean;
  readonly onDidChangeVisibility: Event<{ visible: boolean }>;
  readonly message?: string;
  readonly title?: string;
  readonly description?: string;
  reveal(element: T, options?: { select?: boolean; focus?: boolean; expand?: boolean | number }): Promise<void>;
  dispose(): void;
}

class TreeViewImpl<T> implements TreeView<T> {
  private _onDidExpandElement = new EventEmitter<{ element: T }>();
  private _onDidCollapseElement = new EventEmitter<{ element: T }>();
  private _onDidChangeSelection = new EventEmitter<{ selection: T[] }>();
  private _onDidChangeVisibility = new EventEmitter<{ visible: boolean }>();
  private _selection: T[] = [];
  private _visible = true;

  readonly onDidExpandElement = this._onDidExpandElement.event;
  readonly onDidCollapseElement = this._onDidCollapseElement.event;
  readonly onDidChangeSelection = this._onDidChangeSelection.event;
  readonly onDidChangeVisibility = this._onDidChangeVisibility.event;

  message?: string;
  title?: string;
  description?: string;

  constructor(
    private bridge: ExtensionHostBridge,
    private viewId: string,
    private dataProvider: TreeDataProvider<T>
  ) {
    this.registerProvider();
  }

  get selection(): T[] {
    return this._selection;
  }

  get visible(): boolean {
    return this._visible;
  }

  private registerProvider(): void {
    this.bridge.send('registerTreeDataProvider', {
      viewId: this.viewId
    });

    this.bridge.on(`treeView:${this.viewId}:getChildren`, async (data: any) => {
      const children = await this.dataProvider.getChildren(data.element);
      this.bridge.send('treeViewChildrenResult', {
        requestId: data.requestId,
        children
      });
    });

    this.bridge.on(`treeView:${this.viewId}:getTreeItem`, async (data: any) => {
      const item = await this.dataProvider.getTreeItem(data.element);
      this.bridge.send('treeViewItemResult', {
        requestId: data.requestId,
        item
      });
    });
  }

  async reveal(element: T, options?: { select?: boolean; focus?: boolean; expand?: boolean | number }): Promise<void> {
    this.bridge.send('revealTreeItem', {
      viewId: this.viewId,
      element,
      options
    });
  }

  dispose(): void {
    this._onDidExpandElement.dispose();
    this._onDidCollapseElement.dispose();
    this._onDidChangeSelection.dispose();
    this._onDidChangeVisibility.dispose();
    this.bridge.send('disposeTreeView', { viewId: this.viewId });
  }
}

// Webview
export interface Webview {
  html: string;
  options: WebviewOptions;
  readonly onDidReceiveMessage: Event<any>;
  postMessage(message: any): Promise<boolean>;
  asWebviewUri(localResource: any): any;
  readonly cspSource: string;
}

export interface WebviewOptions {
  enableScripts?: boolean;
  enableForms?: boolean;
  enableCommandUris?: boolean;
  localResourceRoots?: any[];
  portMapping?: Array<{ webviewPort: number; extensionHostPort: number }>;
}

export interface WebviewPanel extends Disposable {
  readonly webview: Webview;
  readonly viewType: string;
  title: string;
  iconPath?: any;
  readonly options: WebviewPanelOptions;
  readonly viewColumn?: number;
  readonly active: boolean;
  readonly visible: boolean;
  readonly onDidChangeViewState: Event<{ webviewPanel: WebviewPanel }>;
  readonly onDidDispose: Event<void>;
  reveal(viewColumn?: number, preserveFocus?: boolean): void;
  dispose(): void;
}

export interface WebviewPanelOptions {
  enableFindWidget?: boolean;
  retainContextWhenHidden?: boolean;
}

class WebviewImpl implements Webview {
  private _html = '';
  private _onDidReceiveMessage = new EventEmitter<any>();

  readonly onDidReceiveMessage = this._onDidReceiveMessage.event;
  readonly cspSource = 'vscode-resource:';

  constructor(
    private bridge: ExtensionHostBridge,
    private panelId: string,
    public options: WebviewOptions
  ) {}

  get html(): string {
    return this._html;
  }

  set html(value: string) {
    this._html = value;
    this.bridge.send('updateWebviewHtml', {
      panelId: this.panelId,
      html: value
    });
  }

  async postMessage(message: any): Promise<boolean> {
    this.bridge.send('webviewPostMessage', {
      panelId: this.panelId,
      message
    });
    return true;
  }

  asWebviewUri(localResource: any): any {
    return { scheme: 'vscode-resource', path: localResource.path };
  }
}

class WebviewPanelImpl implements WebviewPanel {
  private _webview: WebviewImpl;
  private _title = '';
  private _active = true;
  private _visible = true;
  private _onDidChangeViewState = new EventEmitter<{ webviewPanel: WebviewPanel }>();
  private _onDidDispose = new EventEmitter<void>();

  readonly onDidChangeViewState = this._onDidChangeViewState.event;
  readonly onDidDispose = this._onDidDispose.event;

  constructor(
    private bridge: ExtensionHostBridge,
    private panelId: string,
    public readonly viewType: string,
    title: string,
    public readonly viewColumn: number | undefined,
    public readonly options: WebviewPanelOptions,
    webviewOptions: WebviewOptions
  ) {
    this._title = title;
    this._webview = new WebviewImpl(bridge, panelId, webviewOptions);
  }

  get webview(): Webview {
    return this._webview;
  }

  get title(): string {
    return this._title;
  }

  set title(value: string) {
    this._title = value;
    this.bridge.send('updateWebviewTitle', {
      panelId: this.panelId,
      title: value
    });
  }

  get active(): boolean {
    return this._active;
  }

  get visible(): boolean {
    return this._visible;
  }

  reveal(viewColumn?: number, preserveFocus?: boolean): void {
    this.bridge.send('revealWebview', {
      panelId: this.panelId,
      viewColumn,
      preserveFocus
    });
  }

  dispose(): void {
    this._onDidDispose.fire();
    this._onDidChangeViewState.dispose();
    this._onDidDispose.dispose();
    this.bridge.send('disposeWebview', { panelId: this.panelId });
  }
}

// UI API
export class UIAPI {
  constructor(private bridge: ExtensionHostBridge) {}

  createStatusBarItem(alignment?: StatusBarAlignment, priority?: number): StatusBarItem;
  createStatusBarItem(id: string, alignment?: StatusBarAlignment, priority?: number): StatusBarItem;
  createStatusBarItem(
    alignmentOrId?: StatusBarAlignment | string,
    alignmentOrPriority?: StatusBarAlignment | number,
    priority?: number
  ): StatusBarItem {
    if (typeof alignmentOrId === 'string') {
      return new StatusBarItemImpl(
        this.bridge,
        alignmentOrPriority as StatusBarAlignment || StatusBarAlignment.Left,
        priority,
        alignmentOrId
      );
    }
    return new StatusBarItemImpl(
      this.bridge,
      alignmentOrId || StatusBarAlignment.Left,
      alignmentOrPriority as number
    );
  }

  createTreeView<T>(viewId: string, options: { treeDataProvider: TreeDataProvider<T> }): TreeView<T> {
    return new TreeViewImpl(this.bridge, viewId, options.treeDataProvider);
  }

  createWebviewPanel(
    viewType: string,
    title: string,
    showOptions: number | { viewColumn: number; preserveFocus?: boolean },
    options?: WebviewPanelOptions & { enableScripts?: boolean; localResourceRoots?: any[] }
  ): WebviewPanel {
    const panelId = `webview_${Date.now()}_${Math.random()}`;
    const viewColumn = typeof showOptions === 'number' ? showOptions : showOptions.viewColumn;

    const panel = new WebviewPanelImpl(
      this.bridge,
      panelId,
      viewType,
      title,
      viewColumn,
      options || {},
      {
        enableScripts: options?.enableScripts,
        localResourceRoots: options?.localResourceRoots
      }
    );

    this.bridge.send('createWebviewPanel', {
      panelId,
      viewType,
      title,
      viewColumn,
      options
    });

    return panel;
  }

  registerWebviewPanelSerializer(viewType: string, serializer: any): Disposable {
    this.bridge.send('registerWebviewSerializer', { viewType });
    return {
      dispose: () => {
        this.bridge.send('unregisterWebviewSerializer', { viewType });
      }
    };
  }
}
