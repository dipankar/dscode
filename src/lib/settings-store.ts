import { writable } from 'svelte/store';

export interface Settings {
  editor: {
    fontSize: number;
    fontFamily: string;
    tabSize: number;
    insertSpaces: boolean;
    wordWrap: 'on' | 'off' | 'wordWrapColumn';
    lineNumbers: 'on' | 'off' | 'relative';
    minimap: boolean;
    rulers: number[];
  };
  theme: {
    colorTheme: 'dark' | 'light' | 'high-contrast';
  };
  terminal: {
    fontSize: number;
    fontFamily: string;
  };
  git: {
    autoFetch: boolean;
    confirmSync: boolean;
  };
}

const defaultSettings: Settings = {
  editor: {
    fontSize: 14,
    fontFamily: "'Fira Code', 'Consolas', monospace",
    tabSize: 2,
    insertSpaces: true,
    wordWrap: 'off',
    lineNumbers: 'on',
    minimap: true,
    rulers: [],
  },
  theme: {
    colorTheme: 'dark',
  },
  terminal: {
    fontSize: 14,
    fontFamily: "'Menlo', 'Monaco', 'Courier New', monospace",
  },
  git: {
    autoFetch: false,
    confirmSync: true,
  },
};

function loadSettings(): Settings {
  if (typeof localStorage === 'undefined') return defaultSettings;

  const stored = localStorage.getItem('dscode-settings');
  if (!stored) return defaultSettings;

  try {
    return { ...defaultSettings, ...JSON.parse(stored) };
  } catch (e) {
    console.error('[Settings] Failed to parse stored settings:', e);
    return defaultSettings;
  }
}

function createSettingsStore() {
  const { subscribe, set, update } = writable<Settings>(loadSettings());

  return {
    subscribe,
    set: (value: Settings) => {
      set(value);
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('dscode-settings', JSON.stringify(value));
      }
    },
    update: (fn: (value: Settings) => Settings) => {
      update((current) => {
        const updated = fn(current);
        if (typeof localStorage !== 'undefined') {
          localStorage.setItem('dscode-settings', JSON.stringify(updated));
        }
        return updated;
      });
    },
    reset: () => {
      if (typeof localStorage !== 'undefined') {
        localStorage.removeItem('dscode-settings');
      }
      set(defaultSettings);
    },
  };
}

export const settingsStore = createSettingsStore();
