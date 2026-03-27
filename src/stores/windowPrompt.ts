import { writable, get } from 'svelte/store';

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
  secondaryLabel?: string;
}

export interface InputBoxPrompt {
  id: string;
  prompt: string;
  placeholder?: string;
  value?: string;
  resolve?: (value: string | null) => void;
}

const store = writable<WindowPrompt | null>(null);
const inputBoxPromptStore = writable<InputBoxPrompt | null>(null);

export const windowPromptStore = store;
export const localInputBoxStore = inputBoxPromptStore;

export function showWindowPrompt(prompt: WindowPrompt) {
  store.set(prompt);
}

export function clearWindowPrompt() {
  store.set(null);
}

export type ConfirmResult = boolean | undefined;

export function showConfirmPrompt(
  message: string,
  options: {
    level?: WindowPromptLevel;
    confirmLabel?: string;
    cancelLabel?: string;
    secondaryLabel?: string;
  } = {}
): Promise<ConfirmResult> {
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
      secondaryLabel: options.secondaryLabel,
      resolve: (action) => {
        if (action === confirmLabel) {
          resolve(true);
        } else if (action === cancelLabel) {
          resolve(false);
        } else {
          resolve(undefined);
        }
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

export function showLocalInputBox(
  prompt: string,
  options: {
    placeholder?: string;
    value?: string;
  } = {}
): Promise<string | null> {
  return new Promise((resolve) => {
    inputBoxPromptStore.set({
      id: `input-${Date.now()}-${Math.random().toString(36).slice(2)}`,
      prompt,
      placeholder: options.placeholder,
      value: options.value,
      resolve,
    });
  });
}

export function resolveLocalInputBox(value: string | null) {
  const current = get(inputBoxPromptStore);
  if (current) {
    current.resolve?.(value);
    inputBoxPromptStore.set(null);
  }
}
