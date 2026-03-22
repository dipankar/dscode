<script context="module" lang="ts">
  export interface TreeItem {
    label?: string | { label: string };
    id?: string;
    description?: string;
    tooltip?: string;
    collapsibleState?: number; // 0 = None, 1 = Collapsed, 2 = Expanded
    command?: { title: string; command: string; arguments?: any[] };
    contextValue?: string;
  }

  export interface TreeNode {
    element: any;
    item: TreeItem;
    children?: TreeNode[];
  }
</script>

<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import TreeItemView from './TreeItemView.svelte';

  export let viewId: string;
  export let title: string;

  let rootItems: TreeNode[] = [];
  let loading = true;
  let error: string | null = null;
  let selectedIds = new Set<string>();
  let treeContainer: HTMLDivElement | null = null;
  let revealListener: ((event: Event) => void) | null = null;

  onMount(async () => {
    await loadTreeView();
    await notifyTreeEvent('didChangeVisibility', { visible: true });

    revealListener = (event: Event) => {
      const customEvent = event as CustomEvent<any>;
      const detail = customEvent.detail;
      if (!detail || detail.view_id !== viewId) {
        return;
      }
      void handleReveal(detail);
    };

    window.addEventListener('extensionTreeReveal', revealListener as EventListener);
  });

  onDestroy(() => {
    if (revealListener) {
      window.removeEventListener('extensionTreeReveal', revealListener as EventListener);
    }
    void notifyTreeEvent('didChangeVisibility', { visible: false });
  });

  function normalizeNode(raw: any): TreeNode {
    const element = raw?.element ?? raw?.item ?? null;
    const item = (raw?.item ?? raw?.element ?? {}) as TreeItem;

    const node: TreeNode = {
      element,
      item,
      children: Array.isArray(raw?.children) ? raw.children.map(normalizeNode) : undefined,
    };

    return node;
  }

  function getItemKey(node: TreeNode): string {
    const item = node.item;
    if (item.id) return item.id;
    if (typeof item.label === 'string') return item.label;
    if (item.label && typeof item.label === 'object' && item.label.label) {
      return item.label.label;
    }
    return 'tree-item';
  }

  function getRenderKey(node: TreeNode, index: number): string {
    return `${getItemKey(node)}::${index}`;
  }

  function isItemSelected(node: TreeNode): boolean {
    return selectedIds.has(getItemKey(node));
  }

  async function notifyTreeEvent(event: string, details: Record<string, any> = {}) {
    try {
      await invoke('extension_tree_notify_event', {
        viewId,
        event,
        element: details.element,
        selection: details.selection,
        visible: details.visible,
      });
    } catch (err) {
      console.error(`[ExtensionView] Failed to notify tree event ${event}:`, err);
    }
  }

  async function loadTreeView() {
    try {
      loading = true;
      error = null;

      const children = await invoke<any[]>('extension_tree_get_children', {
        viewId,
        element: null,
      });

      rootItems = Array.isArray(children) ? children.map(normalizeNode) : [];
      selectedIds = new Set();
      loading = false;
    } catch (err) {
      console.error(`Failed to load tree view ${viewId}:`, err);
      error = String(err);
      loading = false;
    }
  }

  async function loadChildren(node: TreeNode): Promise<TreeNode[]> {
    try {
      const children = await invoke<any[]>('extension_tree_get_children', {
        viewId,
        element: node.element,
      });
      return Array.isArray(children) ? children.map(normalizeNode) : [];
    } catch (err) {
      console.error(`[ExtensionView] Failed to load children for node:`, err);
      return [];
    }
  }

  function refreshTree() {
    rootItems = [...rootItems];
  }

  async function handleItemClick(node: TreeNode) {
    const key = getItemKey(node);
    selectedIds = new Set([key]);
    await notifyTreeEvent('didChangeSelection', { selection: [node.element] });

    if (node.item.command) {
      try {
        await invoke('extension_execute_command', {
          command: node.item.command.command,
          args: node.item.command.arguments || [],
        });
      } catch (err) {
        console.error('Failed to execute command:', err);
      }
    }

    if (node.item.collapsibleState === 1 || node.item.collapsibleState === 2) {
      await toggleExpanded(node);
    }

    refreshTree();
  }

  async function toggleExpanded(node: TreeNode) {
    if (node.item.collapsibleState === 1) {
      node.item.collapsibleState = 2;
      node.children = await loadChildren(node);
      await notifyTreeEvent('didExpand', { element: node.element });
    } else if (node.item.collapsibleState === 2) {
      node.item.collapsibleState = 1;
      node.children = [];
      await notifyTreeEvent('didCollapse', { element: node.element });
    }

    refreshTree();
  }

  function elementsEqual(a: any, b: any): boolean {
    try {
      return JSON.stringify(a) === JSON.stringify(b);
    } catch (error) {
      return false;
    }
  }

  function findPath(nodes: TreeNode[], key: string, targetElement: any, path: TreeNode[] = []): TreeNode[] | null {
    for (const node of nodes) {
      const currentPath = [...path, node];
      if (getItemKey(node) === key || elementsEqual(node.element, targetElement)) {
        return currentPath;
      }

      if (node.children && node.children.length > 0) {
        const found = findPath(node.children, key, targetElement, currentPath);
        if (found) {
          return found;
        }
      }
    }
    return null;
  }

  async function handleReveal(detail: any) {
    const element = detail?.element;
    if (!element) {
      return;
    }

    let treeItem: any = detail?.item;
    if (!treeItem) {
      try {
        treeItem = await invoke<any>('extension_tree_get_item', { viewId, element });
      } catch (err) {
        console.warn('[ExtensionView] Failed to resolve tree item for reveal:', err);
      }
    }

    const targetNode = normalizeNode({ element, item: treeItem ?? element });
    const key = getItemKey(targetNode);
    const path = findPath(rootItems, key, element);
    if (!path) {
      console.warn(`[ExtensionView] Unable to reveal tree item (not found):`, element);
      return;
    }

    const options = detail?.options || {};

    path.slice(0, -1).forEach((ancestor) => {
      if (ancestor.item.collapsibleState === 1) {
        ancestor.item.collapsibleState = 2;
      }
    });

    const target = path[path.length - 1];
    const expandTarget =
      options.expand === true || (typeof options.expand === 'number' && options.expand > 0);
    if (expandTarget && target.item.collapsibleState === 1) {
      target.item.collapsibleState = 2;
      if (!target.children || target.children.length === 0) {
        target.children = await loadChildren(target);
      }
    }

    if (options.select !== false) {
      const targetKey = getItemKey(target);
      selectedIds = new Set([targetKey]);
      await notifyTreeEvent('didChangeSelection', { selection: [target.element] });
    }

    refreshTree();

    await tick();

    if (treeContainer) {
      const keyValue = getItemKey(target);
      const escapedKey = typeof CSS !== 'undefined' && typeof CSS.escape === 'function'
        ? CSS.escape(keyValue)
        : keyValue.replace(/"/g, '\\"');
      const selector = `[data-tree-item="${escapedKey}"]`;
      const elementNode = treeContainer.querySelector(selector) as HTMLElement | null;
      elementNode?.scrollIntoView({ block: 'nearest' });
    }
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
      <div class="tree-view" bind:this={treeContainer}>
        {#each rootItems as node, idx (getRenderKey(node, idx))}
          <TreeItemView
            {node}
            depth={0}
            onItemClick={handleItemClick}
            isItemSelected={isItemSelected}
          />
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
