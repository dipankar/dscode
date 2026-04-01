import { writable } from 'svelte/store';

export interface Toast {
  id: string;
  type: 'error' | 'warning' | 'info' | 'success';
  message: string;
  details?: string;
  duration?: number;
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  let idCounter = 0;

  return {
    subscribe,

    show: (toast: Omit<Toast, 'id'>) => {
      const id = `toast-${idCounter++}`;
      const newToast: Toast = {
        id,
        duration: 5000,
        ...toast,
      };

      update(toasts => [...toasts, newToast]);

      if (newToast.duration && newToast.duration > 0) {
        setTimeout(() => {
          update(toasts => toasts.filter(t => t.id !== id));
        }, newToast.duration);
      }

      return id;
    },

    dismiss: (id: string) => {
      update(toasts => toasts.filter(t => t.id !== id));
    },

    clear: () => {
      update(() => []);
    },
  };
}

export const toastStore = createToastStore();

/**
 * Parse error and return user-friendly message
 */
export function parseError(error: unknown): { message: string; details?: string } {
  if (typeof error === 'string') {
    return { message: error };
  }

  if (error instanceof Error) {
    return {
      message: error.message,
      details: error.stack,
    };
  }

  if (error && typeof error === 'object') {
    const err = error as any;

    // Tauri error format
    if (err.message) {
      return {
        message: err.message,
        details: err.stack || JSON.stringify(err, null, 2),
      };
    }

    // Generic object
    return {
      message: 'An unexpected error occurred. Please try again or check the console for details.',
      details: JSON.stringify(err, null, 2),
    };
  }

  return { message: 'An unknown error occurred. Please try again or reload the application.' };
}

/**
 * Handle error and show toast notification
 */
export function handleError(error: unknown, context?: string) {
  const { message, details } = parseError(error);
  const fullMessage = context ? `${context}: ${message}` : message;

  console.error(fullMessage, details || error);

  toastStore.show({
    type: 'error',
    message: fullMessage,
    details,
  });
}

/**
 * Show success message
 */
export function showSuccess(message: string) {
  toastStore.show({
    type: 'success',
    message,
    duration: 3000,
  });
}

/**
 * Show info message
 */
export function showInfo(message: string, duration = 4000) {
  toastStore.show({
    type: 'info',
    message,
    duration,
  });
}

/**
 * Show warning message
 */
export function showWarning(message: string) {
  toastStore.show({
    type: 'warning',
    message,
    duration: 5000,
  });
}

/**
 * Wrap async function with error handling
 */
export function withErrorHandling<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  context?: string
): T {
  return (async (...args: any[]) => {
    try {
      return await fn(...args);
    } catch (error) {
      handleError(error, context);
      throw error;
    }
  }) as T;
}

/**
 * Retry async function with exponential backoff
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  options: {
    maxRetries?: number;
    initialDelay?: number;
    maxDelay?: number;
    backoffFactor?: number;
    onRetry?: (attempt: number, error: unknown) => void;
  } = {}
): Promise<T> {
  const {
    maxRetries = 3,
    initialDelay = 1000,
    maxDelay = 10000,
    backoffFactor = 2,
    onRetry,
  } = options;

  let lastError: unknown;
  let delay = initialDelay;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;

      if (attempt < maxRetries) {
        onRetry?.(attempt + 1, error);
        await new Promise(resolve => setTimeout(resolve, delay));
        delay = Math.min(delay * backoffFactor, maxDelay);
      }
    }
  }

  throw lastError;
}
