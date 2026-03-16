<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ActivityBar from './components/ActivityBar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import EditorArea from './components/EditorArea.svelte';
  import PanelArea from './components/PanelArea.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import CommandPalette from './components/CommandPalette.svelte';
  import QuickOpen from './components/QuickOpen.svelte';
  import SymbolSearch from './components/SymbolSearch.svelte';
  import ResizeHandle from './components/ResizeHandle.svelte';
  import ResourceMetrics from './components/ResourceMetrics.svelte';

  let sidebarVisible = true;
  let panelVisible = true;
  let commandPaletteVisible = false;
  let quickOpenVisible = false;
  let symbolSearchVisible = false;
  let metricsVisible = false;

  // Resizable panel widths (load from localStorage or use defaults)
  let sidebarWidth = parseInt(localStorage.getItem('sidebarWidth') || '250');
  let panelHeight = parseInt(localStorage.getItem('panelHeight') || '200');

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

  function handleKeydown(e: KeyboardEvent) {
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

    // Resource Metrics (Ctrl+Shift+M or Cmd+Shift+M)
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'M') {
      e.preventDefault();
      metricsVisible = !metricsVisible;
      console.log('Toggled metrics visibility:', metricsVisible);
    }
  }

  function handleToggleSidebar() {
    sidebarVisible = !sidebarVisible;
  }

  function handleTogglePanel() {
    panelVisible = !panelVisible;
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('toggleSidebar', handleToggleSidebar);
    window.addEventListener('togglePanel', handleTogglePanel);

    // Start extension host (non-blocking)
    invoke('start_extension_host')
      .then(() => console.log('Extension host started'))
      .catch(error => console.error('Failed to start extension host:', error));
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
    window.removeEventListener('toggleSidebar', handleToggleSidebar);
    window.removeEventListener('togglePanel', handleTogglePanel);
  });
</script>

<main class="app">
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

  <CommandPalette
    visible={commandPaletteVisible}
    onClose={() => (commandPaletteVisible = false)}
  />

  <QuickOpen
    visible={quickOpenVisible}
    onClose={() => (quickOpenVisible = false)}
  />

  <SymbolSearch
    visible={symbolSearchVisible}
    onClose={() => (symbolSearchVisible = false)}
  />

  <ResourceMetrics bind:visible={metricsVisible} />
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
