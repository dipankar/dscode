/**
 * UI Components API
 *
 * StatusBar, TreeView, Webview, and other UI components
 */

import { ExtensionHostBridge } from '../bridge';
import { Disposable, Event, EventEmitter } from './events';
import { Uri } from './uri';
import { ThemeColor, ThemeIcon } from './common';

type Command = { title: string; command: string; arguments?: unknown[] };
type ThemeIconPath = Uri | { light: Uri; dark: Uri } | ThemeIcon;

// StatusBar
export enum StatusBarAlignment {
  Left = 1,
  Right = 2,
}

export class StatusBarItem {
  readonly alignment!: StatusBarAlignment;
  readonly priority?: number;
  get text(): string {
    return '';
  }
  set text(value: string) {}
  get tooltip(): string | { value: string } | undefined {
    return undefined;
  }
  set tooltip(value: string | { value: string } | undefined) {}
  get color(): string | undefined {
    return undefined;
  }
  set color(value: string | undefined) {}
  get command(): string | Command | undefined {
    return undefined;
  }
  set command(value: string | Command | undefined) {}
  backgroundColor?: ThemeColor;
  accessibilityInformation?: { label: string; role?: string };
  name?: string;
  show(): void {}
  hide(): void {}
  dispose(): void {}
}

class StatusBarItemImpl extends StatusBarItem {
  private _text = '';
  private _tooltip?: string | { value: string };
  private _color?: string;
  private _command?: string | Command;
  private _visible = false;
  readonly alignment: StatusBarAlignment;
  readonly priority?: number;

  constructor(
    private bridge: ExtensionHostBridge,
    private owner: string,
    alignment: StatusBarAlignment,
    priority?: number,
    private id?: string
  ) {
    super();
    this.alignment = alignment;
    this.priority = priority;
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

  get command(): string | Command | undefined {
    return this._command;
  }

  set command(value: string | Command | undefined) {
    this._command = value;
    this.update();
  }

  show(): void {
    this._visible = true;
    this.update();
  }

  hide(): void {
    this._visible = false;
    this.bridge.send('hideStatusBarItem', { id: this.id, owner: this.owner, visible: false });
  }

  dispose(): void {
    this.bridge.send('disposeStatusBarItem', { id: this.id, owner: this.owner });
  }

  private update(): void {
    if (this._visible) {
      this.bridge.send('updateStatusBarItem', {
        id: this.id,
        owner: this.owner,
        alignment: this.alignment,
        priority: this.priority,
        text: this._text,
        tooltip: this.serializeTooltip(),
        color: this._color,
        command: this._command,
        visible: true,
      });
    }
  }

  private serializeTooltip(): string | undefined {
    if (typeof this._tooltip === 'string') {
      return this._tooltip;
    }

    if (this._tooltip && typeof this._tooltip === 'object') {
      return this._tooltip.value;
    }

    return undefined;
  }
}

// TreeView
export class TreeItem {
  constructor(
    public label?: string | { label: string },
    public collapsibleState?: TreeItemCollapsibleState
  ) {}

  id?: string;
  iconPath?: string | { light: string; dark: string } | ThemeIconPath;
  description?: string | boolean;
  resourceUri?: Uri;
  tooltip?: string | { value: string };
  command?: Command;
  contextValue?: string;
  accessibilityInformation?: { label: string; role?: string };
}

export enum TreeItemCollapsibleState {
  None = 0,
  Collapsed = 1,
  Expanded = 2,
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
  reveal(
    element: T,
    options?: { select?: boolean; focus?: boolean; expand?: boolean | number }
  ): Promise<void>;
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
    private dataProvider: TreeDataProvider<T>,
    private owner: string
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
      viewId: this.viewId,
      owner: this.owner,
    });
  }

  async reveal(
    element: T,
    options?: { select?: boolean; focus?: boolean; expand?: boolean | number }
  ): Promise<void> {
    this.bridge.send('revealTreeItem', {
      viewId: this.viewId,
      element,
      options,
    });
  }

  dispose(): void {
    this._onDidExpandElement.dispose();
    this._onDidCollapseElement.dispose();
    this._onDidChangeSelection.dispose();
    this._onDidChangeVisibility.dispose();
    this.bridge.send('disposeTreeView', { viewId: this.viewId });
  }

  handleHostEvent(event: string, payload: Record<string, unknown>): void {
    switch (event) {
      case 'didExpand':
        this._onDidExpandElement.fire({ element: payload.element as T });
        break;
      case 'didCollapse':
        this._onDidCollapseElement.fire({ element: payload.element as T });
        break;
      case 'didChangeSelection':
        this._selection = Array.isArray(payload.selection) ? (payload.selection as T[]) : [];
        this._onDidChangeSelection.fire({ selection: this._selection });
        break;
      case 'didChangeVisibility':
        this._visible = !!payload.visible;
        this._onDidChangeVisibility.fire({ visible: this._visible });
        break;
      default:
        break;
    }
  }
}

// Webview
export interface Webview {
  html: string;
  options: WebviewOptions;
  readonly onDidReceiveMessage: Event<unknown>;
  postMessage(message: unknown): Promise<boolean>;
  asWebviewUri(localResource: Uri): Uri;
  readonly cspSource: string;
}

export interface WebviewOptions {
  enableScripts?: boolean;
  enableForms?: boolean;
  enableCommandUris?: boolean;
  localResourceRoots?: Uri[];
  portMapping?: Array<{ webviewPort: number; extensionHostPort: number }>;
}

export interface WebviewPanel extends Disposable {
  readonly webview: Webview;
  readonly viewType: string;
  title: string;
  iconPath?: string | { light: string; dark: string } | ThemeIconPath;
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
  private _onDidReceiveMessage = new EventEmitter<unknown>();

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
      html: value,
    });
  }

  async postMessage(message: unknown): Promise<boolean> {
    this.bridge.send('webviewPostMessage', {
      panelId: this.panelId,
      message,
    });
    return true;
  }

  asWebviewUri(localResource: Uri): Uri {
    return Uri.from({ scheme: 'vscode-resource', path: localResource.path });
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
      title: value,
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
      preserveFocus,
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
type TreeProviderEntry<T = unknown> = {
  provider: TreeDataProvider<T>;
  owner: string;
  view: TreeViewImpl<T>;
};

export class UIAPI {
  private currentExtensionId = '__core__';
  private treeProviders = new Map<string, TreeProviderEntry>();

  constructor(private bridge: ExtensionHostBridge) {
    this.bridge.on(
      'treeView:getChildren',
      async (payload: Record<string, unknown>, respond: (response: unknown) => void) => {
        respond(await this.handleTreeGetChildren(payload));
      }
    );

    this.bridge.on(
      'treeView:getTreeItem',
      async (payload: Record<string, unknown>, respond: (response: unknown) => void) => {
        respond(await this.handleTreeGetItem(payload));
      }
    );

    this.bridge.on(
      'treeView:event',
      async (payload: Record<string, unknown>, respond: (response: unknown) => void) => {
        respond(await this.handleTreeEvent(payload));
      }
    );
  }

  async runWithExtension<T>(extensionId: string, callback: () => Promise<T>): Promise<T> {
    const previous = this.currentExtensionId;
    this.currentExtensionId = extensionId || '__core__';
    try {
      return await callback();
    } finally {
      this.currentExtensionId = previous;
    }
  }

  private currentOwner(): string {
    return this.currentExtensionId || '__core__';
  }

  private async handleTreeGetChildren(payload: Record<string, unknown>) {
    const entry = this.treeProviders.get(payload.viewId as string);
    if (!entry) {
      return { success: false, error: `Tree view ${payload.viewId} not registered` };
    }

    try {
      const children = await this.runWithExtension(entry.owner, async () =>
        entry.provider.getChildren(payload.element)
      );
      return { success: true, children: children ?? [] };
    } catch (error: unknown) {
      return { success: false, error: error instanceof Error ? error.message : String(error) };
    }
  }

  private async handleTreeGetItem(payload: Record<string, unknown>) {
    const entry = this.treeProviders.get(payload.viewId as string);
    if (!entry) {
      return { success: false, error: `Tree view ${payload.viewId} not registered` };
    }

    try {
      const item = await this.runWithExtension(entry.owner, async () =>
        entry.provider.getTreeItem(payload.element)
      );
      return { success: true, item };
    } catch (error: unknown) {
      return { success: false, error: error instanceof Error ? error.message : String(error) };
    }
  }

  private async handleTreeEvent(payload: Record<string, unknown>) {
    const entry = this.treeProviders.get(payload.viewId as string);
    if (!entry) {
      return { success: false, error: `Tree view ${payload.viewId} not registered` };
    }

    try {
      await this.runWithExtension(entry.owner, async () => {
        entry.view.handleHostEvent(payload.event as string, payload);
      });
      return { success: true };
    } catch (error: unknown) {
      return { success: false, error: error instanceof Error ? error.message : String(error) };
    }
  }

  createStatusBarItem(alignment?: StatusBarAlignment, priority?: number): StatusBarItem;
  createStatusBarItem(id: string, alignment?: StatusBarAlignment, priority?: number): StatusBarItem;
  createStatusBarItem(
    alignmentOrId?: StatusBarAlignment | string,
    alignmentOrPriority?: StatusBarAlignment | number,
    priority?: number
  ): StatusBarItem {
    const owner = this.currentOwner();

    if (typeof alignmentOrId === 'string') {
      return new StatusBarItemImpl(
        this.bridge,
        owner,
        (alignmentOrPriority as StatusBarAlignment) || StatusBarAlignment.Left,
        priority,
        alignmentOrId
      );
    }
    return new StatusBarItemImpl(
      this.bridge,
      owner,
      alignmentOrId || StatusBarAlignment.Left,
      alignmentOrPriority as number
    );
  }

  createTreeView<T>(
    viewId: string,
    options: { treeDataProvider: TreeDataProvider<T> }
  ): TreeView<T> {
    const owner = this.currentOwner();
    const treeView = new TreeViewImpl<T>(this.bridge, viewId, options.treeDataProvider, owner);
    this.treeProviders.set(viewId, {
      provider: options.treeDataProvider,
      owner,
      view: treeView,
    } as TreeProviderEntry);

    const originalDispose = treeView.dispose.bind(treeView);
    treeView.dispose = () => {
      this.treeProviders.delete(viewId);
      originalDispose();
    };

    return treeView;
  }

  createWebviewPanel(
    viewType: string,
    title: string,
    showOptions: number | { viewColumn: number; preserveFocus?: boolean },
    options?: WebviewPanelOptions & { enableScripts?: boolean; localResourceRoots?: Uri[] }
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
        localResourceRoots: options?.localResourceRoots,
      }
    );

    this.bridge.send('createWebviewPanel', {
      panelId,
      viewType,
      title,
      viewColumn,
      options,
    });

    return panel;
  }

  registerWebviewPanelSerializer(viewType: string, serializer: any): Disposable {
    this.bridge.send('registerWebviewSerializer', { viewType });
    return {
      dispose: () => {
        this.bridge.send('unregisterWebviewSerializer', { viewType });
      },
    };
  }
}
