import { writable } from 'svelte/store';

export interface AppErrorState {
  title: string;
  message: string;
  stack?: string;
}

const store = writable<AppErrorState | null>(null);

export const appErrorStore = store;

export function setAppError(error: unknown, title = 'Unexpected application error') {
  if (error instanceof Error) {
    store.set({
      title,
      message: error.message,
      stack: error.stack,
    });
    return;
  }

  store.set({
    title,
    message: typeof error === 'string' ? error : 'Unknown error',
  });
}

export function clearAppError() {
  store.set(null);
}
