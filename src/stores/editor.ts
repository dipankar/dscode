import { writable } from 'svelte/store';

export interface OpenFile {
  path: string;
  content: string;
  isDirty: boolean;
  language: string;
}

export interface EditorTab {
  id: string;
  path: string;
  label: string;
  isDirty: boolean;
  isPinned: boolean;
}

interface EditorState {
  openFiles: Map<string, OpenFile>;
  tabs: EditorTab[];
  activeTabId: string | null;
  monacoInstance: any | null;
  autoSaveEnabled: boolean;
  autoSaveDelay: number;
}

function disposeModelForPath(path: string): void {
  try {
    const monaco = require('monaco-editor');
    const uri = monaco.Uri.parse(`file://${path}`);
    const model = monaco.editor.getModel(uri);
    if (model) {
      model.dispose();
    }
  } catch {
    // Monaco not loaded yet, nothing to dispose
  }
}

function createEditorStore() {
  const { subscribe, set, update } = writable<EditorState>({
    openFiles: new Map(),
    tabs: [],
    activeTabId: null,
    monacoInstance: null,
    autoSaveEnabled: false,
    autoSaveDelay: 1000,
  });

  return {
    subscribe,
    openFile: (path: string, content: string, language: string = 'typescript') => {
      update((state) => {
        const id = path;
        const label = path.split('/').pop() || path;

        state.openFiles.set(path, { path, content, isDirty: false, language });

        if (!state.tabs.find((t) => t.id === id)) {
          state.tabs.push({
            id,
            path,
            label,
            isDirty: false,
            isPinned: false,
          });
        }

        state.activeTabId = id;

        return state;
      });
    },

    closeTab: (tabId: string, force: boolean = false) => {
      update((state) => {
        const index = state.tabs.findIndex((t) => t.id === tabId);
        if (index === -1) return state;

        const tab = state.tabs[index];

        if (!force && tab.isDirty) {
          return state;
        }

        state.openFiles.delete(tab.path);
        state.tabs.splice(index, 1);

        disposeModelForPath(tab.path);

        if (state.activeTabId === tabId) {
          if (state.tabs.length > 0) {
            const newActiveTab = state.tabs[Math.max(0, index - 1)];
            state.activeTabId = newActiveTab.id;
          } else {
            state.activeTabId = null;
            if (state.monacoInstance) {
              state.monacoInstance.setValue('');
            }
          }
        }

        return state;
      });
    },

    switchTab: (tabId: string) => {
      update((state) => {
        const tab = state.tabs.find((t) => t.id === tabId);
        if (!tab) return state;

        state.activeTabId = tabId;

        // Note: Monaco editor will be updated reactively by the EditorArea component

        return state;
      });
    },

    markDirty: (path: string) => {
      update((state) => {
        const file = state.openFiles.get(path);
        if (file) {
          state.openFiles.set(path, { ...file, isDirty: true });
        }

        state.tabs = state.tabs.map((t) => (t.path === path ? { ...t, isDirty: true } : t));

        return state;
      });
    },

    markClean: (path: string) => {
      update((state) => {
        const file = state.openFiles.get(path);
        if (file) {
          state.openFiles.set(path, { ...file, isDirty: false });
        }

        state.tabs = state.tabs.map((t) => (t.path === path ? { ...t, isDirty: false } : t));

        return state;
      });
    },

    setMonacoInstance: (instance: any) => {
      update((state) => {
        state.monacoInstance = instance;
        return state;
      });
    },

    updateContent: (path: string, content: string) => {
      update((state) => {
        const file = state.openFiles.get(path);
        if (file) {
          state.openFiles.set(path, { ...file, content });
        }
        return state;
      });
    },

    setAutoSave: (enabled: boolean) => {
      update((state) => {
        state.autoSaveEnabled = enabled;
        return state;
      });
    },

    setAutoSaveDelay: (delay: number) => {
      update((state) => {
        state.autoSaveDelay = delay;
        return state;
      });
    },

    saveAll: async (saveFn: (path: string, content: string) => Promise<void>) => {
      let state: EditorState;
      const unsubscribe = subscribe((s) => {
        state = s;
      });

      const dirtyFiles = Array.from(state!.openFiles.entries()).filter(([_, file]) => file.isDirty);

      for (const [path, file] of dirtyFiles) {
        try {
          await saveFn(path, file.content);
          update((s) => {
            const f = s.openFiles.get(path);
            if (f) s.openFiles.set(path, { ...f, isDirty: false });
            s.tabs = s.tabs.map((t) => (t.path === path ? { ...t, isDirty: false } : t));
            return s;
          });
        } catch (error) {
          console.error(`Failed to save ${path}:`, error);
        }
      }

      unsubscribe();
    },

    navigateToLine: (line: number, column: number = 1) => {
      let state: EditorState;
      const unsubscribe = subscribe((s) => {
        state = s;
      });

      if (state!.monacoInstance) {
        // Reveal the line in the center
        state!.monacoInstance.revealLineInCenter(line);

        // Set cursor position
        state!.monacoInstance.setPosition({ lineNumber: line, column });

        // Focus the editor
        state!.monacoInstance.focus();

        // Optionally, select the whole line for highlighting
        state!.monacoInstance.setSelection({
          startLineNumber: line,
          startColumn: 1,
          endLineNumber: line,
          endColumn: state!.monacoInstance.getModel()?.getLineMaxColumn(line) || 1,
        });
      }

      unsubscribe();
    },
  };
}

export const editorStore = createEditorStore();
