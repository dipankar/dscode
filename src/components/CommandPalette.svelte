<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { editorStore } from '../stores/editor';
  import { workspaceStore } from '../stores/workspace';
  import { keybindingManager } from '../lib/keybinding-manager';

  export let visible: boolean = false;
  export let onClose: () => void;

  interface CommandInfo {
    id: string;
    label: string;
    category?: string;
    owner: string;
    keybinding?: string;
    when?: string;
  }

  interface Command {
    id: string;
    label: string;
    category?: string;
    keybinding?: string;
    action: () => void | Promise<void>;
  }

  let searchQuery = '';
  let filteredCommands: Command[] = [];
  let selectedIndex = 0;
  let inputElement: HTMLInputElement;
  let allCommands: Command[] = [];

  // Built-in command actions (these execute in the frontend)
  const builtinActions: Record<string, () => void | Promise<void>> = {
    'file.save': async () => {
      const activeTab = $editorStore.tabs.find(t => t.id === $editorStore.activeTabId);
      if (activeTab && $editorStore.monacoInstance) {
        const content = $editorStore.monacoInstance.getValue();
        await invoke('write_file', { path: activeTab.path, content });
        editorStore.markClean(activeTab.path);
      }
    },
    'file.close': () => {
      if ($editorStore.activeTabId) {
        editorStore.closeTab($editorStore.activeTabId);
      }
    },
    'file.closeAll': () => {
      const tabIds = $editorStore.tabs.map(t => t.id);
      tabIds.forEach(id => editorStore.closeTab(id));
    },
    'view.toggleSidebar': () => {
      window.dispatchEvent(new CustomEvent('toggleSidebar'));
    },
    'view.togglePanel': () => {
      window.dispatchEvent(new CustomEvent('togglePanel'));
    },
    'editor.action.formatDocument': () => {
      if ($editorStore.monacoInstance) {
        $editorStore.monacoInstance.getAction('editor.action.formatDocument')?.run();
      }
    },
    'workbench.action.reloadWindow': () => {
      window.location.reload();
    },
  };

  // Load commands from the command registry
  async function loadCommands() {
    try {
      const commandInfos = await invoke<CommandInfo[]>('get_all_commands');

      // Map commands and fetch keybindings
      allCommands = await Promise.all(
        commandInfos.map(async cmdInfo => {
          const keybinding = await keybindingManager.getKeybindingForCommand(cmdInfo.id);

          return {
            id: cmdInfo.id,
            label: cmdInfo.label,
            category: cmdInfo.category,
            keybinding: keybinding || undefined,
            action: async () => {
              // Check if it's a builtin command with a local action
              if (builtinActions[cmdInfo.id]) {
                await builtinActions[cmdInfo.id]();
              } else {
                // Execute via extension host
                await executeExtensionCommand(cmdInfo.id);
              }
            }
          };
        })
      );

      updateFilteredCommands();
    } catch (error) {
      console.error('[CommandPalette] Failed to load commands:', error);
      // Fallback to built-in commands only
      allCommands = Object.entries(builtinActions).map(([id, action]) => ({
        id,
        label: id,
        action
      }));
    }
  }

  async function executeExtensionCommand(commandId: string, args: any[] = []) {
    try {
      await invoke('extension_execute_command', {
        command: commandId,
        args
      });
    } catch (error) {
      console.error(`[CommandPalette] Failed to execute command ${commandId}:`, error);
    }
  }

  function updateFilteredCommands() {
    if (searchQuery.trim() === '') {
      filteredCommands = allCommands;
    } else {
      const query = searchQuery.toLowerCase();
      filteredCommands = allCommands
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

  // Watch for search query changes
  $: {
    updateFilteredCommands();
  }

  // Listen for command registry updates
  let commandRegisteredUnlisten: (() => void) | null = null;
  let commandUnregisteredUnlisten: (() => void) | null = null;

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
    // Reload commands when palette opens to get latest
    loadCommands();
  }

  onMount(async () => {
    window.addEventListener('keydown', handleKeydown);

    // Initial load of commands
    await loadCommands();

    // Listen for command registry updates
    commandRegisteredUnlisten = await listen('command-registered', () => {
      loadCommands();
    });

    commandUnregisteredUnlisten = await listen('command-unregistered', () => {
      loadCommands();
    });
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);

    // Unlisten from events
    if (commandRegisteredUnlisten) {
      commandRegisteredUnlisten();
    }
    if (commandUnregisteredUnlisten) {
      commandUnregisteredUnlisten();
    }
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
              <div class="command-meta">
                {#if command.keybinding}
                  <span class="command-keybinding">{command.keybinding}</span>
                {/if}
                {#if command.category}
                  <span class="command-category">{command.category}</span>
                {/if}
              </div>
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

  .command-meta {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .command-keybinding {
    color: var(--color-text-secondary);
    font-size: 11px;
    font-family: monospace;
    background-color: var(--color-bg-tertiary);
    padding: 2px 6px;
    border-radius: 3px;
    border: 1px solid var(--color-border);
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
