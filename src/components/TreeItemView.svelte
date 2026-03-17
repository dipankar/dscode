<script lang="ts">
  import type { TreeItem } from './ExtensionView.svelte';

  export let item: TreeItem;
  export let depth: number;
  export let onItemClick: (item: TreeItem) => void;

  function getLabel(item: TreeItem): string {
    if (!item.label) return 'Unnamed';
    if (typeof item.label === 'string') return item.label;
    return item.label.label;
  }
</script>

<div class="tree-item-wrapper">
  <div
    class="tree-item"
    style="padding-left: {depth * 12 + 8}px"
    on:click|stopPropagation={() => onItemClick(item)}
    role="button"
    tabindex="0"
    on:keydown={(e) => e.key === 'Enter' && onItemClick(item)}
  >
    <span class="icon">
      {#if item.collapsibleState === 1}
        ▸
      {:else if item.collapsibleState === 2}
        ▾
      {/if}
    </span>
    <span class="label">{getLabel(item)}</span>
    {#if item.description}
      <span class="description">{item.description}</span>
    {/if}
  </div>

  {#if item.collapsibleState === 2 && item.children}
    {#each item.children as child (child.id || child.label)}
      <svelte:self item={child} depth={depth + 1} {onItemClick} />
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
