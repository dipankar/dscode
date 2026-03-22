import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ===== Type Definitions =====

export interface TaskProvider {
  id: string;
  owner: string;
  taskType: string;
  tasks: TaskDefinition[];
}

export interface TaskDefinition {
  name: string;
  taskType: string;
  command: string;
  args: string[];
  options: TaskOptions;
  problemMatchers: ProblemMatcher[];
  group?: TaskGroup;
  presentation: TaskPresentation;
  runOptions: RunOptions;
}

export interface TaskOptions {
  cwd?: string;
  env: Record<string, string>;
  shell?: ShellConfiguration;
}

export interface ShellConfiguration {
  executable: string;
  args: string[];
}

export interface ProblemMatcher {
  owner: string;
  pattern: ProblemPattern;
  severity?: string;
  source?: string;
  fileLocation: 'auto' | 'relative' | 'absolute';
  background?: BackgroundPattern;
}

export interface ProblemPattern {
  regexp: string;
  file?: number;
  location?: number;
  line?: number;
  column?: number;
  endLine?: number;
  endColumn?: number;
  message: number;
  code?: number;
  severity?: number;
  loop: boolean;
}

export interface BackgroundPattern {
  activeOnStart: boolean;
  beginsPattern: string;
  endsPattern: string;
}

export interface TaskGroup {
  kind: 'build' | 'test' | 'clean';
  isDefault: boolean;
}

export interface TaskPresentation {
  reveal: 'always' | 'silent' | 'never';
  echo: boolean;
  focus: boolean;
  panel: 'shared' | 'dedicated' | 'new';
  showReuseMessage: boolean;
  clear: boolean;
  group?: string;
}

export interface RunOptions {
  runOn: 'default' | 'folderopen';
  instanceLimit: number;
  reevaluateOnRerun: boolean;
}

export interface TaskExecution {
  id: string;
  task: TaskDefinition;
  terminalId?: string;
  state: 'running' | 'completed' | 'failed' | 'cancelled';
  exitCode?: number;
  startedAt: string;
}

export interface TaskStartedEvent {
  execution: TaskExecution;
}

export interface TaskUpdatedEvent {
  execution: TaskExecution;
}

export interface TaskEndedEvent {
  execution: TaskExecution;
}

/**
 * Task Manager for task providers and executions
 */
export class TaskManager {
  private onTaskStartedCallbacks: Array<(event: TaskStartedEvent) => void> = [];
  private onTaskUpdatedCallbacks: Array<(event: TaskUpdatedEvent) => void> = [];
  private onTaskEndedCallbacks: Array<(event: TaskEndedEvent) => void> = [];

  constructor() {}

  /**
   * Initialize task manager
   */
  async initialize(): Promise<void> {
    // Listen for task events
    await listen<TaskExecution>('task-started', (event) => {
      this.notifyTaskStarted({ execution: event.payload });
    });

    await listen<TaskExecution>('task-updated', (event) => {
      this.notifyTaskUpdated({ execution: event.payload });
    });

    await listen<TaskExecution>('task-ended', (event) => {
      this.notifyTaskEnded({ execution: event.payload });
    });

    console.log('[Task] Initialized');
  }

  // ===== Task Provider Management =====

  /**
   * Register task provider
   */
  async registerTaskProvider(provider: TaskProvider): Promise<string> {
    return await invoke<string>('register_task_provider', { provider });
  }

  /**
   * Unregister task provider
   */
  async unregisterTaskProvider(providerId: string): Promise<void> {
    await invoke('unregister_task_provider', { providerId });
  }

  /**
   * Get all task providers
   */
  async getTaskProviders(): Promise<TaskProvider[]> {
    return await invoke<TaskProvider[]>('get_task_providers');
  }

  /**
   * Get task providers by type
   */
  async getTaskProvidersByType(taskType: string): Promise<TaskProvider[]> {
    return await invoke<TaskProvider[]>('get_task_providers_by_type', { taskType });
  }

  /**
   * Get task provider by ID
   */
  async getTaskProvider(providerId: string): Promise<TaskProvider> {
    return await invoke<TaskProvider>('get_task_provider', { providerId });
  }

  // ===== Task Execution Management =====

  /**
   * Start task execution
   */
  async startTaskExecution(execution: TaskExecution): Promise<string> {
    return await invoke<string>('start_task_execution', { execution });
  }

  /**
   * Update task execution
   */
  async updateTaskExecution(
    executionId: string,
    state: 'running' | 'completed' | 'failed' | 'cancelled',
    exitCode?: number
  ): Promise<void> {
    await invoke('update_task_execution', { executionId, state, exitCode });
  }

  /**
   * End task execution
   */
  async endTaskExecution(executionId: string, exitCode: number): Promise<void> {
    await invoke('end_task_execution', { executionId, exitCode });
  }

  /**
   * Get task execution
   */
  async getTaskExecution(executionId: string): Promise<TaskExecution> {
    return await invoke<TaskExecution>('get_task_execution', { executionId });
  }

  /**
   * Get all task executions
   */
  async getAllTaskExecutions(): Promise<TaskExecution[]> {
    return await invoke<TaskExecution[]>('get_all_task_executions');
  }

  /**
   * Clear task executions
   */
  async clearTaskExecutions(): Promise<void> {
    await invoke('clear_task_executions');
  }

  // ===== Utility Methods =====

  /**
   * Clear task data for owner
   */
  async clearTaskData(owner: string): Promise<void> {
    await invoke('clear_task_data', { owner });
  }

  /**
   * Create task definition helper
   */
  createTaskDefinition(
    name: string,
    taskType: string,
    command: string,
    args: string[] = [],
    options: Partial<TaskOptions> = {},
    presentation: Partial<TaskPresentation> = {}
  ): TaskDefinition {
    return {
      name,
      taskType,
      command,
      args,
      options: {
        cwd: options.cwd,
        env: options.env || {},
        shell: options.shell,
      },
      problemMatchers: [],
      presentation: {
        reveal: presentation.reveal || 'always',
        echo: presentation.echo !== undefined ? presentation.echo : true,
        focus: presentation.focus !== undefined ? presentation.focus : false,
        panel: presentation.panel || 'shared',
        showReuseMessage: presentation.showReuseMessage !== undefined ? presentation.showReuseMessage : true,
        clear: presentation.clear !== undefined ? presentation.clear : false,
        group: presentation.group,
      },
      runOptions: {
        runOn: 'default',
        instanceLimit: 1,
        reevaluateOnRerun: true,
      },
    };
  }

  /**
   * Create problem matcher helper
   */
  createProblemMatcher(
    owner: string,
    regexp: string,
    messageIndex: number,
    fileIndex?: number,
    lineIndex?: number,
    columnIndex?: number
  ): ProblemMatcher {
    return {
      owner,
      pattern: {
        regexp,
        file: fileIndex,
        line: lineIndex,
        column: columnIndex,
        message: messageIndex,
        loop: false,
      },
      fileLocation: 'relative',
    };
  }

  /**
   * Parse problem matcher output
   */
  parseProblemMatcherOutput(
    output: string,
    matcher: ProblemMatcher
  ): Array<{
    file?: string;
    line?: number;
    column?: number;
    message: string;
    severity?: string;
  }> {
    const results: Array<{
      file?: string;
      line?: number;
      column?: number;
      message: string;
      severity?: string;
    }> = [];

    const regex = new RegExp(matcher.pattern.regexp, 'gm');
    let match;

    while ((match = regex.exec(output)) !== null) {
      const result: {
        file?: string;
        line?: number;
        column?: number;
        message: string;
        severity?: string;
      } = {
        message: match[matcher.pattern.message] || '',
      };

      if (matcher.pattern.file !== undefined && match[matcher.pattern.file]) {
        result.file = match[matcher.pattern.file];
      }

      if (matcher.pattern.line !== undefined && match[matcher.pattern.line]) {
        result.line = parseInt(match[matcher.pattern.line], 10);
      }

      if (matcher.pattern.column !== undefined && match[matcher.pattern.column]) {
        result.column = parseInt(match[matcher.pattern.column], 10);
      }

      if (matcher.pattern.severity !== undefined && match[matcher.pattern.severity]) {
        result.severity = match[matcher.pattern.severity];
      } else if (matcher.severity) {
        result.severity = matcher.severity;
      }

      results.push(result);

      if (!matcher.pattern.loop) {
        break;
      }
    }

    return results;
  }

  // ===== Event Subscriptions =====

  /**
   * Subscribe to task started events
   */
  onTaskStarted(callback: (event: TaskStartedEvent) => void): () => void {
    this.onTaskStartedCallbacks.push(callback);

    return () => {
      const index = this.onTaskStartedCallbacks.indexOf(callback);
      if (index > -1) {
        this.onTaskStartedCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to task updated events
   */
  onTaskUpdated(callback: (event: TaskUpdatedEvent) => void): () => void {
    this.onTaskUpdatedCallbacks.push(callback);

    return () => {
      const index = this.onTaskUpdatedCallbacks.indexOf(callback);
      if (index > -1) {
        this.onTaskUpdatedCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to task ended events
   */
  onTaskEnded(callback: (event: TaskEndedEvent) => void): () => void {
    this.onTaskEndedCallbacks.push(callback);

    return () => {
      const index = this.onTaskEndedCallbacks.indexOf(callback);
      if (index > -1) {
        this.onTaskEndedCallbacks.splice(index, 1);
      }
    };
  }

  // ===== Internal Methods =====

  private notifyTaskStarted(event: TaskStartedEvent): void {
    for (const callback of this.onTaskStartedCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[Task] Error in task started callback:', error);
      }
    }
  }

  private notifyTaskUpdated(event: TaskUpdatedEvent): void {
    for (const callback of this.onTaskUpdatedCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[Task] Error in task updated callback:', error);
      }
    }
  }

  private notifyTaskEnded(event: TaskEndedEvent): void {
    for (const callback of this.onTaskEndedCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[Task] Error in task ended callback:', error);
      }
    }
  }
}

// Export singleton instance
export const taskManager = new TaskManager();
