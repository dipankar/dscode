<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { workspaceStore } from '../stores/workspace';
  import { editorStore } from '../stores/editor';
  import type { FileNode } from '../stores/workspace';
  import TreeNode from './TreeNode.svelte';
  import { FolderOpen } from 'lucide-svelte';

  let fileTree: FileNode[] = [];
  let selectedFile: string | null = null;

  $: {
    fileTree = $workspaceStore.fileTree;
    selectedFile = $workspaceStore.selectedFile;
  }

  onMount(async () => {
    // Auto-load current directory on startup
    const currentDir = await invoke('get_app_version'); // Placeholder for getting cwd
    // We'll implement proper workspace loading next
  });

  async function openFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Open Folder',
    });

    if (selected && typeof selected === 'string') {
      workspaceStore.setRootPath(selected);
      await loadWorkspace(selected);
    }
  }

  async function loadWorkspace(path: string) {
    try {
      const tree = await invoke<FileNode[]>('read_directory_tree', {
        path,
        maxDepth: 3,
      });
      workspaceStore.setFileTree(tree);
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

  async function handleFileClick(node: FileNode) {
    if (node.node_type === 'file') {
      workspaceStore.selectFile(node.path);
      await openFile(node.path);
    } else {
      workspaceStore.toggleDirectory(node.path);
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

  function renderFileTree(nodes: FileNode[], depth: number = 0) {
    return nodes;
  }
</script>

<div class="sidebar">
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
      <div class="tree-view">
        {#each fileTree as node}
          <TreeNode {node} {selectedFile} onFileClick={handleFileClick} onRefresh={refreshWorkspace} depth={0} />
        {/each}
      </div>
    {/if}
  </div>
</div>

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
    overflow-y: auto;
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
    color: white;
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .empty-state button:hover {
    background-color: var(--color-accent-hover);
  }

  .tree-view {
    padding: 4px 0;
  }
</style>
