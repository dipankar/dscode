/**
 * Progress and QuickInput APIs
 *
 * Progress notifications and advanced input controls
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';
import { ProgressOptions, Progress, ProgressLocation, QuickPickItem, QuickInputButton, InputBoxOptions, QuickPickOptions, CancellationToken } from './common';

// Re-export types and classes from common for convenience
export { QuickPickItem, QuickInputButton, ProgressOptions, Progress, ProgressLocation, InputBoxOptions, QuickPickOptions };

// QuickPick
export class QuickPick<T extends QuickPickItem> extends Disposable {
  get value(): string { return ''; }
  set value(val: string) {}
  get placeholder(): string | undefined { return undefined; }
  set placeholder(val: string | undefined) {}
  readonly onDidChangeValue!: Event<string>;
  readonly onDidAccept!: Event<void>;
  readonly onDidHide!: Event<void>;
  get buttons(): readonly QuickInputButton[] { return []; }
  set buttons(val: readonly QuickInputButton[]) {}
  readonly onDidTriggerButton!: Event<QuickInputButton>;
  get items(): readonly T[] { return []; }
  set items(val: readonly T[]) {}
  get canSelectMany(): boolean { return false; }
  set canSelectMany(val: boolean) {}
  get matchOnDescription(): boolean { return false; }
  set matchOnDescription(val: boolean) {}
  get matchOnDetail(): boolean { return false; }
  set matchOnDetail(val: boolean) {}
  get activeItems(): readonly T[] { return []; }
  set activeItems(val: readonly T[]) {}
  readonly onDidChangeActive!: Event<readonly T[]>;
  get selectedItems(): readonly T[] { return []; }
  set selectedItems(val: readonly T[]) {}
  readonly onDidChangeSelection!: Event<readonly T[]>;
  get title(): string | undefined { return undefined; }
  set title(val: string | undefined) {}
  get step(): number | undefined { return undefined; }
  set step(val: number | undefined) {}
  get totalSteps(): number | undefined { return undefined; }
  set totalSteps(val: number | undefined) {}
  get enabled(): boolean { return true; }
  set enabled(val: boolean) {}
  get busy(): boolean { return false; }
  set busy(val: boolean) {}
  get ignoreFocusOut(): boolean { return false; }
  set ignoreFocusOut(val: boolean) {}
  show(): void {}
  hide(): void {}
}

class QuickPickImpl<T extends QuickPickItem> extends QuickPick<T> {
  private _value = '';
  private _placeholder?: string;
  private _items: readonly T[] = [];
  private _activeItems: readonly T[] = [];
  private _selectedItems: readonly T[] = [];
  private _title?: string;
  private _step?: number;
  private _totalSteps?: number;
  private _enabled = true;
  private _busy = false;
  private _ignoreFocusOut = false;
  private _canSelectMany = false;
  private _matchOnDescription = false;
  private _matchOnDetail = false;
  private _buttons: readonly QuickInputButton[] = [];

  private _onDidChangeValue = new EventEmitter<string>();
  private _onDidAccept = new EventEmitter<void>();
  private _onDidHide = new EventEmitter<void>();
  private _onDidTriggerButton = new EventEmitter<QuickInputButton>();
  private _onDidChangeActive = new EventEmitter<readonly T[]>();
  private _onDidChangeSelection = new EventEmitter<readonly T[]>();
  private _visible = false;

  readonly onDidChangeValue = this._onDidChangeValue.event;
  readonly onDidAccept = this._onDidAccept.event;
  readonly onDidHide = this._onDidHide.event;
  readonly onDidTriggerButton = this._onDidTriggerButton.event;
  readonly onDidChangeActive = this._onDidChangeActive.event;
  readonly onDidChangeSelection = this._onDidChangeSelection.event;

  constructor(private bridge: ExtensionHostBridge) {
    super();
  }

  get value(): string {
    return this._value;
  }

  set value(val: string) {
    this._value = val;
    this._onDidChangeValue.fire(val);
  }

  get placeholder(): string | undefined {
    return this._placeholder;
  }

  set placeholder(val: string | undefined) {
    this._placeholder = val;
  }

  get items(): readonly T[] {
    return this._items;
  }

  set items(val: readonly T[]) {
    this._items = val;
  }

  get activeItems(): readonly T[] {
    return this._activeItems;
  }

  set activeItems(val: readonly T[]) {
    this._activeItems = val;
    this._onDidChangeActive.fire(val);
  }

  get selectedItems(): readonly T[] {
    return this._selectedItems;
  }

  set selectedItems(val: readonly T[]) {
    this._selectedItems = val;
    this._onDidChangeSelection.fire(val);
  }

  get title(): string | undefined {
    return this._title;
  }

  set title(val: string | undefined) {
    this._title = val;
  }

  get step(): number | undefined {
    return this._step;
  }

  set step(val: number | undefined) {
    this._step = val;
  }

  get totalSteps(): number | undefined {
    return this._totalSteps;
  }

  set totalSteps(val: number | undefined) {
    this._totalSteps = val;
  }

  get enabled(): boolean {
    return this._enabled;
  }

  set enabled(val: boolean) {
    this._enabled = val;
  }

  get busy(): boolean {
    return this._busy;
  }

  set busy(val: boolean) {
    this._busy = val;
  }

  get ignoreFocusOut(): boolean {
    return this._ignoreFocusOut;
  }

  set ignoreFocusOut(val: boolean) {
    this._ignoreFocusOut = val;
  }

  get canSelectMany(): boolean {
    return this._canSelectMany;
  }

  set canSelectMany(val: boolean) {
    this._canSelectMany = val;
  }

  get matchOnDescription(): boolean {
    return this._matchOnDescription;
  }

  set matchOnDescription(val: boolean) {
    this._matchOnDescription = val;
  }

  get matchOnDetail(): boolean {
    return this._matchOnDetail;
  }

  set matchOnDetail(val: boolean) {
    this._matchOnDetail = val;
  }

  get buttons(): readonly QuickInputButton[] {
    return this._buttons;
  }

  set buttons(val: readonly QuickInputButton[]) {
    this._buttons = val;
  }

  show(): void {
    if (this._visible) {
      return;
    }
    this._visible = true;
    void this.present();
  }

  hide(): void {
    if (!this._visible) {
      return;
    }
    this._visible = false;
    this._onDidHide.fire();
  }

  dispose(): void {
    this.hide();
    this._onDidChangeValue.dispose();
    this._onDidAccept.dispose();
    this._onDidHide.dispose();
    this._onDidTriggerButton.dispose();
    this._onDidChangeActive.dispose();
    this._onDidChangeSelection.dispose();
  }

  private async present(): Promise<void> {
    try {
      const serializedItems = this._items.map(item => ({
        label: item.label,
        description: item.description,
        detail: (item as any).detail,
        picked: (item as any).picked,
        alwaysShow: (item as any).alwaysShow,
      }));

      const response = await this.bridge.request('window-show-quick-pick', {
        items: serializedItems,
        options: {
          placeHolder: this._placeholder,
          canPickMany: this._canSelectMany,
          matchOnDescription: this._matchOnDescription,
          matchOnDetail: this._matchOnDetail,
          title: this._title,
          ignoreFocusOut: this._ignoreFocusOut,
          value: this._value,
        },
      }) as { selected?: any } | null;

      if (response && response.selected) {
        const selections = Array.isArray(response.selected)
          ? response.selected
          : [response.selected];
        const labels = selections
          .map((entry: any) => (typeof entry === 'string' ? entry : entry?.label))
          .filter(Boolean);
        const matched = this._items.filter(item => labels.includes(item.label));

        if (this._canSelectMany) {
          this.selectedItems = matched;
        } else if (matched.length > 0) {
          this.selectedItems = [matched[0]];
        }

        this._onDidAccept.fire();
      }
    } catch (error) {
      console.error('[QuickPick] Failed to show quick pick:', error);
    } finally {
      this.hide();
    }
  }
}

// InputBox
export class InputBox extends Disposable {
  get value(): string { return ''; }
  set value(val: string) {}
  get placeholder(): string | undefined { return undefined; }
  set placeholder(val: string | undefined) {}
  get password(): boolean { return false; }
  set password(val: boolean) {}
  readonly onDidChangeValue!: Event<string>;
  readonly onDidAccept!: Event<void>;
  readonly onDidHide!: Event<void>;
  get buttons(): readonly QuickInputButton[] { return []; }
  set buttons(val: readonly QuickInputButton[]) {}
  readonly onDidTriggerButton!: Event<QuickInputButton>;
  get prompt(): string | undefined { return undefined; }
  set prompt(val: string | undefined) {}
  get validationMessage(): string | undefined { return undefined; }
  set validationMessage(val: string | undefined) {}
  get title(): string | undefined { return undefined; }
  set title(val: string | undefined) {}
  get step(): number | undefined { return undefined; }
  set step(val: number | undefined) {}
  get totalSteps(): number | undefined { return undefined; }
  set totalSteps(val: number | undefined) {}
  get enabled(): boolean { return true; }
  set enabled(val: boolean) {}
  get busy(): boolean { return false; }
  set busy(val: boolean) {}
  get ignoreFocusOut(): boolean { return false; }
  set ignoreFocusOut(val: boolean) {}
  show(): void {}
  hide(): void {}
}

class InputBoxImpl extends InputBox {
  private _value = '';
  private _placeholder?: string;
  private _password = false;
  private _prompt?: string;
  private _validationMessage?: string;
  private _title?: string;
  private _step?: number;
  private _totalSteps?: number;
  private _enabled = true;
  private _busy = false;
  private _ignoreFocusOut = false;
  private _buttons: readonly QuickInputButton[] = [];

  private _onDidChangeValue = new EventEmitter<string>();
  private _onDidAccept = new EventEmitter<void>();
  private _onDidHide = new EventEmitter<void>();
  private _onDidTriggerButton = new EventEmitter<QuickInputButton>();

  readonly onDidChangeValue = this._onDidChangeValue.event;
  readonly onDidAccept = this._onDidAccept.event;
  readonly onDidHide = this._onDidHide.event;
  readonly onDidTriggerButton = this._onDidTriggerButton.event;

  constructor(private bridge: ExtensionHostBridge) {
    super();
  }

  get value(): string {
    return this._value;
  }

  set value(val: string) {
    this._value = val;
    this._onDidChangeValue.fire(val);
  }

  get placeholder(): string | undefined {
    return this._placeholder;
  }

  set placeholder(val: string | undefined) {
    this._placeholder = val;
  }

  get password(): boolean {
    return this._password;
  }

  set password(val: boolean) {
    this._password = val;
  }

  get prompt(): string | undefined {
    return this._prompt;
  }

  set prompt(val: string | undefined) {
    this._prompt = val;
  }

  get validationMessage(): string | undefined {
    return this._validationMessage;
  }

  set validationMessage(val: string | undefined) {
    this._validationMessage = val;
  }

  get title(): string | undefined {
    return this._title;
  }

  set title(val: string | undefined) {
    this._title = val;
  }

  get step(): number | undefined {
    return this._step;
  }

  set step(val: number | undefined) {
    this._step = val;
  }

  get totalSteps(): number | undefined {
    return this._totalSteps;
  }

  set totalSteps(val: number | undefined) {
    this._totalSteps = val;
  }

  get enabled(): boolean {
    return this._enabled;
  }

  set enabled(val: boolean) {
    this._enabled = val;
  }

  get busy(): boolean {
    return this._busy;
  }

  set busy(val: boolean) {
    this._busy = val;
  }

  get ignoreFocusOut(): boolean {
    return this._ignoreFocusOut;
  }

  set ignoreFocusOut(val: boolean) {
    this._ignoreFocusOut = val;
  }

  get buttons(): readonly QuickInputButton[] {
    return this._buttons;
  }

  set buttons(val: readonly QuickInputButton[]) {
    this._buttons = val;
  }

  show(): void {
    void this.present();
  }

  hide(): void {
    this._onDidHide.fire();
  }

  dispose(): void {
    this._onDidChangeValue.dispose();
    this._onDidAccept.dispose();
    this._onDidHide.dispose();
    this._onDidTriggerButton.dispose();
  }

  private async present(): Promise<void> {
    try {
      const response = await this.bridge.request('window-show-input-box', {
        value: this._value,
        placeHolder: this._placeholder,
        password: this._password,
        prompt: this._prompt,
        title: this._title,
        step: this._step,
        totalSteps: this._totalSteps,
        ignoreFocusOut: this._ignoreFocusOut,
      }) as { value?: string } | null;

      if (response && typeof response.value === 'string') {
        this.value = response.value;
        this._onDidAccept.fire();
      }
    } catch (error) {
      console.error('[InputBox] Failed to show input box:', error);
    } finally {
      this.hide();
    }
  }
}

// Progress API
export class ProgressAPI {
  constructor(private bridge: ExtensionHostBridge) {}

  async withProgress<R>(
    options: ProgressOptions,
    task: (progress: Progress<{ message?: string; increment?: number }>, token: CancellationToken) => Promise<R>
  ): Promise<R> {
    const progressId = `progress_${Date.now()}_${Math.random()}`;

    const progress: Progress<{ message?: string; increment?: number }> = {
      report: (value) => {
        this.bridge.send('reportProgress', {
          progressId,
          ...value
        });
      }
    };

    const token: CancellationToken = {
      isCancellationRequested: false,
      onCancellationRequested: () => ({ dispose: () => {} })
    };

    this.bridge.send('startProgress', {
      progressId,
      location: options.location,
      title: options.title,
      cancellable: options.cancellable
    });

    try {
      const result = await task(progress, token);
      this.bridge.send('endProgress', { progressId });
      return result;
    } catch (error) {
      this.bridge.send('endProgress', { progressId });
      throw error;
    }
  }

  createQuickPick<T extends QuickPickItem>(): QuickPick<T> {
    return new QuickPickImpl<T>(this.bridge);
  }

  createInputBox(): InputBox {
    return new InputBoxImpl(this.bridge);
  }
}
