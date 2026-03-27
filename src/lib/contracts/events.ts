export const TauriEventName = {
  session: 'session-event',
} as const;

export const WindowEventName = {
  focusExplorer: 'focusExplorer',
  focusSearch: 'focusSearch',
  focusGit: 'focusGit',
  closeAllEditors: 'closeAllEditors',
  toggleSidebar: 'toggleSidebar',
  togglePanel: 'togglePanel',
  showCommandPalette: 'showCommandPalette',
  showQuickOpen: 'showQuickOpen',
  showSymbolSearch: 'showSymbolSearch',
  toggleResourceMetrics: 'toggleResourceMetrics',
  openExtensions: 'openExtensions',
  openSettings: 'openSettings',
  refreshWorkspace: 'refreshWorkspace',
  extensionTreeReveal: 'extensionTreeReveal',
  editorDecorations: 'editor-decorations',
  openDiff: 'openDiff',
} as const;

export function dispatchWindowEvent(name: (typeof WindowEventName)[keyof typeof WindowEventName]) {
  window.dispatchEvent(new CustomEvent(name));
}

export function dispatchWindowDetailEvent<T>(
  name: (typeof WindowEventName)[keyof typeof WindowEventName],
  detail: T
) {
  window.dispatchEvent(new CustomEvent(name, { detail }));
}
