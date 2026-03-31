<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { editorStore } from '../stores/editor';
  import { workspaceStore } from '../stores/workspace';
  import { focusTrap } from '../lib/focus-trap';

  interface FileSearchResult {
    name: string;
    path: string;
    node_type: string;
  }

  export let visible: boolean = false;
  export let onClose: () => void;

  let searchQuery = '';
  let filteredFiles: FileSearchResult[] = [];
  let selectedIndex = 0;
  let inputElement: HTMLInputElement;
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;
  let isSearching = false;
  let modalContainer: HTMLDivElement;

  function trapFocus(e: KeyboardEvent) {
    if (e.key !== 'Tab' || !modalContainer) return;
    const focusable = Array.from(
      modalContainer.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      )
    ).filter((el) => el.offsetParent !== null);
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  $: rootPath = $workspaceStore.rootPath;

  async function searchFiles(query: string) {
    if (!rootPath || query.trim() === '') {
      filteredFiles = [];
      return;
    }

    isSearching = true;
    try {
      const results = await invoke<FileSearchResult[]>('search_files', {
        query,
        rootPath,
        maxResults: 50,
      });
      filteredFiles = results.filter((r: FileSearchResult) => r.node_type === 'file');
      selectedIndex = 0;
    } catch (error) {
      console.error('Failed to search files:', error);
      filteredFiles = [];
    } finally {
      isSearching = false;
    }
  }

  $: {
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    if (visible && searchQuery.trim() !== '') {
      searchTimeout = setTimeout(() => searchFiles(searchQuery), 150);
    } else {
      filteredFiles = [];
    }
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

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  async function openFile(file: FileSearchResult) {
    try {
      if ($editorStore.openFiles.has(file.path)) {
        const tab = $editorStore.tabs.find((t) => t.path === file.path);
        if (tab) {
          editorStore.switchTab(tab.id);
          onClose();
          return;
        }
      }

      const content = await invoke<string>('read_file', { path: file.path });
      const language = await invoke<string>('get_file_language', { path: file.path });

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
    if (fileName.endsWith('.rs')) return '\u{1F980}';
    if (fileName.endsWith('.ts')) return '\u{1F4D8}';
    if (fileName.endsWith('.js')) return '\u{1F4DC}';
    if (fileName.endsWith('.svelte')) return '\u{1F536}';
    if (fileName.endsWith('.json')) return '\u{1F4CB}';
    if (fileName.endsWith('.md')) return '\u{1F4DD}';
    if (fileName.endsWith('.toml')) return '\u{2699}\u{FE0F}';
    if (fileName.endsWith('.css')) return '\u{1F3A8}';
    if (fileName.endsWith('.html')) return '\u{1F310}';
    return '\u{1F4C4}';
  }

  $: if (visible && inputElement) {
    searchQuery = '';
    filteredFiles = [];
    selectedIndex = 0;
    setTimeout(() => inputElement.focus(), 0);
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('keydown', trapFocus);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
    window.removeEventListener('keydown', trapFocus);
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
  });
</script>

{#if visible}
  <div
    class="quick-open-overlay"
    on:click={handleOverlayClick}
    on:keydown={handleKeydown}
    role="presentation"
    tabindex="-1"
  >
    <div
      bind:this={modalContainer}
      class="quick-open"
      role="dialog"
      aria-modal="true"
      aria-label="Quick open"
      use:focusTrap
    >
      <div class="search-container">
        <input
          bind:this={inputElement}
          bind:value={searchQuery}
          type="text"
          placeholder="Search files by name..."
          class="search-input"
          aria-label="Search files"
          aria-controls="quick-open-list"
        />
      </div>

      <div class="files-list" id="quick-open-list" role="listbox" aria-label="Files">
        {#if isSearching}
          <div class="no-results">Searching...</div>
        {:else if filteredFiles.length === 0}
          <div class="no-results">
            {#if searchQuery.trim() === ''}
              Type to search files
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
              role="option"
              aria-selected={index === selectedIndex}
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
        <span class="hint">Type to search • Up/Down to navigate • Enter to open • Esc to close</span
        >
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
    box-shadow: var(--shadow-md);
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
