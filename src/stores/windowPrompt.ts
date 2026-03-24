import { writable } from 'svelte/store';

export type WindowPromptLevel = 'info' | 'warning' | 'error';

export interface WindowPrompt {
  id: string;
  level: WindowPromptLevel;
  message: string;
  actions: string[];
  kind?: 'backend' | 'local';
  resolve?: (action?: string) => void;
  showCancelButton?: boolean;
  cancelLabel?: string;
}

const store = writable<WindowPrompt | null>(null);

export const windowPromptStore = store;

export function showWindowPrompt(prompt: WindowPrompt) {
  store.set(prompt);
}

export function clearWindowPrompt() {
  store.set(null);
}

export function showConfirmPrompt(
  message: string,
  options: {
    level?: WindowPromptLevel;
    confirmLabel?: string;
    cancelLabel?: string;
  } = {}
): Promise<boolean> {
  const confirmLabel = options.confirmLabel ?? 'Confirm';
  const cancelLabel = options.cancelLabel ?? 'Cancel';

  return new Promise((resolve) => {
    showWindowPrompt({
      id: `local-${Date.now()}-${Math.random().toString(36).slice(2)}`,
      kind: 'local',
      level: options.level ?? 'warning',
      message,
      actions: [confirmLabel],
      showCancelButton: true,
      cancelLabel,
      resolve: (action) => {
        resolve(action === confirmLabel);
      },
    });
  });
}

export function showAlertPrompt(
  message: string,
  options: {
    level?: WindowPromptLevel;
    acknowledgeLabel?: string;
  } = {}
): Promise<void> {
  const acknowledgeLabel = options.acknowledgeLabel ?? 'OK';

  return new Promise((resolve) => {
    showWindowPrompt({
      id: `local-${Date.now()}-${Math.random().toString(36).slice(2)}`,
      kind: 'local',
      level: options.level ?? 'info',
      message,
      actions: [acknowledgeLabel],
      showCancelButton: false,
      resolve: () => resolve(),
    });
  });
}
