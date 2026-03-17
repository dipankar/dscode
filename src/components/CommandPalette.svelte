<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { editorStore } from '../stores/editor';
  import { workspaceStore } from '../stores/workspace';

  export let visible: boolean = false;
  export let onClose: () => void;

  interface Command {
    id: string;
    label: string;
    category?: string;
    action: () => void | Promise<void>;
  }

  let searchQuery = '';
  let filteredCommands: Command[] = [];
  let selectedIndex = 0;
  let inputElement: HTMLInputElement;

  const commands: Command[] = [
    {
      id: 'file.save',
      label: 'File: Save',
      category: 'File',
      action: async () => {
        const activeTab = $editorStore.tabs.find(t => t.id === $editorStore.activeTabId);
        if (activeTab && $editorStore.monacoInstance) {
          const content = $editorStore.monacoInstance.getValue();
          await invoke('write_file', { path: activeTab.path, content });
          editorStore.markClean(activeTab.path);
        }
      },
    },
    {
      id: 'file.close',
      label: 'File: Close Editor',
      category: 'File',
      action: () => {
        if ($editorStore.activeTabId) {
          editorStore.closeTab($editorStore.activeTabId);
        }
      },
    },
    {
      id: 'file.closeAll',
      label: 'File: Close All Editors',
      category: 'File',
      action: () => {
        const tabIds = $editorStore.tabs.map(t => t.id);
        tabIds.forEach(id => editorStore.closeTab(id));
      },
    },
    {
      id: 'view.toggleSidebar',
      label: 'View: Toggle Sidebar',
      category: 'View',
      action: () => {
        // Will be implemented via event
        window.dispatchEvent(new CustomEvent('toggleSidebar'));
      },
    },
    {
      id: 'view.togglePanel',
      label: 'View: Toggle Panel',
      category: 'View',
      action: () => {
        window.dispatchEvent(new CustomEvent('togglePanel'));
      },
    },
    {
      id: 'editor.action.formatDocument',
      label: 'Format Document',
      category: 'Editor',
      action: () => {
        if ($editorStore.monacoInstance) {
          $editorStore.monacoInstance.getAction('editor.action.formatDocument')?.run();
        }
      },
    },
    {
      id: 'workbench.action.reloadWindow',
      label: 'Developer: Reload Window',
      category: 'Developer',
      action: () => {
        window.location.reload();
      },
    },
  ];

  $: {
    if (searchQuery.trim() === '') {
      filteredCommands = commands;
    } else {
      const query = searchQuery.toLowerCase();
      filteredCommands = commands
        .filter(cmd =>
          cmd.label.toLowerCase().includes(query) ||
          cmd.category?.toLowerCase().includes(query)
        )
        .sort((a, b) => {
          // Prioritize exact matches
          const aLower = a.label.toLowerCase();
          const bLower = b.label.toLowerCase();
          const aStarts = aLower.startsWith(query);
          const bStarts = bLower.startsWith(query);
          if (aStarts && !bStarts) return -1;
          if (!aStarts && bStarts) return 1;
          return 0;
        });
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
        selectedIndex = Math.min(selectedIndex + 1, filteredCommands.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          executeCommand(filteredCommands[selectedIndex]);
        }
        break;
    }
  }

  async function executeCommand(command: Command) {
    try {
      await command.action();
      onClose();
    } catch (error) {
      console.error('Failed to execute command:', error);
    }
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
  <div class="command-palette-overlay" on:click={onClose}>
    <div class="command-palette" on:click|stopPropagation>
      <div class="search-container">
        <input
          bind:this={inputElement}
          bind:value={searchQuery}
          type="text"
          placeholder="Type a command or search..."
          class="search-input"
        />
      </div>

      <div class="commands-list">
        {#if filteredCommands.length === 0}
          <div class="no-results">No commands found</div>
        {:else}
          {#each filteredCommands as command, index}
            <button
              class="command-item"
              class:selected={index === selectedIndex}
              on:click={() => executeCommand(command)}
              on:mouseenter={() => (selectedIndex = index)}
            >
              <span class="command-label">{command.label}</span>
              {#if command.category}
                <span class="command-category">{command.category}</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <div class="command-palette-footer">
        <span class="hint">↑↓ to navigate • Enter to select • Esc to close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .command-palette-overlay {
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

  .command-palette {
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

  .commands-list {
    overflow-y: auto;
    max-height: 380px;
  }

  .command-item {
    width: 100%;
    padding: 10px 16px;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
  }

  .command-item:hover,
  .command-item.selected {
    background-color: var(--editor-selection);
  }

  .command-label {
    flex: 1;
  }

  .command-category {
    color: var(--color-text-secondary);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .no-results {
    padding: 20px;
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 13px;
  }

  .command-palette-footer {
    padding: 8px 16px;
    border-top: 1px solid var(--color-border);
    background-color: var(--color-bg-tertiary);
  }

  .hint {
    font-size: 11px;
    color: var(--color-text-secondary);
  }
</style>
