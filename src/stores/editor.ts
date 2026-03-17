import { writable } from 'svelte/store';
import * as monaco from 'monaco-editor';

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
  monacoInstance: monaco.editor.IStandaloneCodeEditor | null;
  autoSaveEnabled: boolean;
  autoSaveDelay: number; // in milliseconds
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

        // Add to open files
        state.openFiles.set(path, { path, content, isDirty: false, language });

        // Add tab if not exists
        if (!state.tabs.find((t) => t.id === id)) {
          state.tabs.push({
            id,
            path,
            label,
            isDirty: false,
            isPinned: false,
          });
        }

        // Set as active
        state.activeTabId = id;

        // Note: Monaco editor will be updated reactively by the EditorArea component

        return state;
      });
    },

    closeTab: (tabId: string, force: boolean = false) => {
      update((state) => {
        const index = state.tabs.findIndex((t) => t.id === tabId);
        if (index === -1) return state;

        const tab = state.tabs[index];

        // Check if file has unsaved changes and not forcing close
        if (!force && tab.isDirty) {
          // Don't close - will be handled by component with confirmation
          return state;
        }

        state.openFiles.delete(tab.path);
        state.tabs.splice(index, 1);

        // If closing active tab, switch to another
        if (state.activeTabId === tabId) {
          if (state.tabs.length > 0) {
            const newActiveTab = state.tabs[Math.max(0, index - 1)];
            state.activeTabId = newActiveTab.id;
            // Note: Monaco editor will be updated reactively by the EditorArea component
          } else {
            state.activeTabId = null;
            // Clear editor when no tabs are open
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
          file.isDirty = true;
        }

        const tab = state.tabs.find((t) => t.path === path);
        if (tab) {
          tab.isDirty = true;
        }

        return state;
      });
    },

    markClean: (path: string) => {
      update((state) => {
        const file = state.openFiles.get(path);
        if (file) {
          file.isDirty = false;
        }

        const tab = state.tabs.find((t) => t.path === path);
        if (tab) {
          tab.isDirty = false;
        }

        return state;
      });
    },

    setMonacoInstance: (instance: monaco.editor.IStandaloneCodeEditor) => {
      update((state) => {
        state.monacoInstance = instance;
        return state;
      });
    },

    updateContent: (path: string, content: string) => {
      update((state) => {
        const file = state.openFiles.get(path);
        if (file) {
          file.content = content;
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
      const unsubscribe = subscribe((s) => { state = s; });

      const dirtyFiles = Array.from(state!.openFiles.entries())
        .filter(([_, file]) => file.isDirty);

      for (const [path, file] of dirtyFiles) {
        try {
          await saveFn(path, file.content);
          update((s) => {
            const f = s.openFiles.get(path);
            if (f) f.isDirty = false;
            const tab = s.tabs.find((t) => t.path === path);
            if (tab) tab.isDirty = false;
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
      const unsubscribe = subscribe((s) => { state = s; });

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
