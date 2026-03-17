/**
 * Progress and QuickInput APIs
 *
 * Progress notifications and advanced input controls
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';
import { ProgressOptions, Progress, ProgressLocation, QuickPickItem, QuickInputButton, InputBoxOptions, QuickPickOptions, CancellationToken } from './common';

// Re-export types from common for convenience
export type { QuickPickItem, QuickInputButton, ProgressOptions, Progress, ProgressLocation, InputBoxOptions, QuickPickOptions };

// QuickPick
export interface QuickPick<T extends QuickPickItem> extends Disposable {
  value: string;
  placeholder: string | undefined;
  readonly onDidChangeValue: Event<string>;
  readonly onDidAccept: Event<void>;
  readonly onDidHide: Event<void>;
  buttons: readonly QuickInputButton[];
  readonly onDidTriggerButton: Event<QuickInputButton>;
  items: readonly T[];
  canSelectMany: boolean;
  matchOnDescription: boolean;
  matchOnDetail: boolean;
  activeItems: readonly T[];
  readonly onDidChangeActive: Event<readonly T[]>;
  selectedItems: readonly T[];
  readonly onDidChangeSelection: Event<readonly T[]>;
  title: string | undefined;
  step: number | undefined;
  totalSteps: number | undefined;
  enabled: boolean;
  busy: boolean;
  ignoreFocusOut: boolean;
  show(): void;
  hide(): void;
  dispose(): void;
}

class QuickPickImpl<T extends QuickPickItem> implements QuickPick<T> {
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

  readonly onDidChangeValue = this._onDidChangeValue.event;
  readonly onDidAccept = this._onDidAccept.event;
  readonly onDidHide = this._onDidHide.event;
  readonly onDidTriggerButton = this._onDidTriggerButton.event;
  readonly onDidChangeActive = this._onDidChangeActive.event;
  readonly onDidChangeSelection = this._onDidChangeSelection.event;

  constructor(private bridge: ExtensionHostBridge) {}

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
    this.bridge.send('showQuickPick', {
      value: this._value,
      placeholder: this._placeholder,
      items: this._items,
      title: this._title,
      canSelectMany: this._canSelectMany
    });
  }

  hide(): void {
    this._onDidHide.fire();
  }

  dispose(): void {
    this._onDidChangeValue.dispose();
    this._onDidAccept.dispose();
    this._onDidHide.dispose();
    this._onDidTriggerButton.dispose();
    this._onDidChangeActive.dispose();
    this._onDidChangeSelection.dispose();
  }
}

// InputBox
export interface InputBox extends Disposable {
  value: string;
  placeholder: string | undefined;
  password: boolean;
  readonly onDidChangeValue: Event<string>;
  readonly onDidAccept: Event<void>;
  readonly onDidHide: Event<void>;
  buttons: readonly QuickInputButton[];
  readonly onDidTriggerButton: Event<QuickInputButton>;
  prompt: string | undefined;
  validationMessage: string | undefined;
  title: string | undefined;
  step: number | undefined;
  totalSteps: number | undefined;
  enabled: boolean;
  busy: boolean;
  ignoreFocusOut: boolean;
  show(): void;
  hide(): void;
  dispose(): void;
}

class InputBoxImpl implements InputBox {
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

  constructor(private bridge: ExtensionHostBridge) {}

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
    this.bridge.send('showInputBox', {
      value: this._value,
      placeholder: this._placeholder,
      password: this._password,
      prompt: this._prompt,
      title: this._title
    });
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
