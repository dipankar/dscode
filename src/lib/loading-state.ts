import { writable } from 'svelte/store';

export interface LoadingState {
  [key: string]: boolean;
}

function createLoadingStore() {
  const { subscribe, update } = writable<LoadingState>({});

  return {
    subscribe,

    start: (key: string) => {
      update(state => ({ ...state, [key]: true }));
    },

    stop: (key: string) => {
      update(state => {
        const newState = { ...state };
        delete newState[key];
        return newState;
      });
    },

    isLoading: (key: string): boolean => {
      let loading = false;
      subscribe(state => {
        loading = state[key] || false;
      })();
      return loading;
    },
  };
}

export const loadingStore = createLoadingStore();

/**
 * Wrap async function with loading state
 */
export async function withLoading<T>(
  key: string,
  fn: () => Promise<T>
): Promise<T> {
  loadingStore.start(key);
  try {
    return await fn();
  } finally {
    loadingStore.stop(key);
  }
}

/**
 * Create a derived store that tracks if any operations are loading
 */
export function createLoadingChecker(keys: string[]) {
  return {
    subscribe: (callback: (loading: boolean) => void) => {
      return loadingStore.subscribe(state => {
        const isLoading = keys.some(key => state[key]);
        callback(isLoading);
      });
    },
  };
}
