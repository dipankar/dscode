// Debug types matching the Rust backend
export interface Breakpoint {
  id?: number;
  verified: boolean;
  message?: string;
  source?: Source;
  line?: number;
  column?: number;
}

export interface Source {
  name?: string;
  path?: string;
  sourceReference?: number;
}

export interface StackFrame {
  id: number;
  name: string;
  source?: Source;
  line: number;
  column: number;
}

export interface Variable {
  name: string;
  value: string;
  type?: string;
  variablesReference: number;
}

export interface Scope {
  name: string;
  variablesReference: number;
  expensive: boolean;
}

export type DebugState = 'Stopped' | 'Running' | 'Paused' | 'Terminated';

export interface DebugSession {
  id: string;
  name: string;
  state: DebugState;
  adapter_type: string;
}

export interface LaunchRequestArguments {
  type: string;
  request: string;
  name: string;
  program: string;
  [key: string]: any;
}
