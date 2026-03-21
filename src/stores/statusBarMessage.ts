import { writable } from 'svelte/store';

export interface StatusBarTransientMessage {
  id: string;
  text: string;
}

const store = writable<StatusBarTransientMessage | null>(null);

export function showStatusBarMessage(id: string, text: string) {
  store.set({ id, text });
}

export function clearStatusBarMessage(id: string) {
  store.update((current) => {
    if (current && current.id === id) {
      return null;
    }
    return current;
  });
}

export default store;
