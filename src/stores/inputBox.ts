import { writable } from 'svelte/store';

export interface InputBoxState {
  id: string;
  prompt?: string;
  placeHolder?: string;
  value?: string;
  password: boolean;
  valueSelection?: [number, number];
}

interface InputBoxEventData {
  id: string;
  prompt?: string | null;
  place_holder?: string | null;
  value?: string | null;
  password?: boolean;
  value_selection?: number[] | null;
}

const store = writable<InputBoxState | null>(null);

export function showInputBox(data: InputBoxEventData) {
  let selection: [number, number] | undefined;
  if (Array.isArray(data.value_selection) && data.value_selection.length === 2) {
    const [start, end] = data.value_selection;
    if (typeof start === 'number' && typeof end === 'number') {
      selection = [start, end];
    }
  }

  store.set({
    id: data.id,
    prompt: data.prompt ?? undefined,
    placeHolder: data.place_holder ?? undefined,
    value: data.value ?? undefined,
    password: Boolean(data.password),
    valueSelection: selection,
  });
}

export function clearInputBox() {
  store.set(null);
}

export const inputBoxStore = store;

export default inputBoxStore;
