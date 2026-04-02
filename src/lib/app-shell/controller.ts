import {
  preloadCriticalModules,
  restoreWorkspaceState,
  cacheWorkspaceState,
} from '../preload';
import { enableWindowPersistence } from '../window-persistence';
import { WindowEventName } from '../contracts/events';

type ComponentRef = any;

export interface AppShellState {
  sidebarVisible: boolean;
  panelVisible: boolean;
  commandPaletteVisible: boolean;
  quickOpenVisible: boolean;
  symbolSearchVisible: boolean;
  metricsVisible: boolean;
  galleryVisible: boolean;
  settingsVisible: boolean;
  sidebarWidth: number;
  panelHeight: number;
  componentsLoaded: boolean;
  CommandPalette: ComponentRef | null;
  QuickOpen: ComponentRef | null;
  SymbolSearch: ComponentRef | null;
  ResourceMetrics: ComponentRef | null;
  ExtensionGallery: ComponentRef | null;
  SettingsModal: ComponentRef | null;
}

type ShellStateGetter = () => AppShellState;
type ShellStateSetter = (nextState: AppShellState) => void;
type OverlayVisibilityKey =
  | 'commandPaletteVisible'
  | 'quickOpenVisible'
  | 'symbolSearchVisible'
  | 'metricsVisible'
  | 'galleryVisible'
  | 'settingsVisible';

interface AppShellControllerOptions {
  getState: ShellStateGetter;
  setState: ShellStateSetter;
}

const MIN_SIDEBAR_WIDTH = 150;
const MAX_SIDEBAR_WIDTH = 600;
const MIN_PANEL_HEIGHT = 100;
const MAX_PANEL_HEIGHT = 600;

export function createDefaultAppShellState(): AppShellState {
  return {
    sidebarVisible: true,
    panelVisible: true,
    commandPaletteVisible: false,
    quickOpenVisible: false,
    symbolSearchVisible: false,
    metricsVisible: false,
    galleryVisible: false,
    settingsVisible: false,
    sidebarWidth: 250,
    panelHeight: 200,
    componentsLoaded: false,
    CommandPalette: null,
    QuickOpen: null,
    SymbolSearch: null,
    ResourceMetrics: null,
    ExtensionGallery: null,
    SettingsModal: null,
  };
}

export function createAppShellController({
  getState,
  setState,
}: AppShellControllerOptions) {
  let cacheInterval: number | null = null;

  const updateState = (updater: (current: AppShellState) => AppShellState) => {
    setState(updater(getState()));
  };

  const setOverlayVisibility = (key: OverlayVisibilityKey, visible: boolean) => {
    updateState((state) => ({ ...state, [key]: visible }));
  };

  const clampSidebarWidth = (width: number) =>
    Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, width));

  const clampPanelHeight = (height: number) =>
    Math.max(MIN_PANEL_HEIGHT, Math.min(MAX_PANEL_HEIGHT, height));

  const persistDimensions = (state: AppShellState) => {
    if (typeof localStorage === 'undefined') {
      return;
    }

    localStorage.setItem('sidebarWidth', state.sidebarWidth.toString());
    localStorage.setItem('panelHeight', state.panelHeight.toString());
  };

  const loadPersistedDimensions = () => {
    if (typeof localStorage === 'undefined') {
      return;
    }

    const savedSidebarWidth = localStorage.getItem('sidebarWidth');
    const savedPanelHeight = localStorage.getItem('panelHeight');

    updateState((state) => ({
      ...state,
      sidebarWidth: savedSidebarWidth
        ? clampSidebarWidth(parseInt(savedSidebarWidth, 10))
        : state.sidebarWidth,
      panelHeight: savedPanelHeight
        ? clampPanelHeight(parseInt(savedPanelHeight, 10))
        : state.panelHeight,
    }));
  };

  const applyCachedWorkspaceState = () => {
    const cachedState = restoreWorkspaceState();
    if (!cachedState) {
      return;
    }

    updateState((state) => ({
      ...state,
      sidebarWidth: clampSidebarWidth(cachedState.sidebarWidth || state.sidebarWidth),
      panelHeight: clampPanelHeight(cachedState.panelHeight || state.panelHeight),
      sidebarVisible: cachedState.sidebarVisible ?? state.sidebarVisible,
      panelVisible: cachedState.panelVisible ?? state.panelVisible,
    }));
  };

  const cacheCurrentWorkspaceState = () => {
    const state = getState();
    cacheWorkspaceState({
      sidebarWidth: state.sidebarWidth,
      panelHeight: state.panelHeight,
      sidebarVisible: state.sidebarVisible,
      panelVisible: state.panelVisible,
    });
  };

  async function ensureOverlayComponentsLoaded() {
    if (getState().componentsLoaded) {
      return;
    }

    const [cp, qo, ss, rm, eg, sm] = await Promise.all([
      import('../../components/CommandPalette.svelte'),
      import('../../components/QuickOpen.svelte'),
      import('../../components/SymbolSearch.svelte'),
      import('../../components/ResourceMetrics.svelte'),
      import('../../components/ExtensionGallery.svelte'),
      import('../../components/SettingsModal.svelte'),
    ]);

    updateState((state) => ({
      ...state,
      CommandPalette: cp.default,
      QuickOpen: qo.default,
      SymbolSearch: ss.default,
      ResourceMetrics: rm.default,
      ExtensionGallery: eg.default,
      SettingsModal: sm.default,
      componentsLoaded: true,
    }));
  }

  async function showCommandPalette() {
    await ensureOverlayComponentsLoaded();
    updateState((state) => ({
      ...state,
      commandPaletteVisible: true,
      quickOpenVisible: false,
      symbolSearchVisible: false,
    }));
  }

  async function showQuickOpen() {
    await ensureOverlayComponentsLoaded();
    updateState((state) => ({
      ...state,
      quickOpenVisible: true,
      commandPaletteVisible: false,
      symbolSearchVisible: false,
    }));
  }

  async function showSymbolSearch() {
    await ensureOverlayComponentsLoaded();
    updateState((state) => ({
      ...state,
      symbolSearchVisible: true,
      commandPaletteVisible: false,
      quickOpenVisible: false,
    }));
  }

  async function toggleMetrics() {
    await ensureOverlayComponentsLoaded();
    updateState((state) => ({
      ...state,
      metricsVisible: !state.metricsVisible,
    }));
  }

  async function openExtensions() {
    await ensureOverlayComponentsLoaded();
    setOverlayVisibility('galleryVisible', true);
  }

  async function openSettings() {
    await ensureOverlayComponentsLoaded();
    setOverlayVisibility('settingsVisible', true);
  }

  function toggleSidebar() {
    updateState((state) => ({
      ...state,
      sidebarVisible: !state.sidebarVisible,
    }));
  }

  function togglePanel() {
    updateState((state) => ({
      ...state,
      panelVisible: !state.panelVisible,
    }));
  }

  function handleSidebarEvent() {
    toggleSidebar();
  }

  function handlePanelEvent() {
    togglePanel();
  }

  function revealSidebar() {
    updateState((state) => ({ ...state, sidebarVisible: true }));
  }

  function resizeSidebar(delta: number) {
    updateState((state) => {
      const nextState = {
        ...state,
        sidebarWidth: clampSidebarWidth(state.sidebarWidth + delta),
      };
      persistDimensions(nextState);
      return nextState;
    });
  }

  function resizePanel(delta: number) {
    updateState((state) => {
      const nextState = {
        ...state,
        panelHeight: clampPanelHeight(state.panelHeight - delta),
      };
      persistDimensions(nextState);
      return nextState;
    });
  }

  function initialize() {
    window.addEventListener(WindowEventName.toggleSidebar, handleSidebarEvent);
    window.addEventListener(WindowEventName.togglePanel, handlePanelEvent);
    window.addEventListener(WindowEventName.showCommandPalette, showCommandPalette);
    window.addEventListener(WindowEventName.showQuickOpen, showQuickOpen);
    window.addEventListener(WindowEventName.showSymbolSearch, showSymbolSearch);
    window.addEventListener(WindowEventName.toggleResourceMetrics, toggleMetrics);
    window.addEventListener(WindowEventName.openExtensions, openExtensions);
    window.addEventListener(WindowEventName.openSettings, openSettings);
    window.addEventListener(WindowEventName.focusExplorer, revealSidebar);
    window.addEventListener(WindowEventName.focusSearch, revealSidebar);
    window.addEventListener(WindowEventName.focusGit, revealSidebar);

    enableWindowPersistence().catch((error) => {
      console.error('Failed to enable window persistence:', error);
    });

    loadPersistedDimensions();
    applyCachedWorkspaceState();

    window.setTimeout(() => {
      void ensureOverlayComponentsLoaded();
    }, 50);

    preloadCriticalModules();

    cacheInterval = window.setInterval(cacheCurrentWorkspaceState, 5000);
  }

  function destroy() {
    window.removeEventListener(WindowEventName.toggleSidebar, handleSidebarEvent);
    window.removeEventListener(WindowEventName.togglePanel, handlePanelEvent);
    window.removeEventListener(WindowEventName.showCommandPalette, showCommandPalette);
    window.removeEventListener(WindowEventName.showQuickOpen, showQuickOpen);
    window.removeEventListener(WindowEventName.showSymbolSearch, showSymbolSearch);
    window.removeEventListener(WindowEventName.toggleResourceMetrics, toggleMetrics);
    window.removeEventListener(WindowEventName.openExtensions, openExtensions);
    window.removeEventListener(WindowEventName.openSettings, openSettings);
    window.removeEventListener(WindowEventName.focusExplorer, revealSidebar);
    window.removeEventListener(WindowEventName.focusSearch, revealSidebar);
    window.removeEventListener(WindowEventName.focusGit, revealSidebar);

    if (cacheInterval) {
      window.clearInterval(cacheInterval);
      cacheInterval = null;
    }
  }

  return {
    initialize,
    destroy,
    resizeSidebar,
    resizePanel,
    setOverlayVisibility,
  };
}
