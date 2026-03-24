<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCommandContext, type CommandContext } from '../lib/command-context';
  import { keybindingManager } from '../lib/keybinding-manager';
  import { executeCommand as dispatchCommand } from '../lib/command-dispatcher';
  import { registryCommands } from '../lib/contracts/commands';
  import { evaluateWhenClause } from '../lib/when-clause';

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
  let commandContext: CommandContext = getCommandContext();
  let wasVisible = false;

  // Load commands from the command registry
  async function loadCommands(context: CommandContext = commandContext) {
    try {
      const commandInfos = await registryCommands.getAllCommands<CommandInfo>();
      const availableCommands = commandInfos.filter((cmdInfo) =>
        evaluateWhenClause(cmdInfo.when, context)
      );

      // Map commands and fetch keybindings
      allCommands = await Promise.all(
        availableCommands.map(async (cmdInfo) => {
          const keybinding = await keybindingManager.getKeybindingForCommand(cmdInfo.id);

          return {
            id: cmdInfo.id,
            label: cmdInfo.label,
            category: cmdInfo.category,
            keybinding: keybinding || undefined,
            action: async () => {
              await dispatchCommand(cmdInfo.id);
            },
          };
        })
      );

      updateFilteredCommands();
    } catch (error) {
      console.error('[CommandPalette] Failed to load commands:', error);
      allCommands = [];
    }
  }

  function updateFilteredCommands() {
    if (searchQuery.trim() === '') {
      filteredCommands = allCommands;
    } else {
      const query = searchQuery.toLowerCase();
      filteredCommands = allCommands
        .filter(
          (cmd) =>
            cmd.label.toLowerCase().includes(query) || cmd.category?.toLowerCase().includes(query)
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

  $: if (visible && !wasVisible) {
    commandContext = getCommandContext();
    wasVisible = true;
  }

  $: if (!visible && wasVisible) {
    wasVisible = false;
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

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
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
    loadCommands(commandContext);
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
  <div
    class="command-palette-overlay"
    on:click={handleOverlayClick}
    on:keydown={handleKeydown}
    role="presentation"
    tabindex="-1"
  >
    <div class="command-palette" role="dialog" aria-modal="true" aria-label="Command palette">
      <div class="search-container">
        <input
          bind:this={inputElement}
          bind:value={searchQuery}
          type="text"
          placeholder="Type a command or search..."
          class="search-input"
          aria-label="Search commands"
          aria-controls="command-palette-list"
        />
      </div>

      <div
        class="commands-list"
        id="command-palette-list"
        role="listbox"
        aria-label="Available commands"
      >
        {#if filteredCommands.length === 0}
          <div class="no-results">No commands found</div>
        {:else}
          {#each filteredCommands as command, index}
            <button
              class="command-item"
              class:selected={index === selectedIndex}
              on:click={() => executeCommand(command)}
              on:mouseenter={() => (selectedIndex = index)}
              role="option"
              aria-selected={index === selectedIndex}
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
