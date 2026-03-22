/**
 * Testing API
 *
 * Test explorer and test runner support
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';
import { Uri } from './uri';
import { Range } from './textDocument';

export interface TestRunProfile {
  label: string;
  kind: TestRunProfileKind;
  isDefault: boolean;
  tag?: any;
  runHandler: (request: TestRunRequest, token: any) => void | Promise<void>;
  dispose(): void;
}

export enum TestRunProfileKind {
  Run = 1,
  Debug = 2,
  Coverage = 3
}

export interface TestItem {
  readonly id: string;
  readonly uri: Uri | undefined;
  readonly children: any; // TestItemCollection
  readonly parent: TestItem | undefined;
  label: string;
  description?: string;
  sortText?: string;
  canResolveChildren: boolean;
  busy: boolean;
  tags: readonly any[]; // TestTag[]
  range: Range | undefined;
  error: string | { value: string } | undefined;
}

export interface TestRunRequest {
  include?: readonly TestItem[];
  exclude?: readonly TestItem[];
  profile?: TestRunProfile;
}

export interface TestRun {
  readonly name: string | undefined;
  readonly token: any; // CancellationToken
  readonly isPersisted: boolean;
  enqueued(test: TestItem): void;
  started(test: TestItem): void;
  skipped(test: TestItem): void;
  failed(test: TestItem, message: any, duration?: number): void;
  errored(test: TestItem, message: any, duration?: number): void;
  passed(test: TestItem, duration?: number): void;
  appendOutput(output: string, location?: any, test?: TestItem): void;
  end(): void;
}

class TestRunImpl implements TestRun {
  constructor(
    private bridge: ExtensionHostBridge,
    private runId: string,
    public readonly name: string | undefined,
    public readonly token: any,
    public readonly isPersisted: boolean
  ) {}

  enqueued(test: TestItem): void {
    this.bridge.send('testRunEnqueued', {
      runId: this.runId,
      testId: test.id
    });
  }

  started(test: TestItem): void {
    this.bridge.send('testRunStarted', {
      runId: this.runId,
      testId: test.id
    });
  }

  skipped(test: TestItem): void {
    this.bridge.send('testRunSkipped', {
      runId: this.runId,
      testId: test.id
    });
  }

  failed(test: TestItem, message: any, duration?: number): void {
    this.bridge.send('testRunFailed', {
      runId: this.runId,
      testId: test.id,
      message,
      duration
    });
  }

  errored(test: TestItem, message: any, duration?: number): void {
    this.bridge.send('testRunErrored', {
      runId: this.runId,
      testId: test.id,
      message,
      duration
    });
  }

  passed(test: TestItem, duration?: number): void {
    this.bridge.send('testRunPassed', {
      runId: this.runId,
      testId: test.id,
      duration
    });
  }

  appendOutput(output: string, location?: any, test?: TestItem): void {
    this.bridge.send('testRunAppendOutput', {
      runId: this.runId,
      testId: test?.id,
      output,
      location
    });
  }

  end(): void {
    this.bridge.send('testRunEnd', { runId: this.runId });
  }
}

class TestRunProfileImpl implements TestRunProfile {
  private _label: string;
  private _isDefault = false;

  constructor(
    private bridge: ExtensionHostBridge,
    private profileId: string,
    label: string,
    public readonly kind: TestRunProfileKind,
    public readonly runHandler: (request: TestRunRequest, token: any) => void | Promise<void>,
    public tag?: any
  ) {
    this._label = label;
  }

  get label(): string {
    return this._label;
  }

  set label(value: string) {
    this._label = value;
  }

  get isDefault(): boolean {
    return this._isDefault;
  }

  set isDefault(value: boolean) {
    this._isDefault = value;
  }

  dispose(): void {
    this.bridge.send('disposeTestRunProfile', { profileId: this.profileId });
  }
}

export class TestController {
  readonly id!: string;
  get label(): string { return ''; }
  set label(value: string) {}
  get items(): any { return { size: 0 }; }
  createRunProfile(label: string, kind: TestRunProfileKind, runHandler: (request: TestRunRequest, token: any) => void | Promise<void>, isDefault?: boolean, tag?: any): TestRunProfile { throw new Error('Not implemented'); }
  createTestRun(request: TestRunRequest, name?: string, persist?: boolean): TestRun { throw new Error('Not implemented'); }
  createTestItem(id: string, label: string, uri?: Uri): TestItem { throw new Error('Not implemented'); }
  dispose(): void {}
}

class TestControllerImpl extends TestController {
  private _label: string;
  private _items: any = { size: 0 }; // TestItemCollection mock

  constructor(
    private bridge: ExtensionHostBridge,
    id: string,
    label: string
  ) {
    super();
    (this as any).id = id;
    this._label = label;
  }

  get label(): string {
    return this._label;
  }

  set label(value: string) {
    this._label = value;
  }

  get items(): any {
    return this._items;
  }

  createRunProfile(
    label: string,
    kind: TestRunProfileKind,
    runHandler: (request: TestRunRequest, token: any) => void | Promise<void>,
    isDefault?: boolean,
    tag?: any
  ): TestRunProfile {
    const profileId = `profile_${Date.now()}_${Math.random()}`;
    const profile = new TestRunProfileImpl(this.bridge, profileId, label, kind, runHandler, tag);
    if (isDefault) {
      profile.isDefault = true;
    }

    this.bridge.send('createTestRunProfile', {
      controllerId: this.id,
      profileId,
      label,
      kind,
      isDefault
    });

    return profile;
  }

  createTestRun(request: TestRunRequest, name?: string, persist?: boolean): TestRun {
    const runId = `run_${Date.now()}_${Math.random()}`;
    const token = { isCancellationRequested: false };

    this.bridge.send('createTestRun', {
      controllerId: this.id,
      runId,
      name,
      persist
    });

    return new TestRunImpl(this.bridge, runId, name, token, persist || false);
  }

  createTestItem(id: string, label: string, uri?: Uri): TestItem {
    const item: TestItem = {
      id,
      uri,
      children: { size: 0 },
      parent: undefined,
      label,
      description: undefined,
      sortText: undefined,
      canResolveChildren: false,
      busy: false,
      tags: [],
      range: undefined,
      error: undefined
    };

    this.bridge.send('createTestItem', {
      controllerId: this.id,
      id,
      label,
      uri
    });

    return item;
  }

  dispose(): void {
    this.bridge.send('disposeTestController', { id: this.id });
  }
}

export class TestingAPI {
  constructor(private bridge: ExtensionHostBridge) {}

  createTestController(id: string, label: string): TestController {
    return new TestControllerImpl(this.bridge, id, label);
  }
}
