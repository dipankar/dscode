import { invoke } from '@tauri-apps/api/core';

export function useFileWatchers() {
  const watchedFiles = new Set<string>();
  let fileWatchUnlisten: (() => void) | null = null;

  async function startFileWatch(path: string) {
    if (watchedFiles.has(path)) return;
    watchedFiles.add(path);
    try {
      await invoke('watch_file', { path });
    } catch (error) {
      watchedFiles.delete(path);
      console.warn('[FileWatch] Failed to watch file:', path, error);
    }
  }

  async function stopFileWatch(path: string) {
    if (!watchedFiles.has(path)) return;
    watchedFiles.delete(path);
    try {
      await invoke('unwatch_file', { path });
    } catch (error) {
      console.warn('[FileWatch] Failed to unwatch file:', path, error);
    }
  }

  function stopAllFileWatches() {
    for (const path of watchedFiles) {
      try {
        invoke('unwatch_file', { path });
      } catch {
        // Ignore errors during cleanup
      }
    }
    watchedFiles.clear();
  }

  function setWatchUnlisten(fn: (() => void) | null) {
    fileWatchUnlisten = fn;
  }

  function cleanup() {
    if (fileWatchUnlisten) {
      fileWatchUnlisten();
      fileWatchUnlisten = null;
    }
    stopAllFileWatches();
  }

  return {
    watchedFiles,
    startFileWatch,
    stopFileWatch,
    stopAllFileWatches,
    setWatchUnlisten,
    cleanup,
  };
}
