/**
 * Source Control Management (SCM) API
 *
 * Allows extensions to integrate with source control systems like Git
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';

export interface SourceControlResourceState {
  readonly resourceUri: any;
  readonly command?: { title: string; command: string; arguments?: any[] };
  readonly decorations?: SourceControlResourceDecorations;
}

export interface SourceControlResourceDecorations {
  readonly strikeThrough?: boolean;
  readonly faded?: boolean;
  readonly tooltip?: string;
  readonly light?: { iconPath: string };
  readonly dark?: { iconPath: string };
}

export interface SourceControlResourceGroup {
  readonly id: string;
  label: string;
  hideWhenEmpty?: boolean;
  resourceStates: SourceControlResourceState[];
  dispose(): void;
}

class SourceControlResourceGroupImpl implements SourceControlResourceGroup {
  private _label: string;
  private _hideWhenEmpty?: boolean;
  private _resourceStates: SourceControlResourceState[] = [];

  constructor(
    private bridge: ExtensionHostBridge,
    private scmId: string,
    public readonly id: string,
    label: string
  ) {
    this._label = label;
  }

  get label(): string {
    return this._label;
  }

  set label(value: string) {
    this._label = value;
    this.update();
  }

  get hideWhenEmpty(): boolean | undefined {
    return this._hideWhenEmpty;
  }

  set hideWhenEmpty(value: boolean | undefined) {
    this._hideWhenEmpty = value;
    this.update();
  }

  get resourceStates(): SourceControlResourceState[] {
    return this._resourceStates;
  }

  set resourceStates(value: SourceControlResourceState[]) {
    this._resourceStates = value;
    this.update();
  }

  private update(): void {
    this.bridge.send('updateSCMResourceGroup', {
      scmId: this.scmId,
      groupId: this.id,
      label: this._label,
      hideWhenEmpty: this._hideWhenEmpty,
      resourceStates: this._resourceStates
    });
  }

  dispose(): void {
    this.bridge.send('disposeSCMResourceGroup', {
      scmId: this.scmId,
      groupId: this.id
    });
  }
}

export class SourceControl {
  readonly id!: string;
  readonly label!: string;
  readonly rootUri!: any | undefined;
  get inputBox(): SourceControlInputBox { throw new Error('Not implemented'); }
  get count(): number | undefined { return undefined; }
  set count(value: number | undefined) {}
  get quickDiffProvider(): QuickDiffProvider | undefined { return undefined; }
  set quickDiffProvider(value: QuickDiffProvider | undefined) {}
  get commitTemplate(): string | undefined { return undefined; }
  set commitTemplate(value: string | undefined) {}
  get acceptInputCommand(): { title: string; command: string; arguments?: any[] } | undefined { return undefined; }
  set acceptInputCommand(value: { title: string; command: string; arguments?: any[] } | undefined) {}
  get statusBarCommands(): { title: string; command: string; arguments?: any[] }[] | undefined { return undefined; }
  set statusBarCommands(value: { title: string; command: string; arguments?: any[] }[] | undefined) {}
  createResourceGroup(id: string, label: string): SourceControlResourceGroup { throw new Error('Not implemented'); }
  dispose(): void {}
}

export interface SourceControlInputBox {
  value: string;
  placeholder?: string;
  enabled: boolean;
  visible: boolean;
  readonly onDidChange: Event<string>;
}

class SourceControlInputBoxImpl implements SourceControlInputBox {
  private _value = '';
  private _placeholder?: string;
  private _enabled = true;
  private _visible = true;
  private _onDidChange = new EventEmitter<string>();

  readonly onDidChange = this._onDidChange.event;

  constructor(
    private bridge: ExtensionHostBridge,
    private scmId: string
  ) {
    this.setupListeners();
  }

  private setupListeners(): void {
    this.bridge.on(`scm:${this.scmId}:inputChange`, (data: any) => {
      this._value = data.value;
      this._onDidChange.fire(data.value);
    });
  }

  get value(): string {
    return this._value;
  }

  set value(val: string) {
    this._value = val;
    this.bridge.send('updateSCMInputBox', {
      scmId: this.scmId,
      value: val
    });
  }

  get placeholder(): string | undefined {
    return this._placeholder;
  }

  set placeholder(val: string | undefined) {
    this._placeholder = val;
    this.bridge.send('updateSCMInputBox', {
      scmId: this.scmId,
      placeholder: val
    });
  }

  get enabled(): boolean {
    return this._enabled;
  }

  set enabled(val: boolean) {
    this._enabled = val;
    this.bridge.send('updateSCMInputBox', {
      scmId: this.scmId,
      enabled: val
    });
  }

  get visible(): boolean {
    return this._visible;
  }

  set visible(val: boolean) {
    this._visible = val;
    this.bridge.send('updateSCMInputBox', {
      scmId: this.scmId,
      visible: val
    });
  }

  dispose(): void {
    this._onDidChange.dispose();
  }
}

export interface QuickDiffProvider {
  provideOriginalResource?(uri: any, token?: any): any | null | undefined | Promise<any | null | undefined>;
}

class SourceControlImpl extends SourceControl {
  private _inputBox: SourceControlInputBoxImpl;
  private _count?: number;
  private _quickDiffProvider?: QuickDiffProvider;
  private _commitTemplate?: string;
  private _acceptInputCommand?: { title: string; command: string; arguments?: any[] };
  private _statusBarCommands?: { title: string; command: string; arguments?: any[] }[];
  private _resourceGroups: SourceControlResourceGroup[] = [];

  constructor(
    private bridge: ExtensionHostBridge,
    id: string,
    label: string,
    rootUri: any | undefined
  ) {
    super();
    (this as any).id = id;
    (this as any).label = label;
    (this as any).rootUri = rootUri;
    this._inputBox = new SourceControlInputBoxImpl(bridge, id);
  }

  get inputBox(): SourceControlInputBox {
    return this._inputBox;
  }

  get count(): number | undefined {
    return this._count;
  }

  set count(value: number | undefined) {
    this._count = value;
    this.update();
  }

  get quickDiffProvider(): QuickDiffProvider | undefined {
    return this._quickDiffProvider;
  }

  set quickDiffProvider(value: QuickDiffProvider | undefined) {
    this._quickDiffProvider = value;
  }

  get commitTemplate(): string | undefined {
    return this._commitTemplate;
  }

  set commitTemplate(value: string | undefined) {
    this._commitTemplate = value;
    this.update();
  }

  get acceptInputCommand(): { title: string; command: string; arguments?: any[] } | undefined {
    return this._acceptInputCommand;
  }

  set acceptInputCommand(value: { title: string; command: string; arguments?: any[] } | undefined) {
    this._acceptInputCommand = value;
    this.update();
  }

  get statusBarCommands(): { title: string; command: string; arguments?: any[] }[] | undefined {
    return this._statusBarCommands;
  }

  set statusBarCommands(value: { title: string; command: string; arguments?: any[] }[] | undefined) {
    this._statusBarCommands = value;
    this.update();
  }

  createResourceGroup(id: string, label: string): SourceControlResourceGroup {
    const group = new SourceControlResourceGroupImpl(this.bridge, this.id, id, label);
    this._resourceGroups.push(group);
    return group;
  }

  private update(): void {
    this.bridge.send('updateSourceControl', {
      id: this.id,
      label: this.label,
      count: this._count,
      commitTemplate: this._commitTemplate,
      acceptInputCommand: this._acceptInputCommand,
      statusBarCommands: this._statusBarCommands
    });
  }

  dispose(): void {
    for (const group of this._resourceGroups) {
      group.dispose();
    }
    this._inputBox.dispose();
    this.bridge.send('disposeSourceControl', { id: this.id });
  }
}

export class SCMAPI {
  private _onDidChangeActiveProvider = new EventEmitter<SourceControl | undefined>();
  private _inputBoxes: SourceControlInputBoxImpl[] = [];

  readonly onDidChangeActiveProvider = this._onDidChangeActiveProvider.event;

  constructor(private bridge: ExtensionHostBridge) {}

  createSourceControl(
    id: string,
    label: string,
    rootUri?: any
  ): SourceControl {
    const scm = new SourceControlImpl(this.bridge, id, label, rootUri);

    this.bridge.send('createSourceControl', {
      id,
      label,
      rootUri
    });

    return scm;
  }
}
