<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import TreeItemView from './TreeItemView.svelte';

  export let viewId: string;
  export let title: string;

  export interface TreeItem {
    label?: string | { label: string };
    id?: string;
    description?: string;
    tooltip?: string;
    collapsibleState?: number; // 0 = None, 1 = Collapsed, 2 = Expanded
    command?: { title: string; command: string; arguments?: any[] };
    contextValue?: string;
    children?: TreeItem[];
  }

  let rootItems: TreeItem[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    await loadTreeView();
  });

  async function loadTreeView() {
    try {
      loading = true;
      error = null;

      // Request root children from the extension's tree data provider
      const children = await invoke<TreeItem[]>('extension_tree_get_children', {
        viewId,
        element: null, // null means get root items
      });

      rootItems = children || [];
      loading = false;
    } catch (err) {
      console.error(`Failed to load tree view ${viewId}:`, err);
      error = String(err);
      loading = false;
    }
  }

  async function loadChildren(item: TreeItem): Promise<TreeItem[]> {
    try {
      const children = await invoke<TreeItem[]>('extension_tree_get_children', {
        viewId,
        element: item,
      });
      return children || [];
    } catch (err) {
      console.error(`Failed to load children for ${item.label}:`, err);
      return [];
    }
  }

  async function handleItemClick(item: TreeItem) {
    // If item has a command, execute it
    if (item.command) {
      try {
        await invoke('extension_execute_command', {
          command: item.command.command,
          args: item.command.arguments || [],
        });
      } catch (err) {
        console.error('Failed to execute command:', err);
      }
    }

    // If item is collapsible, toggle expansion
    if (item.collapsibleState === 1 || item.collapsibleState === 2) {
      await toggleExpanded(item);
    }
  }

  async function toggleExpanded(item: TreeItem) {
    // Toggle collapsible state
    if (item.collapsibleState === 1) {
      // Was collapsed, expand it
      item.collapsibleState = 2;
      item.children = await loadChildren(item);
    } else if (item.collapsibleState === 2) {
      // Was expanded, collapse it
      item.collapsibleState = 1;
      item.children = [];
    }

    rootItems = rootItems; // Trigger reactivity
  }
</script>

<div class="extension-view">
  <div class="view-header">
    <h3>{title}</h3>
  </div>

  <div class="view-content">
    {#if loading}
      <div class="loading-state">
        <p>Loading...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <p>Failed to load view</p>
        <small>{error}</small>
      </div>
    {:else if rootItems.length === 0}
      <div class="empty-state">
        <p>No items</p>
      </div>
    {:else}
      <div class="tree-view">
        {#each rootItems as item (item.id || item.label)}
          <TreeItemView {item} depth={0} onItemClick={handleItemClick} />
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .extension-view {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-secondary);
  }

  .view-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .view-header h3 {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .view-content {
    flex: 1;
    overflow-y: auto;
  }

  .loading-state,
  .error-state,
  .empty-state {
    padding: 20px;
    text-align: center;
    color: var(--color-text-secondary);
  }

  .error-state {
    color: var(--color-error);
  }

  .error-state small {
    display: block;
    margin-top: 8px;
    font-size: 10px;
  }

  .tree-view {
    padding: 4px 0;
  }

  .tree-item-wrapper {
    width: 100%;
  }

  .tree-item {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    cursor: pointer;
    color: var(--color-text);
    user-select: none;
  }

  .tree-item:hover {
    background-color: var(--color-hover);
  }

  .icon {
    width: 16px;
    margin-right: 4px;
    font-size: 10px;
    color: var(--color-text-secondary);
  }

  .label {
    flex: 1;
    font-size: 13px;
  }

  .description {
    margin-left: 8px;
    font-size: 11px;
    color: var(--color-text-secondary);
  }
</style>
