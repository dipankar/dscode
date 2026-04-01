<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import ActivityBar from './components/ActivityBar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import EditorArea from './components/EditorArea.svelte';
  import PanelArea from './components/PanelArea.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import ResizeHandle from './components/ResizeHandle.svelte';
  import { settingsStore } from './lib/settings-store';
  import './lib/keybinding-manager';
  import ToastContainer from './components/ToastContainer.svelte';
  import WindowPromptModal from './components/WindowPromptModal.svelte';
  import QuickPickModal from './components/QuickPickModal.svelte';
  import InputBoxModal from './components/InputBoxModal.svelte';
  import LocalInputBoxModal from './components/LocalInputBoxModal.svelte';
  import DiffViewerOverlay from './components/DiffViewerOverlay.svelte';
  import ErrorOverlay from './components/ErrorOverlay.svelte';
  import { windowPromptStore, localInputBoxStore } from './stores/windowPrompt';
  import { diffStore } from './stores/diffViewer';
  import { quickPickStore } from './stores/quickPick';
  import { inputBoxStore } from './stores/inputBox';
  import { appErrorStore, setAppError } from './stores/appError';
  import { createAppShellController, createDefaultAppShellState } from './lib/app-shell/controller';
  import { workspaceStore } from './stores/workspace';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { sessionLoading } from './stores/session';

  $: theme = $settingsStore.theme.colorTheme;

  $: {
    const rootPath = $workspaceStore.rootPath;
    if (rootPath) {
      const folderName = rootPath.split('/').pop() || rootPath.split('\\').pop() || rootPath;
      const title = `${folderName} - DSCode`;
      document.title = title;
      getCurrentWindow().setTitle(title).catch(() => {});
    } else {
      document.title = 'DSCode';
      getCurrentWindow().setTitle('DSCode').catch(() => {});
    }
  }

  let shellState = createDefaultAppShellState();

  // Track session initialization lifecycle to hide the loading screen.
  // sessionLoading starts false, goes true during init, then false when done.
  // We wait until it has been true at least once before dismissing the splash.
  let hasSessionLoaded = false;

  function hideLoadingScreen() {
    const el = document.querySelector('.loading-screen');
    if (el && !el.classList.contains('loading-screen--fade-out')) {
      el.classList.add('loading-screen--fade-out');
      el.addEventListener('transitionend', () => el.remove(), { once: true });
      // Fallback removal in case transitionend never fires (e.g. prefers-reduced-motion)
      setTimeout(() => { if (el.parentNode) el.remove(); }, 1000);
    }
  }

  $: {
    if ($sessionLoading) {
      hasSessionLoaded = true;
    }
    if (hasSessionLoaded && !$sessionLoading) {
      hideLoadingScreen();
    }
  }

  const appShellController = createAppShellController({
    getState: () => shellState,
    setState: (nextState) => {
      shellState = nextState;
    },
  });

  function handleSidebarResize(event: CustomEvent) {
    appShellController.resizeSidebar(event.detail.delta);
  }

  function handlePanelResize(event: CustomEvent) {
    appShellController.resizePanel(event.detail.delta);
  }

  onMount(() => {
    appShellController.initialize();

    // Safety: hide loading screen after 10s even if session init stalls
    setTimeout(hideLoadingScreen, 10000);

    const handleWindowError = (event: ErrorEvent) => {
      setAppError(event.error ?? new Error(event.message));
    };

    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      setAppError(event.reason);
    };

    window.addEventListener('error', handleWindowError);
    window.addEventListener('unhandledrejection', handleUnhandledRejection);

    return () => {
      window.removeEventListener('error', handleWindowError);
      window.removeEventListener('unhandledrejection', handleUnhandledRejection);
    };
  });

  onDestroy(() => {
    appShellController.destroy();
  });
</script>

<main class="app theme-{theme}">
  <div class="layout">
    <ActivityBar bind:sidebarVisible={shellState.sidebarVisible} />

    {#if shellState.sidebarVisible}
      <div
        class="sidebar-container"
        data-command-focus="sidebar"
        style="width: {shellState.sidebarWidth}px;"
      >
        <Sidebar />
      </div>
      <ResizeHandle direction="horizontal" on:resize={handleSidebarResize} />
    {/if}

    <div class="main-content">
      <div class="editor-container" data-command-focus="editor">
        <EditorArea />
      </div>

      {#if shellState.panelVisible}
        <ResizeHandle direction="vertical" on:resize={handlePanelResize} />
        <div
          class="panel-container"
          data-command-focus="panel"
          style="height: {shellState.panelHeight}px;"
        >
          <PanelArea />
        </div>
      {/if}
    </div>
  </div>

  <StatusBar />

  <ToastContainer />

  {#if $windowPromptStore}
    <WindowPromptModal prompt={$windowPromptStore} />
  {/if}

  {#if $quickPickStore}
    <QuickPickModal request={$quickPickStore} />
  {/if}

  {#if $inputBoxStore}
    <InputBoxModal request={$inputBoxStore} />
  {/if}

  {#if $localInputBoxStore}
    <LocalInputBoxModal request={$localInputBoxStore} />
  {/if}

  {#if $diffStore}
    <DiffViewerOverlay />
  {/if}

  {#if $appErrorStore}
    <ErrorOverlay error={$appErrorStore} />
  {/if}

  {#if shellState.componentsLoaded}
    <svelte:component
      this={shellState.CommandPalette}
      visible={shellState.commandPaletteVisible}
      onClose={() => appShellController.setOverlayVisibility('commandPaletteVisible', false)}
    />

    <svelte:component
      this={shellState.QuickOpen}
      visible={shellState.quickOpenVisible}
      onClose={() => appShellController.setOverlayVisibility('quickOpenVisible', false)}
    />

    <svelte:component
      this={shellState.SymbolSearch}
      visible={shellState.symbolSearchVisible}
      onClose={() => appShellController.setOverlayVisibility('symbolSearchVisible', false)}
    />

    <svelte:component
      this={shellState.ExtensionGallery}
      visible={shellState.galleryVisible}
      onClose={() => appShellController.setOverlayVisibility('galleryVisible', false)}
    />

    <svelte:component
      this={shellState.SettingsModal}
      visible={shellState.settingsVisible}
      onClose={() => appShellController.setOverlayVisibility('settingsVisible', false)}
    />

    <svelte:component this={shellState.ResourceMetrics} bind:visible={shellState.metricsVisible} />
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
