import { writable } from 'svelte/store';

export type WindowPromptLevel = 'info' | 'warning' | 'error';

export interface WindowPrompt {
  id: string;
  level: WindowPromptLevel;
  message: string;
  actions: string[];
}

const store = writable<WindowPrompt | null>(null);

export const windowPromptStore = store;

export function showWindowPrompt(prompt: WindowPrompt) {
  store.set(prompt);
}

export function clearWindowPrompt() {
  store.set(null);
}
