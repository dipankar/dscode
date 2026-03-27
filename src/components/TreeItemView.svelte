<script lang="ts">
  import type { TreeNode } from './ExtensionView.svelte';

  export let node: TreeNode;
  export let depth: number;
  export let onItemClick: (node: TreeNode) => void;
  export let isItemSelected: (node: TreeNode) => boolean = () => false;

  function getLabel(target: TreeNode): string {
    const item = target.item;
    if (!item.label) return 'Unnamed';
    if (typeof item.label === 'string') return item.label;
    return item.label.label;
  }

  function getItemKey(target: TreeNode): string {
    const item = target.item;
    if (item.id) return item.id;
    return getLabel(target);
  }

  function getRenderKey(target: TreeNode, index: number): string {
    return `${getItemKey(target)}::${index}`;
  }
</script>

<div class="tree-item-wrapper">
  <div
    class="tree-item"
    class:selected={isItemSelected(node)}
    style="padding-left: {depth * 12 + 8}px"
    data-tree-item={getItemKey(node)}
    title={node.item.tooltip ?? getLabel(node)}
    on:click|stopPropagation={() => onItemClick(node)}
    role="treeitem"
    tabindex="0"
    aria-selected={isItemSelected(node)}
    on:keydown={(e) => e.key === 'Enter' && onItemClick(node)}
  >
    <span class="icon">
      {#if node.item.collapsibleState === 1}
        ▸
      {:else if node.item.collapsibleState === 2}
        ▾
      {/if}
    </span>
    <span class="label">{getLabel(node)}</span>
    {#if node.item.description}
      <span class="description">{node.item.description}</span>
    {/if}
  </div>

  {#if node.item.collapsibleState === 2 && node.children}
    {#each node.children as child, idx (getRenderKey(child, idx))}
      <svelte:self node={child} depth={depth + 1} {onItemClick} {isItemSelected} />
    {/each}
  {/if}
</div>

<style>
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
    background-color: var(--color-bg-tertiary, var(--color-surface-hover));
  }

  .tree-item.selected {
    background-color: var(--color-bg-tertiary, var(--color-surface-focus));
    color: var(--color-text, inherit);
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
