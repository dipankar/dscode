import { derived, writable } from 'svelte/store';

export interface OutputChannelState {
  id: string;
  lines: string[];
  visible: boolean;
}

type OutputChannelMap = Record<string, OutputChannelState>;

const channelsStore = writable<OutputChannelMap>({});
const activeChannelStore = writable<string | null>(null);
const revealStore = writable<{ channel: string; token: number } | null>(null);
let revealCounter = 0;

export const outputChannels = channelsStore;
export const activeOutputChannel = activeChannelStore;
export const outputChannelReveal = revealStore;

export const outputChannelList = derived(outputChannels, ($channels) => {
  return Object.values($channels).sort((a, b) => a.id.localeCompare(b.id));
});

function ensureChannel(map: OutputChannelMap, id: string): OutputChannelState {
  if (!map[id]) {
    map[id] = {
      id,
      lines: [],
      visible: false,
    };
  }
  return map[id];
}

export function registerOutputChannel(id: string) {
  channelsStore.update((map) => {
    const next = { ...map };
    ensureChannel(next, id);
    return next;
  });
}

export function appendOutputChannel(id: string, value: string) {
  channelsStore.update((map) => {
    const next = { ...map };
    const entry = ensureChannel(next, id);
    const lines = value.split(/\r?\n/);
    entry.lines = [...entry.lines, ...lines];
    if (entry.lines.length > 2000) {
      entry.lines = entry.lines.slice(entry.lines.length - 2000);
    }
    next[id] = { ...entry };
    return next;
  });
}

export function clearOutputChannel(id: string) {
  channelsStore.update((map) => {
    if (!map[id]) {
      return map;
    }
    const next = { ...map };
    next[id] = {
      ...next[id],
      lines: [],
    };
    return next;
  });
}

export function disposeOutputChannel(id: string) {
  channelsStore.update((map) => {
    if (!map[id]) {
      return map;
    }
    const next = { ...map };
    delete next[id];
    return next;
  });

  activeOutputChannel.update((current) => {
    if (current === id) {
      let fallback: string | null = null;
      const remaining = Object.keys(getStoreSnapshot(channelsStore));
      fallback = remaining.length > 0 ? remaining[0] : null;
      return fallback;
    }
    return current;
  });
}

export function setOutputChannelVisibility(id: string, visible: boolean) {
  channelsStore.update((map) => {
    const next = { ...map };
    const entry = ensureChannel(next, id);
    entry.visible = visible;
    next[id] = { ...entry };
    return next;
  });

  if (visible) {
    activeOutputChannel.set(id);
    revealCounter += 1;
    revealStore.set({ channel: id, token: revealCounter });
  } else {
    activeOutputChannel.update((current) => (current === id ? null : current));
    revealStore.set(null);
  }
}

export function setActiveOutputChannel(id: string) {
  activeOutputChannel.set(id);
}

function getStoreSnapshot<T>(store: { subscribe: (run: (value: T) => void) => () => void }): T {
  let value: T | undefined;
  const unsubscribe = store.subscribe((v) => {
    value = v;
  });
  unsubscribe();
  if (value === undefined) {
    throw new Error('Failed to read store snapshot');
  }
  return value;
}
