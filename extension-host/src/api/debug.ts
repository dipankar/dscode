/**
 * Debug API
 *
 * Debugging support
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';

export class DebugSession {
  readonly id!: string;
  readonly type!: string;
  readonly name!: string;
  readonly workspaceFolder!: any | undefined;
  readonly configuration!: DebugConfiguration;
  customRequest(command: string, args?: any): Promise<any> { return Promise.resolve(undefined); }
  getDebugProtocolBreakpoint(breakpoint: any): Promise<any | undefined> { return Promise.resolve(undefined); }
}

export interface DebugConfiguration {
  type: string;
  name: string;
  request: string;
  [key: string]: any;
}

export interface DebugAdapterDescriptor {}

export interface DebugAdapterExecutable extends DebugAdapterDescriptor {
  readonly command: string;
  readonly args: string[];
  readonly options?: { cwd?: string; env?: { [key: string]: string } };
}

export interface DebugAdapterServer extends DebugAdapterDescriptor {
  readonly port: number;
  readonly host?: string;
}

export interface DebugConfigurationProvider {
  provideDebugConfigurations?(folder: any | undefined, token?: any): Promise<DebugConfiguration[]>;
  resolveDebugConfiguration?(folder: any | undefined, config: DebugConfiguration, token?: any): Promise<DebugConfiguration | null | undefined>;
  resolveDebugConfigurationWithSubstitutedVariables?(folder: any | undefined, config: DebugConfiguration, token?: any): Promise<DebugConfiguration | null | undefined>;
}

export interface DebugAdapterDescriptorFactory {
  createDebugAdapterDescriptor(session: DebugSession, executable: DebugAdapterExecutable | undefined): Promise<DebugAdapterDescriptor>;
}

export interface DebugAdapterTrackerFactory {
  createDebugAdapterTracker(session: DebugSession): Promise<DebugAdapterTracker>;
}

export interface DebugAdapterTracker {
  onWillStartSession?(): void;
  onWillReceiveMessage?(message: any): void;
  onDidSendMessage?(message: any): void;
  onWillStopSession?(): void;
  onError?(error: Error): void;
  onExit?(code: number | undefined, signal: string | undefined): void;
}

export class Breakpoint {
  readonly id!: string;
  readonly enabled!: boolean;
  readonly condition?: string;
  readonly hitCondition?: string;
  readonly logMessage?: string;
}

export class SourceBreakpoint extends Breakpoint {
  readonly location!: any; // Location
}

export class FunctionBreakpoint extends Breakpoint {
  readonly functionName!: string;
}

class DebugSessionImpl extends DebugSession {
  constructor(
    private bridge: ExtensionHostBridge,
    id: string,
    type: string,
    name: string,
    workspaceFolder: any | undefined,
    configuration: DebugConfiguration
  ) {
    super();
    (this as any).id = id;
    (this as any).type = type;
    (this as any).name = name;
    (this as any).workspaceFolder = workspaceFolder;
    (this as any).configuration = configuration;
  }

  async customRequest(command: string, args?: any): Promise<any> {
    const result = await this.bridge.request('debugCustomRequest', {
      sessionId: this.id,
      command,
      args
    }) as { response?: any };
    return result.response;
  }

  async getDebugProtocolBreakpoint(breakpoint: any): Promise<any | undefined> {
    const result = await this.bridge.request('debugGetProtocolBreakpoint', {
      sessionId: this.id,
      breakpoint
    }) as { protocolBreakpoint?: any };
    return result.protocolBreakpoint;
  }
}

export class DebugAPI {
  private _onDidChangeActiveDebugSession = new EventEmitter<DebugSession | undefined>();
  private _onDidStartDebugSession = new EventEmitter<DebugSession>();
  private _onDidReceiveDebugSessionCustomEvent = new EventEmitter<any>();
  private _onDidTerminateDebugSession = new EventEmitter<DebugSession>();
  private _onDidChangeBreakpoints = new EventEmitter<any>();

  readonly onDidChangeActiveDebugSession = this._onDidChangeActiveDebugSession.event;
  readonly onDidStartDebugSession = this._onDidStartDebugSession.event;
  readonly onDidReceiveDebugSessionCustomEvent = this._onDidReceiveDebugSessionCustomEvent.event;
  readonly onDidTerminateDebugSession = this._onDidTerminateDebugSession.event;
  readonly onDidChangeBreakpoints = this._onDidChangeBreakpoints.event;

  private _activeDebugSession?: DebugSession;
  private _breakpoints: Breakpoint[] = [];

  constructor(private bridge: ExtensionHostBridge) {
    this.setupListeners();
  }

  private setupListeners(): void {
    this.bridge.on('debugSessionStarted', (data: any) => {
      const session = new DebugSessionImpl(
        this.bridge,
        data.id,
        data.type,
        data.name,
        data.workspaceFolder,
        data.configuration
      );
      this._onDidStartDebugSession.fire(session);
    });

    this.bridge.on('debugSessionTerminated', (data: any) => {
      const session = new DebugSessionImpl(
        this.bridge,
        data.id,
        data.type,
        data.name,
        data.workspaceFolder,
        data.configuration
      );
      this._onDidTerminateDebugSession.fire(session);
    });
  }

  get activeDebugSession(): DebugSession | undefined {
    return this._activeDebugSession;
  }

  get activeDebugConsole(): any {
    return {
      append: (value: string) => {
        this.bridge.send('debugConsoleAppend', { value });
      },
      appendLine: (value: string) => {
        this.bridge.send('debugConsoleAppendLine', { value });
      }
    };
  }

  get breakpoints(): readonly Breakpoint[] {
    return this._breakpoints;
  }

  registerDebugConfigurationProvider(
    debugType: string,
    provider: DebugConfigurationProvider,
    triggerKind?: any
  ): Disposable {
    this.bridge.send('registerDebugConfigurationProvider', {
      debugType,
      triggerKind
    });

    return {
      dispose: () => {
        this.bridge.send('unregisterDebugConfigurationProvider', { debugType });
      }
    };
  }

  registerDebugAdapterDescriptorFactory(
    debugType: string,
    factory: DebugAdapterDescriptorFactory
  ): Disposable {
    this.bridge.send('registerDebugAdapterDescriptorFactory', { debugType });

    return {
      dispose: () => {
        this.bridge.send('unregisterDebugAdapterDescriptorFactory', { debugType });
      }
    };
  }

  registerDebugAdapterTrackerFactory(
    debugType: string,
    factory: DebugAdapterTrackerFactory
  ): Disposable {
    this.bridge.send('registerDebugAdapterTrackerFactory', { debugType });

    return {
      dispose: () => {
        this.bridge.send('unregisterDebugAdapterTrackerFactory', { debugType });
      }
    };
  }

  async startDebugging(
    folder: any | undefined,
    nameOrConfiguration: string | DebugConfiguration,
    parentSessionOrOptions?: DebugSession | any
  ): Promise<boolean> {
    const result = await this.bridge.request('startDebugging', {
      folder,
      nameOrConfiguration,
      parentSessionOrOptions
    }) as { success?: boolean };
    return result.success || false;
  }

  async stopDebugging(session?: DebugSession): Promise<void> {
    await this.bridge.request('stopDebugging', {
      sessionId: session?.id
    });
  }

  addBreakpoints(breakpoints: Breakpoint[]): void {
    this._breakpoints.push(...breakpoints);
    this.bridge.send('addBreakpoints', { breakpoints });
    this._onDidChangeBreakpoints.fire({ added: breakpoints, removed: [], changed: [] });
  }

  removeBreakpoints(breakpoints: Breakpoint[]): void {
    this._breakpoints = this._breakpoints.filter(bp => !breakpoints.includes(bp));
    this.bridge.send('removeBreakpoints', { breakpoints });
    this._onDidChangeBreakpoints.fire({ added: [], removed: breakpoints, changed: [] });
  }

  asDebugSourceUri(source: any, session?: DebugSession): any {
    return { scheme: 'debug', path: source.path };
  }
}

// Tasks API
export class Task {
  readonly definition!: TaskDefinition;
  readonly scope!: any | undefined;
  name!: string;
  detail?: string;
  execution?: ProcessExecution | ShellExecution | CustomExecution;
  isBackground!: boolean;
  source!: string;
  group?: TaskGroup;
  presentationOptions!: TaskPresentationOptions;
  problemMatchers!: string[];
  runOptions!: RunOptions;

  constructor(
    definition?: TaskDefinition,
    scope?: any,
    name?: string,
    source?: string,
    execution?: ProcessExecution | ShellExecution | CustomExecution,
    problemMatchers?: string | string[]
  ) {
    if (definition) (this as any).definition = definition;
    if (scope !== undefined) (this as any).scope = scope;
    if (name) this.name = name;
    if (source) this.source = source;
    if (execution) this.execution = execution;
    this.isBackground = false;
    this.presentationOptions = {};
    this.problemMatchers = Array.isArray(problemMatchers) ? problemMatchers : problemMatchers ? [problemMatchers] : [];
    this.runOptions = {};
  }
}

export interface TaskDefinition {
  readonly type: string;
  [name: string]: any;
}

export interface ProcessExecution {
  process: string;
  args: string[];
  options?: { cwd?: string; env?: { [key: string]: string } };
}

export interface ShellExecution {
  commandLine?: string;
  command?: string | { value: string; quoting: any };
  args?: Array<string | { value: string; quoting: any }>;
  options?: { cwd?: string; env?: { [key: string]: string }; executable?: string; shellArgs?: string[] };
}

export interface CustomExecution {
  callback: (resolvedDefinition: TaskDefinition) => Promise<Pseudoterminal>;
}

export interface Pseudoterminal {
  onDidWrite: Event<string>;
  onDidClose?: Event<number | void>;
  open(): void;
  close(): void;
  handleInput?(data: string): void;
}

export interface TaskGroup {
  readonly id: string;
  readonly label: string;
  isDefault?: boolean | { filePattern?: string };
}

export namespace TaskGroup {
  export const Clean: TaskGroup = { id: 'clean', label: 'Clean' };
  export const Build: TaskGroup = { id: 'build', label: 'Build' };
  export const Rebuild: TaskGroup = { id: 'rebuild', label: 'Rebuild' };
  export const Test: TaskGroup = { id: 'test', label: 'Test' };
}

export interface TaskPresentationOptions {
  reveal?: TaskRevealKind;
  echo?: boolean;
  focus?: boolean;
  panel?: TaskPanelKind;
  showReuseMessage?: boolean;
  clear?: boolean;
  group?: string;
}

export enum TaskRevealKind {
  Always = 1,
  Silent = 2,
  Never = 3
}

export enum TaskPanelKind {
  Shared = 1,
  Dedicated = 2,
  New = 3
}

export interface RunOptions {
  reevaluateOnRerun?: boolean;
}

export interface TaskProvider<T extends Task = Task> {
  provideTasks(token?: any): Promise<T[]>;
  resolveTask(task: T, token?: any): Promise<T | undefined>;
}

export class TasksAPI {
  private _onDidStartTask = new EventEmitter<any>();
  private _onDidEndTask = new EventEmitter<any>();
  private _onDidStartTaskProcess = new EventEmitter<any>();
  private _onDidEndTaskProcess = new EventEmitter<any>();

  readonly onDidStartTask = this._onDidStartTask.event;
  readonly onDidEndTask = this._onDidEndTask.event;
  readonly onDidStartTaskProcess = this._onDidStartTaskProcess.event;
  readonly onDidEndTaskProcess = this._onDidEndTaskProcess.event;

  constructor(private bridge: ExtensionHostBridge) {}

  async fetchTasks(filter?: any): Promise<Task[]> {
    const result = await this.bridge.request('fetchTasks', { filter }) as { tasks?: Task[] };
    return result.tasks || [];
  }

  async executeTask(task: Task): Promise<any> {
    const result = await this.bridge.request('executeTask', { task }) as { execution?: any };
    return result.execution;
  }

  registerTaskProvider(type: string, provider: TaskProvider): Disposable {
    this.bridge.send('registerTaskProvider', { type });

    this.bridge.on(`taskProvider:${type}:provideTasks`, async (data: any) => {
      const tasks = await provider.provideTasks(data.token);
      this.bridge.send('taskProviderResult', {
        requestId: data.requestId,
        tasks
      });
    });

    return {
      dispose: () => {
        this.bridge.send('unregisterTaskProvider', { type });
      }
    };
  }
}
