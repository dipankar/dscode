<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { editorStore } from '../stores/editor';
  import { workspaceStore } from '../stores/workspace';
  import type { FileNode } from '../stores/workspace';

  export let visible: boolean = false;
  export let onClose: () => void;

  let searchQuery = '';
  let filteredFiles: FileNode[] = [];
  let selectedIndex = 0;
  let inputElement: HTMLInputElement;
  let allFiles: FileNode[] = [];

  $: rootPath = $workspaceStore.rootPath;
  $: fileTree = $workspaceStore.fileTree;

  // Flatten file tree to get all files
  function flattenFileTree(nodes: FileNode[]): FileNode[] {
    let files: FileNode[] = [];
    for (const node of nodes) {
      if (node.node_type === 'file') {
        files.push(node);
      }
      if (node.children && node.children.length > 0) {
        files = files.concat(flattenFileTree(node.children));
      }
    }
    return files;
  }

  $: {
    allFiles = flattenFileTree(fileTree);
  }

  $: {
    if (searchQuery.trim() === '') {
      // Show recently opened files when no query
      const recentFiles = allFiles.filter(file =>
        $editorStore.openFiles.has(file.path)
      );
      filteredFiles = recentFiles.length > 0 ? recentFiles : allFiles.slice(0, 20);
    } else {
      const query = searchQuery.toLowerCase();
      filteredFiles = allFiles
        .filter(file => {
          const fileName = file.name.toLowerCase();
          const filePath = file.path.toLowerCase();
          return fileName.includes(query) || filePath.includes(query);
        })
        .sort((a, b) => {
          // Fuzzy matching score
          const aName = a.name.toLowerCase();
          const bName = b.name.toLowerCase();
          const aStarts = aName.startsWith(query);
          const bStarts = bName.startsWith(query);

          if (aStarts && !bStarts) return -1;
          if (!aStarts && bStarts) return 1;

          // Prioritize shorter names (more relevant)
          return aName.length - bName.length;
        })
        .slice(0, 50); // Limit results
    }
    selectedIndex = 0;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;

    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, filteredFiles.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredFiles[selectedIndex]) {
          openFile(filteredFiles[selectedIndex]);
        }
        break;
    }
  }

  async function openFile(file: FileNode) {
    try {
      // Check if already open
      if ($editorStore.openFiles.has(file.path)) {
        const tab = $editorStore.tabs.find(t => t.path === file.path);
        if (tab) {
          editorStore.switchTab(tab.id);
          onClose();
          return;
        }
      }

      // Read file content
      const content = await invoke<string>('read_file', { path: file.path });
      const language = await invoke<string>('get_file_language', { path: file.path });

      // Open in editor
      editorStore.openFile(file.path, content, language);
      workspaceStore.selectFile(file.path);
      onClose();
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  }

  function getRelativePath(filePath: string): string {
    if (!rootPath) return filePath;
    return filePath.replace(rootPath + '/', '');
  }

  function getFileIcon(fileName: string): string {
    if (fileName.endsWith('.rs')) return '🦀';
    if (fileName.endsWith('.ts')) return '📘';
    if (fileName.endsWith('.js')) return '📜';
    if (fileName.endsWith('.svelte')) return '🔶';
    if (fileName.endsWith('.json')) return '📋';
    if (fileName.endsWith('.md')) return '📝';
    if (fileName.endsWith('.toml')) return '⚙️';
    if (fileName.endsWith('.css')) return '🎨';
    if (fileName.endsWith('.html')) return '🌐';
    return '📄';
  }

  $: if (visible && inputElement) {
    inputElement.focus();
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

{#if visible}
  <div class="quick-open-overlay" on:click={onClose}>
    <div class="quick-open" on:click|stopPropagation>
      <div class="search-container">
        <input
          bind:this={inputElement}
          bind:value={searchQuery}
          type="text"
          placeholder="Search files by name..."
          class="search-input"
        />
      </div>

      <div class="files-list">
        {#if filteredFiles.length === 0}
          <div class="no-results">
            {#if allFiles.length === 0}
              No files in workspace. Open a folder to get started.
            {:else}
              No files match "{searchQuery}"
            {/if}
          </div>
        {:else}
          {#each filteredFiles as file, index}
            <button
              class="file-item"
              class:selected={index === selectedIndex}
              on:click={() => openFile(file)}
              on:mouseenter={() => (selectedIndex = index)}
            >
              <span class="file-icon">{getFileIcon(file.name)}</span>
              <div class="file-info">
                <span class="file-name">{file.name}</span>
                <span class="file-path">{getRelativePath(file.path)}</span>
              </div>
            </button>
          {/each}
        {/if}
      </div>

      <div class="quick-open-footer">
        <span class="hint">↑↓ to navigate • Enter to open • Esc to close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .quick-open-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--modal-overlay-bg);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 80px;
    z-index: 1000;
  }

  .quick-open {
    background-color: var(--modal-bg);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    width: 600px;
    max-height: 500px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  .search-container {
    padding: 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .search-input {
    width: 100%;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 14px;
    outline: none;
  }

  .search-input:focus {
    border-color: var(--color-accent);
  }

  .files-list {
    overflow-y: auto;
    max-height: 380px;
  }

  .file-item {
    width: 100%;
    padding: 10px 16px;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .file-item:hover,
  .file-item.selected {
    background-color: var(--editor-selection);
  }

  .file-icon {
    font-size: 16px;
    flex-shrink: 0;
  }

  .file-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .file-name {
    font-size: 13px;
    font-weight: 500;
  }

  .file-path {
    font-size: 11px;
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .no-results {
    padding: 20px;
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 13px;
  }

  .quick-open-footer {
    padding: 8px 16px;
    border-top: 1px solid var(--color-border);
    background-color: var(--color-bg-tertiary);
  }

  .hint {
    font-size: 11px;
    color: var(--color-text-secondary);
  }
</style>
