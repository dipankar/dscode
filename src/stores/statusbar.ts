import { derived, writable } from 'svelte/store';

export interface StatusBarCommand {
  id: string;
  arguments?: any[];
}

export interface StatusBarItemState {
  id: string;
  owner: string;
  text: string;
  tooltip?: string;
  color?: string;
  command?: StatusBarCommand;
  alignment: 'left' | 'right';
  priority?: number;
}

const itemsStore = writable<StatusBarItemState[]>([]);

export function setStatusBarItems(items: StatusBarItemState[] | undefined) {
  itemsStore.set(items ?? []);
}

export const statusBarItems = itemsStore;

export const leftStatusBarItems = derived(statusBarItems, ($items) =>
  $items.filter((item) => item.alignment === 'left')
);

export const rightStatusBarItems = derived(statusBarItems, ($items) =>
  $items.filter((item) => item.alignment === 'right')
);

export default statusBarItems;
