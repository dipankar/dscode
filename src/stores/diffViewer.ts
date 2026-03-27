import { writable } from 'svelte/store';

export interface DiffState {
  filePath: string;
  originalContent: string;
  modifiedContent: string;
  language: string;
}

export const diffStore = writable<DiffState | null>(null);

export function showDiff(state: DiffState) {
  diffStore.set(state);
}

export function closeDiff() {
  diffStore.set(null);
}
