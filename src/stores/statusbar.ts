import { derived, writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

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
  background_color?: string;
  command?: StatusBarCommand;
  alignment: 'left' | 'right';
  priority?: number;
}

const itemsStore = writable<StatusBarItemState[]>([]);

export function setStatusBarItems(items: StatusBarItemState[] | undefined) {
  itemsStore.set(items ?? []);
}

// Initialize listener for status bar updates
let initialized = false;

export async function initializeStatusBarStore() {
  if (initialized) return;
  initialized = true;

  // Load initial items
  try {
    const items = await invoke<StatusBarItemState[]>('get_status_bar_items');
    itemsStore.set(items);
  } catch (error) {
    console.error('[StatusBar] Failed to load initial items:', error);
  }

  // Listen for updates
  await listen<StatusBarItemState[]>('status-bar-items-changed', (event) => {
    itemsStore.set(event.payload);
  });
}

// Auto-initialize when imported
if (typeof window !== 'undefined') {
  initializeStatusBarStore().catch((error) => {
    console.error('[StatusBar] Failed to initialize store:', error);
  });
}

export const statusBarItems = itemsStore;

export const leftStatusBarItems = derived(statusBarItems, ($items) =>
  $items.filter((item) => item.alignment === 'left').sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0))
);

export const rightStatusBarItems = derived(statusBarItems, ($items) =>
  $items.filter((item) => item.alignment === 'right').sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0))
);

export default statusBarItems;
