<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
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
  } from 'lucide-svelte';

  export let node: FileNode;
  export let selectedFile: string | null;
  export let onFileClick: (node: FileNode) => void;
  export let onRefresh: () => void;
  export let depth: number;

  $: isSelected = selectedFile === node.path;
  $: isExpanded = node.isExpanded ?? false;
  $: hasChildren = node.children && node.children.length > 0;

  let contextMenuVisible = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let renamingMode = false;
  let newName = '';

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
  }

  function startRename() {
    newName = node.name;
    renamingMode = true;
    closeContextMenu();
  }

  async function confirmRename() {
    if (newName && newName !== node.name) {
      try {
        const parentPath = node.path.substring(0, node.path.lastIndexOf('/'));
        const newPath = `${parentPath}/${newName}`;
        await invoke('rename_path', { oldPath: node.path, newPath });
        onRefresh();
      } catch (error) {
        console.error('Failed to rename:', error);
        alert(`Failed to rename: ${error}`);
      }
    }
    renamingMode = false;
  }

  async function handleDelete() {
    const confirmMsg = node.node_type === 'directory'
      ? `Delete folder "${node.name}" and all its contents?`
      : `Delete file "${node.name}"?`;

    if (confirm(confirmMsg)) {
      try {
        if (node.node_type === 'directory') {
          await invoke('delete_folder', { path: node.path });
        } else {
          await invoke('delete_file', { path: node.path });
        }
        onRefresh();
      } catch (error) {
        console.error('Failed to delete:', error);
        alert(`Failed to delete: ${error}`);
      }
    }
    closeContextMenu();
  }

  async function handleNewFile() {
    const fileName = prompt('Enter file name:');
    if (fileName) {
      try {
        const newFilePath = node.node_type === 'directory'
          ? `${node.path}/${fileName}`
          : `${node.path.substring(0, node.path.lastIndexOf('/'))}/${fileName}`;
        await invoke('create_file', { path: newFilePath });
        onRefresh();
      } catch (error) {
        console.error('Failed to create file:', error);
        alert(`Failed to create file: ${error}`);
      }
    }
    closeContextMenu();
  }

  async function handleNewFolder() {
    const folderName = prompt('Enter folder name:');
    if (folderName) {
      try {
        const newFolderPath = node.node_type === 'directory'
          ? `${node.path}/${folderName}`
          : `${node.path.substring(0, node.path.lastIndexOf('/'))}/${folderName}`;
        await invoke('create_folder', { path: newFolderPath });
        onRefresh();
      } catch (error) {
        console.error('Failed to create folder:', error);
        alert(`Failed to create folder: ${error}`);
      }
    }
    closeContextMenu();
  }

  async function copyPath() {
    try {
      await navigator.clipboard.writeText(node.path);
      console.log('Copied path to clipboard:', node.path);
    } catch (error) {
      console.error('Failed to copy path:', error);
    }
    closeContextMenu();
  }

  async function copyRelativePath() {
    try {
      // Get just the filename for now
      // In a real implementation, you'd get the relative path from workspace root
      const relativePath = node.name;
      await navigator.clipboard.writeText(relativePath);
      console.log('Copied relative path to clipboard:', relativePath);
    } catch (error) {
      console.error('Failed to copy relative path:', error);
    }
    closeContextMenu();
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

<svelte:window on:click={closeContextMenu} />

<div class="tree-node" style="padding-left: {depth * 12}px">
  <button
    class="node-item"
    class:selected={isSelected}
    class:directory={node.node_type === 'directory'}
    on:click={() => onFileClick(node)}
    on:contextmenu={handleContextMenu}
  >
    <span class="node-icon">
      <svelte:component this={getIconComponent(node)} size={16} />
    </span>
    {#if renamingMode}
      <input
        type="text"
        class="rename-input"
        bind:value={newName}
        on:blur={confirmRename}
        on:keydown={(e) => {
          if (e.key === 'Enter') confirmRename();
          if (e.key === 'Escape') renamingMode = false;
        }}
        autofocus
      />
    {:else}
      <span class="node-name">{node.name}</span>
    {/if}
  </button>

  {#if contextMenuVisible}
    <div class="context-menu" style="left: {contextMenuX}px; top: {contextMenuY}px;">
      {#if node.node_type === 'directory'}
        <button on:click={handleNewFile}>New File</button>
        <button on:click={handleNewFolder}>New Folder</button>
        <div class="context-menu-separator"></div>
      {/if}
      <button on:click={copyPath}>Copy Path</button>
      <button on:click={copyRelativePath}>Copy Relative Path</button>
      <div class="context-menu-separator"></div>
      <button on:click={startRename}>Rename</button>
      <button on:click={handleDelete} class="danger">Delete</button>
    </div>
  {/if}

  {#if node.node_type === 'directory' && isExpanded && hasChildren}
    <div class="node-children">
      {#each node.children || [] as child}
        <svelte:self
          node={child}
          {selectedFile}
          {onFileClick}
          {onRefresh}
          depth={depth + 1}
        />
      {/each}
    </div>
  {/if}
</div>

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

  .node-children {
    /* Children will be indented via parent's depth prop */
  }

  .rename-input {
    flex: 1;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-accent);
    color: var(--color-text);
    padding: 2px 4px;
    font-size: 13px;
    outline: none;
  }

  .context-menu {
    position: fixed;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    padding: 4px 0;
    min-width: 150px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
  }

  .context-menu button {
    width: 100%;
    padding: 6px 12px;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    font-size: 13px;
  }

  .context-menu button:hover {
    background-color: var(--color-bg-tertiary);
  }

  .context-menu button.danger {
    color: #ef4444;
  }

  .context-menu-separator {
    height: 1px;
    background: var(--color-border);
    margin: 4px 0;
  }
</style>
