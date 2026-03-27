import { get } from 'svelte/store';
import { editorStore } from '../../stores/editor';

export function useAutoSave() {
  let autoSaveTimeout: number | null = null;

  function scheduleAutoSave(
    path: string,
    delayMs: number,
    saveFn: (path: string) => Promise<void>
  ) {
    cancelAutoSave();

    autoSaveTimeout = window.setTimeout(async () => {
      autoSaveTimeout = null;
      const state = get(editorStore);
      const content = state.openFiles.get(path)?.content;
      if (content !== undefined) {
        try {
          await saveFn(path);
          editorStore.markClean(path);
        } catch (error) {
          console.warn('[AutoSave] Failed:', error);
        }
      }
    }, delayMs);
  }

  function cancelAutoSave() {
    if (autoSaveTimeout !== null) {
      clearTimeout(autoSaveTimeout);
      autoSaveTimeout = null;
    }
  }

  return {
    scheduleAutoSave,
    cancelAutoSave,
  };
}
