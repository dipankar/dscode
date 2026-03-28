import { writable } from 'svelte/store';

/**
 * STATE MACHINE: DocumentLifecycle
 *
 * Tracks the lifecycle state of an open document in the editor.
 *
 * State Diagram:
 *
 *   Closed ──► Opening ──► Open ──► Dirty ──► Saving ──► Open
 *     ▲           │                   ▲          │          │
 *     │   (error) │                   │  (fail)  │          │
 *     │           ▼                   │          ▼          │
 *     │        Closed                 └───── Dirty         │
 *     │                                                     │
 *     │                        Closing                      │
 *     │                     ▲         ▲                     │
 *     │                     │         │                     │
 *     │              (from Open) (from Dirty, force)        │
 *     └─────────────────────┴─────────┘                     │
 *                                                           │
 *     Closed ◄────────── Closing ◄──────────────────────────┘
 *
 * Transitions:
 *   Closed   -> Opening   (openFile() called, loading content)
 *   Opening  -> Open      (content loaded, tab created, model set)
 *   Opening  -> Closed    (load failed, file not found)
 *   Open     -> Dirty     (user edits content, model change detected)
 *   Dirty    -> Saving    (save initiated, content being written to disk)
 *   Saving   -> Open      (save completed successfully, file on disk matches editor)
 *   Saving   -> Dirty     (save failed, content still modified in memory)
 *   Open     -> Closing   (closeTab() called on clean file)
 *   Dirty    -> Closing   (closeTab(force=true) called, discarding changes)
 *   Closing  -> Closed    (Monaco model disposed, tab removed)
 *
 * Concurrency:
 *   Svelte store updates are synchronous within their update callbacks.
 *   No races possible — this is purely UI state.
 *   However, the save operation is async (IPC to backend). Between initiating
 *   save and receiving confirmation, the user can continue editing, which
 *   transitions Saving -> Dirty (the save result then applies to the old content,
 *   and the document remains Dirty with the new unsaved changes).
 *
 * Interruption Table:
 * ┌──────────┬────────────────────────────────────────────────────────────────┐
 * │ State    │ What happens if app crashes or frontend reloads               │
 * ├──────────┼────────────────────────────────────────────────────────────────┤
 * │ Closed   │ Safe. No data to lose.                                        │
 * │ Opening  │ No data loss. File content is on disk. Reopen after restart.  │
 * │ Open     │ No data loss. Content matches disk (clean state).             │
 * │ Dirty    │ DATA LOSS. Unsaved changes in Monaco model are lost.          │
 * │          │ The file on disk has the last saved version.                  │
 * │          │ TODO: Consider auto-save or recovery file mechanism.          │
 * │ Saving   │ UNCERTAIN. Save IPC may or may not have completed.           │
 * │          │ File on disk may have old or new content.                     │
 * │          │ On restart: file reflects whatever was last written to disk.  │
 * │ Closing  │ Monaco model being disposed. No data loss (content already   │
 * │          │ saved or user chose to discard).                              │
 * └──────────┴────────────────────────────────────────────────────────────────┘
 */
export type DocumentState = 'Closed' | 'Opening' | 'Open' | 'Dirty' | 'Saving' | 'Closing';

/** Backward-compatible helper: returns true if document has unsaved changes. */
export function isDocumentDirty(state: DocumentState): boolean {
  return state === 'Dirty' || state === 'Saving';
}

export interface OpenFile {
  path: string;
  content: string;
  state: DocumentState;
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

/** Derive tab isDirty from the OpenFile state for backward compatibility. */
function deriveTabDirty(tabs: EditorTab[], path: string, fileState: DocumentState): EditorTab[] {
  const dirty = isDocumentDirty(fileState);
  return tabs.map((t) => (t.path === path ? { ...t, isDirty: dirty } : t));
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

        state.openFiles.set(path, { path, content, state: 'Open', language });

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
        const file = state.openFiles.get(tab.path);
        const fileState = file?.state ?? 'Closed';

        if (!force && isDocumentDirty(fileState)) {
          return state;
        }

        // Transition to Closing
        if (file) {
          state.openFiles.set(tab.path, { ...file, state: 'Closing' });
        }

        // Transition to Closed: dispose model and remove
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
        if (file && (file.state === 'Open' || file.state === 'Saving')) {
          state.openFiles.set(path, { ...file, state: 'Dirty' });
        }

        state.tabs = deriveTabDirty(state.tabs, path, 'Dirty');

        return state;
      });
    },

    markClean: (path: string) => {
      update((state) => {
        const file = state.openFiles.get(path);
        if (file && (file.state === 'Dirty' || file.state === 'Saving')) {
          state.openFiles.set(path, { ...file, state: 'Open' });
        }

        state.tabs = deriveTabDirty(state.tabs, path, 'Open');

        return state;
      });
    },

    markSaving: (path: string) => {
      update((state) => {
        const file = state.openFiles.get(path);
        if (file && file.state === 'Dirty') {
          state.openFiles.set(path, { ...file, state: 'Saving' });
        }

        state.tabs = deriveTabDirty(state.tabs, path, 'Saving');

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

      const dirtyFiles = Array.from(state!.openFiles.entries()).filter(([_, file]) =>
        isDocumentDirty(file.state)
      );

      for (const [path, file] of dirtyFiles) {
        try {
          // Transition to Saving
          update((s) => {
            const f = s.openFiles.get(path);
            if (f && f.state === 'Dirty') s.openFiles.set(path, { ...f, state: 'Saving' });
            s.tabs = deriveTabDirty(s.tabs, path, 'Saving');
            return s;
          });

          await saveFn(path, file.content);

          // Transition to Open on success
          update((s) => {
            const f = s.openFiles.get(path);
            if (f) s.openFiles.set(path, { ...f, state: 'Open' });
            s.tabs = deriveTabDirty(s.tabs, path, 'Open');
            return s;
          });
        } catch (error) {
          // Transition back to Dirty on failure
          update((s) => {
            const f = s.openFiles.get(path);
            if (f && f.state === 'Saving') s.openFiles.set(path, { ...f, state: 'Dirty' });
            s.tabs = deriveTabDirty(s.tabs, path, 'Dirty');
            return s;
          });
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
