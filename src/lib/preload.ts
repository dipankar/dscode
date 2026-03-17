// Preload strategy for near-instant subsequent loads

export async function preloadCriticalModules() {
  // Use requestIdleCallback to preload during idle time
  if ('requestIdleCallback' in window) {
    requestIdleCallback(async () => {
      // Preload Monaco Editor
      const monacoPromise = import('monaco-editor');

      // Preload heavy modules
      const xtermPromise = import('@xterm/xterm');

      // Wait for all in parallel
      await Promise.all([monacoPromise, xtermPromise]);

      console.log('[Preload] Critical modules loaded in background');
    });
  } else {
    // Fallback for older browsers
    setTimeout(async () => {
      await Promise.all([
        import('monaco-editor'),
        import('@xterm/xterm')
      ]);
    }, 1000);
  }
}

// Prefetch resources for next likely action
export function prefetchForEditor() {
  if ('prefetch' in HTMLLinkElement.prototype) {
    const link = document.createElement('link');
    link.rel = 'prefetch';
    link.href = '/monaco-editor-chunk.js'; // Adjust to your chunk name
    document.head.appendChild(link);
  }
}

// Cache workspace state for instant restore
export function cacheWorkspaceState(state: any) {
  try {
    sessionStorage.setItem('dscode-workspace', JSON.stringify(state));
  } catch (e) {
    console.warn('Failed to cache workspace state:', e);
  }
}

export function restoreWorkspaceState(): any {
  try {
    const cached = sessionStorage.getItem('dscode-workspace');
    return cached ? JSON.parse(cached) : null;
  } catch (e) {
    console.warn('Failed to restore workspace state:', e);
    return null;
  }
}
