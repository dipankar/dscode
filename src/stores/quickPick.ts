import { writable } from 'svelte/store';

export interface QuickPickEntry {
  id: number;
  label: string;
  description?: string;
  detail?: string;
  raw: any;
}

export interface QuickPickState {
  id: string;
  items: QuickPickEntry[];
  canPickMany: boolean;
  placeHolder?: string;
  title?: string;
  matchOnDescription: boolean;
  matchOnDetail: boolean;
}

interface QuickPickEventData {
  id: string;
  items: any[];
  can_pick_many?: boolean;
  place_holder?: string | null;
  title?: string | null;
  match_on_description?: boolean;
  match_on_detail?: boolean;
}

const store = writable<QuickPickState | null>(null);

function normalizeItem(item: any, index: number): QuickPickEntry {
  if (typeof item === 'string') {
    return {
      id: index,
      label: item,
      raw: item,
    };
  }

  if (item && typeof item === 'object') {
    const label = typeof item.label === 'string' ? item.label : JSON.stringify(item.label ?? '');
    return {
      id: index,
      label,
      description: typeof item.description === 'string' ? item.description : undefined,
      detail: typeof item.detail === 'string' ? item.detail : undefined,
      raw: item,
    };
  }

  return {
    id: index,
    label: String(item ?? ''),
    raw: item,
  };
}

export function showQuickPick(data: QuickPickEventData) {
  const normalizedItems = (data.items ?? []).map((item, index) => normalizeItem(item, index));

  store.set({
    id: data.id,
    items: normalizedItems,
    canPickMany: Boolean(data.can_pick_many),
    placeHolder: data.place_holder ?? undefined,
    title: data.title ?? undefined,
    matchOnDescription: Boolean(data.match_on_description),
    matchOnDetail: Boolean(data.match_on_detail),
  });
}

export function clearQuickPick() {
  store.set(null);
}

export const quickPickStore = store;

export default quickPickStore;
