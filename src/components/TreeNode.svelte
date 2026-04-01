<script lang="ts">
  import { isNodeExpanded as checkNodeExpanded, type FileNode } from '../stores/workspace';
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
  } from 'lucide-svelte';
  import ContextMenu from './ContextMenu.svelte';

  export let node: FileNode;
  export let selectedFile: string | null;
  export let onFileClick: (node: FileNode) => void;
  export let depth: number;

  $: isSelected = selectedFile === node.path;
  $: isExpanded = checkNodeExpanded(node);
  $: hasChildren = node.children && node.children.length > 0;

  let contextMenuVisible = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let contextMenuContext: {
    explorer_focused: boolean;
    resource_path: string;
    resource_extension?: string;
    custom: {
      explorerResourceIsDirectory: boolean;
      explorerResourceIsFile: boolean;
    };
  } = {
    explorer_focused: true,
    resource_path: '',
    custom: {
      explorerResourceIsDirectory: false,
      explorerResourceIsFile: false,
    },
  };

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuContext = {
      explorer_focused: true,
      resource_path: node.path,
      resource_extension:
        node.node_type === 'file' && node.name.includes('.')
          ? node.name.split('.').pop()
          : undefined,
      custom: {
        explorerResourceIsDirectory: node.node_type === 'directory',
        explorerResourceIsFile: node.node_type === 'file',
      },
    };
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
  }

  function getIconComponent(node: FileNode) {
    if (node.node_type === 'directory') {
      return isExpanded ? FolderOpen : Folder;
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
</script>

<div class="tree-node" style="padding-left: {depth * 12}px">
  <button
    class="node-item"
    class:selected={isSelected}
    class:directory={node.node_type === 'directory'}
    role="treeitem"
    aria-selected={isSelected}
    on:click={() => onFileClick(node)}
    on:contextmenu={handleContextMenu}
  >
    <span class="node-icon">
      <svelte:component this={getIconComponent(node)} size={16} />
    </span>
    <span class="node-name">{node.name}</span>
  </button>

  {#if node.node_type === 'directory' && isExpanded && hasChildren}
    <div class="node-children">
      {#each node.children || [] as child}
        <svelte:self
          node={child}
          {selectedFile}
          {onFileClick}
          depth={depth + 1}
        />
      {/each}
    </div>
  {/if}
</div>

<ContextMenu
  visible={contextMenuVisible}
  x={contextMenuX}
  y={contextMenuY}
  location="explorer/context"
  context={contextMenuContext}
  onClose={closeContextMenu}
/>

<style>
  .tree-node {
    user-select: none;
  }

  .node-item {
    width: 100%;
    padding: 4px 8px;
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    font-size: 13px;
  }

  .node-item:hover {
    background-color: var(--color-bg-tertiary);
  }

  .node-item.selected {
    background-color: var(--editor-selection);
  }

  .node-item.directory {
    font-weight: 500;
  }

  .node-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .node-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
