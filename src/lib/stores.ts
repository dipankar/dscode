import { writable, type Writable } from 'svelte/store';
import type { DebugSession, StackFrame, Variable, Scope } from './types';

// Debug session store
export const debugSession: Writable<DebugSession | null> = writable(null);

// Debug state stores
export const debugState: Writable<'Stopped' | 'Running' | 'Paused' | 'Terminated'> = writable('Stopped');
export const callStack: Writable<StackFrame[]> = writable([]);
export const variables: Writable<Variable[]> = writable([]);
export const scopes: Writable<Scope[]> = writable([]);
export const debugConsole: Writable<string[]> = writable([]);
