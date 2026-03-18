<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ActivityBar from './components/ActivityBar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import EditorArea from './components/EditorArea.svelte';
  import PanelArea from './components/PanelArea.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import ResizeHandle from './components/ResizeHandle.svelte';
  import { preloadCriticalModules, restoreWorkspaceState, cacheWorkspaceState } from './lib/preload';
  import { enableWindowPersistence } from './lib/window-persistence';
  import { settingsStore } from './lib/settings-store';
  import ToastContainer from './components/ToastContainer.svelte';
  import { initializeSession } from './stores/session';

  $: theme = $settingsStore.theme.colorTheme;

  // Lazy load overlay components (they start hidden anyway)
  let CommandPalette: any = null;
  let QuickOpen: any = null;
  let SymbolSearch: any = null;
  let ResourceMetrics: any = null;
  let ExtensionGallery: any = null;
  let SettingsModal: any = null;
  let componentsLoaded = false;

  let sidebarVisible = true;
  let panelVisible = true;
  let commandPaletteVisible = false;
  let quickOpenVisible = false;
  let symbolSearchVisible = false;
  let metricsVisible = false;
  let galleryVisible = false;
  let settingsVisible = false;

  // Resizable panel widths (use defaults first, then load from localStorage)
  let sidebarWidth = 250;
  let panelHeight = 200;

  // Load saved sizes asynchronously to not block initial render
  if (typeof localStorage !== 'undefined') {
    setTimeout(() => {
      const savedSidebarWidth = localStorage.getItem('sidebarWidth');
      const savedPanelHeight = localStorage.getItem('panelHeight');
      if (savedSidebarWidth) sidebarWidth = Math.max(150, parseInt(savedSidebarWidth));
      if (savedPanelHeight) panelHeight = Math.max(100, parseInt(savedPanelHeight));
    }, 0);
  }

  function handleSidebarResize(event: CustomEvent) {
    const newWidth = sidebarWidth + event.detail.delta;
    sidebarWidth = Math.max(150, Math.min(600, newWidth));
    localStorage.setItem('sidebarWidth', sidebarWidth.toString());
  }

  function handlePanelResize(event: CustomEvent) {
    const newHeight = panelHeight - event.detail.delta; // Subtract because panel grows upward
    panelHeight = Math.max(100, Math.min(600, newHeight));
    localStorage.setItem('panelHeight', panelHeight.toString());
  }

  async function handleKeydown(e: KeyboardEvent) {
    // Ensure components are loaded before showing them
    if (!componentsLoaded) {
      await loadOverlayComponents();
    }

    // Command Palette (Ctrl+Shift+P or Cmd+Shift+P)
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'P') {
      e.preventDefault();
      commandPaletteVisible = true;
      quickOpenVisible = false;
    }

    // Quick Open (Ctrl+P or Cmd+P)
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'p') {
      e.preventDefault();
      quickOpenVisible = true;
      commandPaletteVisible = false;
      symbolSearchVisible = false;
    }

    // Symbol Search (Ctrl+T or Cmd+T)
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 't') {
      e.preventDefault();
      symbolSearchVisible = true;
      commandPaletteVisible = false;
      quickOpenVisible = false;
    }

    // Toggle Sidebar (Ctrl+B or Cmd+B)
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'b') {
      e.preventDefault();
      sidebarVisible = !sidebarVisible;
    }

    // Toggle Panel (Ctrl+J or Cmd+J)
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'j') {
      e.preventDefault();
      panelVisible = !panelVisible;
    }

    // Focus File Explorer (Ctrl+Shift+E or Cmd+Shift+E)
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'E') {
      e.preventDefault();
      sidebarVisible = true;
      window.dispatchEvent(new CustomEvent('focusExplorer'));
    }

    // Focus Search (Ctrl+Shift+F or Cmd+Shift+F)
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'F') {
      e.preventDefault();
      sidebarVisible = true;
      window.dispatchEvent(new CustomEvent('focusSearch'));
    }

    // Focus Git (Ctrl+Shift+G or Cmd+Shift+G)
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'G') {
      e.preventDefault();
      sidebarVisible = true;
      window.dispatchEvent(new CustomEvent('focusGit'));
    }

    // Resource Metrics (Ctrl+Shift+M or Cmd+Shift+M)
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'M') {
      e.preventDefault();
      metricsVisible = !metricsVisible;
    }

    // Extension Gallery (Ctrl+Shift+X or Cmd+Shift+X)
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'X') {
      e.preventDefault();
      galleryVisible = !galleryVisible;
    }

    // Settings (Ctrl+, or Cmd+,)
    if ((e.ctrlKey || e.metaKey) && e.key === ',') {
      e.preventDefault();
      settingsVisible = true;
    }

    // Close All Editors (Ctrl+K W or Cmd+K W)
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      // Wait for second key press
      const handleSecondKey = (e2: KeyboardEvent) => {
        if (e2.key === 'w') {
          e2.preventDefault();
          window.dispatchEvent(new CustomEvent('closeAllEditors'));
        }
        window.removeEventListener('keydown', handleSecondKey);
      };
      window.addEventListener('keydown', handleSecondKey);
      setTimeout(() => window.removeEventListener('keydown', handleSecondKey), 1000);
    }
  }

  function handleToggleSidebar() {
    sidebarVisible = !sidebarVisible;
  }

  function handleTogglePanel() {
    panelVisible = !panelVisible;
  }

  async function handleOpenExtensions() {
    if (!componentsLoaded) {
      await loadOverlayComponents();
    }
    galleryVisible = true;
  }

  async function handleOpenSettings() {
    if (!componentsLoaded) {
      await loadOverlayComponents();
    }
    settingsVisible = true;
  }

  async function loadOverlayComponents() {
    if (componentsLoaded) return;

    const [cp, qo, ss, rm, eg, sm] = await Promise.all([
      import('./components/CommandPalette.svelte'),
      import('./components/QuickOpen.svelte'),
      import('./components/SymbolSearch.svelte'),
      import('./components/ResourceMetrics.svelte'),
      import('./components/ExtensionGallery.svelte'),
      import('./components/SettingsModal.svelte')
    ]);

    CommandPalette = cp.default;
    QuickOpen = qo.default;
    SymbolSearch = ss.default;
    ResourceMetrics = rm.default;
    ExtensionGallery = eg.default;
    SettingsModal = sm.default;
    componentsLoaded = true;
  }

  onMount(() => {
    // Initialize session manager (handles extension state and events)
    initializeSession().catch(err => {
      console.error('Failed to initialize session:', err);
    });

    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('toggleSidebar', handleToggleSidebar);
    window.addEventListener('togglePanel', handleTogglePanel);
    window.addEventListener('openExtensions', handleOpenExtensions);
    window.addEventListener('openSettings', handleOpenSettings);

    // Enable window persistence for instant reopening
    enableWindowPersistence().catch(err => {
      console.error('Failed to enable window persistence:', err);
    });

    // Restore workspace state instantly if available
    const cachedState = restoreWorkspaceState();
    if (cachedState) {
      sidebarWidth = Math.max(150, cachedState.sidebarWidth || sidebarWidth);
      panelHeight = Math.max(100, cachedState.panelHeight || panelHeight);
      sidebarVisible = cachedState.sidebarVisible ?? sidebarVisible;
      panelVisible = cachedState.panelVisible ?? panelVisible;
    }

    // Load overlay components after initial render
    setTimeout(loadOverlayComponents, 50);

    // Preload heavy modules in background during idle time
    preloadCriticalModules();

    // Defer extension host initialization to not block UI
    setTimeout(() => {
      invoke('start_extension_host')
        .then(() => console.log('Extension host started'))
        .catch(error => console.error('Failed to start extension host:', error));
    }, 200);

    // Cache state periodically
    const cacheInterval = setInterval(() => {
      cacheWorkspaceState({
        sidebarWidth,
        panelHeight,
        sidebarVisible,
        panelVisible
      });
    }, 5000);

    return () => clearInterval(cacheInterval);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
    window.removeEventListener('toggleSidebar', handleToggleSidebar);
    window.removeEventListener('togglePanel', handleTogglePanel);
    window.removeEventListener('openExtensions', handleOpenExtensions);
    window.removeEventListener('openSettings', handleOpenSettings);
  });
</script>

<main class="app theme-{theme}">
  <div class="layout">
    <ActivityBar bind:sidebarVisible />

    {#if sidebarVisible}
      <div class="sidebar-container" style="width: {sidebarWidth}px;">
        <Sidebar />
      </div>
      <ResizeHandle direction="horizontal" on:resize={handleSidebarResize} />
    {/if}

    <div class="main-content">
      <div class="editor-container">
        <EditorArea />
      </div>

      {#if panelVisible}
        <ResizeHandle direction="vertical" on:resize={handlePanelResize} />
        <div class="panel-container" style="height: {panelHeight}px;">
          <PanelArea />
        </div>
      {/if}
    </div>
  </div>

  <StatusBar />

  <ToastContainer />

  {#if componentsLoaded}
    <svelte:component
      this={CommandPalette}
      visible={commandPaletteVisible}
      onClose={() => (commandPaletteVisible = false)}
    />

    <svelte:component
      this={QuickOpen}
      visible={quickOpenVisible}
      onClose={() => (quickOpenVisible = false)}
    />

    <svelte:component
      this={SymbolSearch}
      visible={symbolSearchVisible}
      onClose={() => (symbolSearchVisible = false)}
    />

    <svelte:component
      this={ExtensionGallery}
      visible={galleryVisible}
      onClose={() => (galleryVisible = false)}
    />

    <svelte:component
      this={SettingsModal}
      visible={settingsVisible}
      onClose={() => (settingsVisible = false)}
    />

    <svelte:component this={ResourceMetrics} bind:visible={metricsVisible} />
  {/if}
</main>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background-color: var(--color-bg);
  }

  .layout {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar-container {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .main-content {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .panel-container {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
