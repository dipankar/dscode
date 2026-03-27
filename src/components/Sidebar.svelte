<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { workspaceStore } from '../stores/workspace';
  import { editorStore } from '../stores/editor';
  import { addWorkspaceFolder } from '../stores/session';
  import VirtualList from './VirtualList.svelte';
  import type { VirtualListOptions } from './VirtualList.svelte';
  import type { VirtualTreeRow } from '../lib/composables/useVirtualTree';
  import type { FileNode } from '../stores/workspace';
  import {
    Folder,
    FolderOpen,
    File,
    FileText,
    FileJson,
    FileCode,
    Palette,
    Globe,
    Settings,
    ChevronRight,
    ChevronDown,
    Loader,
  } from 'lucide-svelte';
  import { activeActivity, type ActivityId } from '../lib/activity-store';
  import { WindowEventName } from '../lib/contracts/events';
  import ContextMenu from './ContextMenu.svelte';

  let selectedFile: string | null = null;
  let currentActivity: ActivityId = 'explorer';

  const panelImportCache = new Map<string, Promise<any>>();

  function loadPanel(activity: string): Promise<any> {
    let promise = panelImportCache.get(activity);
    if (promise) return promise;

    const loaders: Record<string, () => Promise<any>> = {
      debug: () => import('./DebugView.svelte'),
      scm: () => import('./GitView.svelte'),
      search: () => import('./SearchView.svelte'),
    };

    const loader =
      loaders[activity] ??
      (extensionViews.some((v) => v.id === activity)
        ? () => import('./ExtensionView.svelte')
        : null);

    if (!loader) return Promise.reject(new Error(`Unknown panel: ${activity}`));

    promise = loader();
    panelImportCache.set(activity, promise);
    return promise;
  }

  interface ExtensionViewInfo {
    id: string;
    title: string;
  }

  let extensionViews: ExtensionViewInfo[] = [];
  let refreshWorkspaceListener: (() => void) | null = null;

  $: fileTree = $workspaceStore.fileTree;
  $: selectedFile = $workspaceStore.selectedFile;

  const unsubscribeActiveActivity = activeActivity.subscribe((activity) => {
    currentActivity = activity;
  });

  let expandedPaths = new Set<string>();
  let flatRows: VirtualTreeRow[] = [];

  $: {
    const newFlatRows: VirtualTreeRow[] = [];
    function walk(items: FileNode[], depth: number) {
      for (const node of items) {
        newFlatRows.push({
          id: node.path,
          node: node,
          depth,
        });

        if (node.node_type === 'directory' && expandedPaths.has(node.path) && node.children) {
          walk(node.children, depth + 1);
        }
      }
    }
    walk(fileTree, 0);
    flatRows = newFlatRows;
  }

  onMount(async () => {
    const handleRefreshWorkspace = () => {
      void refreshWorkspace();
    };

    window.addEventListener(WindowEventName.refreshWorkspace, handleRefreshWorkspace);
    refreshWorkspaceListener = () =>
      window.removeEventListener(WindowEventName.refreshWorkspace, handleRefreshWorkspace);

    await loadExtensionViews();
  });

  onDestroy(() => {
    unsubscribeActiveActivity();
    refreshWorkspaceListener?.();
  });

  async function loadExtensionViews() {
    try {
      const contributions = await invoke<any[]>('get_extension_contributions');

      for (const contrib of contributions) {
        const contributes = contrib.contributes;
        if (!contributes) continue;

        if (contributes.views && typeof contributes.views === 'object') {
          for (const [, views] of Object.entries(contributes.views)) {
            for (const view of views as any[]) {
              extensionViews.push({
                id: view.id,
                title: view.name || contrib.extension_name,
              });
            }
          }
        }
      }

      extensionViews = extensionViews;
    } catch (error) {
      console.error('Failed to load extension views:', error);
    }
  }

  async function openFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Open Folder',
    });

    if (selected && typeof selected === 'string') {
      await addWorkspaceFolder(selected);
      workspaceStore.setRootPath(selected);
      await loadWorkspace(selected);
    }
  }

  async function loadWorkspace(path: string) {
    try {
      const tree = await invoke<FileNode[]>('read_directory', { path });
      workspaceStore.setFileTree(
        tree.map((node) => ({
          ...node,
          isLoaded: node.node_type === 'directory' ? false : undefined,
        }))
      );
    } catch (error) {
      console.error('Failed to load workspace:', error);
    }
  }

  async function refreshWorkspace() {
    const rootPath = $workspaceStore.rootPath;
    if (rootPath) {
      await loadWorkspace(rootPath);
    }
  }

  async function handleRowClick(row: VirtualTreeRow) {
    if (row.node.node_type === 'directory') {
      await toggleDirectory(row.node);
    } else {
      workspaceStore.selectFile(row.node.path);
      await openFile(row.node.path);
    }
  }

  async function toggleDirectory(node: FileNode) {
    const path = node.path;

    if (expandedPaths.has(path)) {
      expandedPaths.delete(path);
      expandedPaths = expandedPaths;
      return;
    }

    expandedPaths.add(path);
    expandedPaths = expandedPaths;

    if (!node.isLoaded && !node.isLoading) {
      workspaceStore.expandDirectory(path);
    }
  }

  async function openFile(path: string) {
    try {
      const content = await invoke<string>('read_file', { path });
      const language = await invoke<string>('get_file_language', { path });
      editorStore.openFile(path, content, language);
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  }

  function getIconComponent(node: FileNode) {
    if (node.node_type === 'directory') {
      return expandedPaths.has(node.path) ? FolderOpen : Folder;
    }

    const ext = '.' + node.name.split('.').pop();
    const iconMap: Record<string, any> = {
      '.rs': FileCode,
      '.js': FileCode,
      '.ts': FileCode,
      '.svelte': FileCode,
      '.json': FileJson,
      '.md': FileText,
      '.toml': Settings,
      '.css': Palette,
      '.html': Globe,
    };

    return iconMap[ext] || File;
  }

  let contextMenuVisible = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let contextMenuNode: FileNode | null = null;

  function handleRowContextMenu(e: MouseEvent, row: VirtualTreeRow) {
    e.preventDefault();
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuNode = row.node;
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
  }

  const virtualListOptions: VirtualListOptions = { itemHeight: 24, overscan: 8 };
</script>

<div class="sidebar">
  {#if currentActivity === 'explorer'}
    <div class="sidebar-header">
      <h3>EXPLORER</h3>
      <button class="open-folder-btn" on:click={openFolder} title="Open Folder">
        <FolderOpen size={18} />
      </button>
    </div>

    <div class="sidebar-content">
      {#if fileTree.length === 0}
        <div class="empty-state">
          <p>No folder opened</p>
          <button on:click={openFolder}>Open Folder</button>
        </div>
      {:else}
        <VirtualList items={flatRows} options={virtualListOptions} let:item={row}>
          <button
            class="tree-row"
            class:selected={row.node.node_type === 'file' && selectedFile === row.node.path}
            style="padding-left: {row.depth * 12 + 8}px"
            on:click={() => handleRowClick(row)}
            on:contextmenu|stopPropagation={(e) => handleRowContextMenu(e, row)}
          >
            {#if row.node.node_type === 'directory'}
              <span class="tree-toggle">
                {#if row.node.isLoading}
                  <Loader size={14} class="spinning" />
                {:else if expandedPaths.has(row.node.path)}
                  <ChevronDown size={14} />
                {:else}
                  <ChevronRight size={14} />
                {/if}
              </span>
            {:else}
              <span class="tree-toggle tree-toggle-file"></span>
            {/if}
            <span class="tree-icon">
              <svelte:component this={getIconComponent(row.node)} size={16} />
            </span>
            <span class="tree-name">{row.node.name}</span>
          </button>
        </VirtualList>
      {/if}
    </div>
  {:else if currentActivity === 'debug'}
    {#await loadPanel('debug')}
      <div class="panel-loader">Loading...</div>
    {:then module}
      <svelte:component this={module.default} />
    {:catch}
      <div class="panel-loader">Failed to load panel</div>
    {/await}
  {:else if currentActivity === 'scm'}
    {#await loadPanel('scm')}
      <div class="panel-loader">Loading...</div>
    {:then module}
      <svelte:component this={module.default} />
    {:catch}
      <div class="panel-loader">Failed to load panel</div>
    {/await}
  {:else if currentActivity === 'search'}
    {#await loadPanel('search')}
      <div class="panel-loader">Loading...</div>
    {:then module}
      <svelte:component this={module.default} />
    {:catch}
      <div class="panel-loader">Failed to load panel</div>
    {/await}
  {:else}
    {#each extensionViews as extView}
      {#if currentActivity === extView.id}
        {#await loadPanel(extView.id)}
          <div class="panel-loader">Loading...</div>
        {:then module}
          <svelte:component this={module.default} viewId={extView.id} title={extView.title} />
        {:catch}
          <div class="panel-loader">Failed to load panel</div>
        {/await}
      {/if}
    {/each}
  {/if}
</div>

<ContextMenu
  visible={contextMenuVisible}
  x={contextMenuX}
  y={contextMenuY}
  location="explorer/context"
  context={{
    explorer_focused: true,
    resource_path: contextMenuNode?.path ?? '',
    resource_extension: contextMenuNode?.name.includes('.')
      ? contextMenuNode.name.split('.').pop()
      : undefined,
    custom: {
      explorerResourceIsDirectory: contextMenuNode?.node_type === 'directory',
      explorerResourceIsFile: contextMenuNode?.node_type === 'file',
    },
  }}
  onClose={closeContextMenu}
/>

<style>
  .sidebar {
    width: 100%;
    height: 100%;
    background-color: var(--color-bg-secondary);
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--color-border);
  }

  .sidebar-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .sidebar-header h3 {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.5px;
  }

  .open-folder-btn {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 16px;
    padding: 4px;
  }

  .open-folder-btn:hover {
    color: var(--color-text);
  }

  .sidebar-content {
    flex: 1;
    overflow: hidden;
  }

  .sidebar-content > :global(.virtual-list-container) {
    height: 100%;
  }

  .empty-state {
    padding: 20px;
    text-align: center;
    color: var(--color-text-secondary);
  }

  .empty-state p {
    margin-bottom: 12px;
  }

  .empty-state button {
    background-color: var(--color-accent);
    color: var(--color-text-on-accent);
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .empty-state button:hover {
    background-color: var(--color-accent-hover);
  }

  .tree-row {
    display: flex;
    align-items: center;
    width: 100%;
    height: 24px;
    padding: 0 8px;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
    font-size: 13px;
    box-sizing: border-box;
  }

  .tree-row:hover {
    background-color: var(--color-bg-tertiary, var(--color-surface-hover));
  }

  .tree-row.selected {
    background-color: var(--editor-selection, var(--color-selection-bg));
  }

  .tree-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    margin-right: 2px;
  }

  .tree-toggle-file {
    width: 16px;
  }

  .tree-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    margin-right: 4px;
  }

  .tree-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .panel-loader {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    color: var(--color-text-secondary);
    font-size: 12px;
  }
</style>
