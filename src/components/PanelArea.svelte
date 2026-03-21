<script lang="ts">
  import { onMount } from 'svelte';
  import ProblemsPanel from './ProblemsPanel.svelte';
  import OutputPanel from './OutputPanel.svelte';
  import DebugConsole from './DebugConsole.svelte';
  import Terminal from './Terminal.svelte';
import { editorStore } from '../stores/editor';
import { invoke } from '@tauri-apps/api/core';
import { Plus, X } from 'lucide-svelte';
import { outputChannelReveal } from '../stores/outputChannels';

  let activePanel = 'terminal';

  const panels = [
    { id: 'problems', label: 'Problems', icon: '⚠️' },
    { id: 'output', label: 'Output', icon: '📤' },
    { id: 'debug', label: 'Debug Console', icon: '🐛' },
    { id: 'terminal', label: 'Terminal', icon: '💻' },
  ];

  interface TerminalTab {
    id: string;
    name: string;
  }

  let terminals: TerminalTab[] = [];
  let activeTerminalId: string | null = null;

  async function createNewTerminal() {
    try {
      const terminalId = await invoke<string>('create_terminal', {
        name: `Terminal ${terminals.length + 1}`,
        shell: null,
        cwd: null
      });

      terminals = [...terminals, { id: terminalId, name: `Terminal ${terminals.length + 1}` }];
      activeTerminalId = terminalId;
      activePanel = 'terminal';
      console.log(`Terminal created: ${terminalId}, total terminals: ${terminals.length}`);
    } catch (error) {
      console.error('Failed to create terminal:', error);
    }
  }

  function closeTerminal(terminalId: string) {
    terminals = terminals.filter(t => t.id !== terminalId);
    if (activeTerminalId === terminalId) {
      activeTerminalId = terminals.length > 0 ? terminals[0].id : null;
    }
  }

  onMount(() => {
    // Create initial terminal
    createNewTerminal();
  });

  $: if ($outputChannelReveal && activePanel !== 'output') {
    activePanel = 'output';
  }
</script>

<div class="panel-area">
  <div class="panel-tabs">
    {#each panels as panel}
      <button
        class="panel-tab"
        class:active={activePanel === panel.id}
        on:click={() => (activePanel = panel.id)}
      >
        <span class="panel-icon">{panel.icon}</span>
        <span class="panel-label">{panel.label}</span>
      </button>
    {/each}

    <div class="panel-actions">
      <button class="panel-action" title="Maximize Panel">⬆️</button>
      <button class="panel-action" title="Close Panel">×</button>
    </div>
  </div>

  <div class="panel-content">
    {#if activePanel === 'problems'}
      <ProblemsPanel visible={activePanel === 'problems'} editor={$editorStore.monacoInstance} />
    {:else if activePanel === 'output'}
      <OutputPanel visible={activePanel === 'output'} />
    {:else if activePanel === 'debug'}
      <DebugConsole visible={activePanel === 'debug'} />
    {:else if activePanel === 'terminal'}
      <div class="terminal-panel">
        <!-- Terminal tabs -->
        {#if terminals.length > 0}
          <div class="terminal-tabs">
            {#each terminals as terminal (terminal.id)}
              <div
                class="terminal-tab"
                class:active={activeTerminalId === terminal.id}
                on:click={() => activeTerminalId = terminal.id}
              >
                <span class="terminal-tab-name">{terminal.name}</span>
                <button
                  class="terminal-tab-close"
                  on:click|stopPropagation={() => closeTerminal(terminal.id)}
                  title="Close terminal"
                >
                  <X size={14} />
                </button>
              </div>
            {/each}
            <button class="terminal-tab-new" on:click={createNewTerminal} title="New Terminal">
              <Plus size={14} />
            </button>
          </div>
        {/if}

        <!-- Terminal content -->
        <div class="terminal-content">
          {#each terminals as terminal (terminal.id)}
            <div class="terminal-instance" class:hidden={activeTerminalId !== terminal.id}>
              <Terminal
                terminalId={terminal.id}
                visible={activeTerminalId === terminal.id}
                onClose={() => closeTerminal(terminal.id)}
              />
            </div>
          {/each}

          {#if terminals.length === 0}
            <div class="empty-state">
              <p>No terminals open</p>
              <button on:click={createNewTerminal}>Create Terminal</button>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .panel-area {
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border);
    height: 100%;
    min-height: 0;
  }

  .panel-tabs {
    display: flex;
    align-items: center;
    height: 35px;
    border-bottom: 1px solid var(--color-border);
  }

  .panel-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 12px;
    height: 100%;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 13px;
  }

  .panel-tab:hover {
    color: var(--color-text);
  }

  .panel-tab.active {
    color: var(--color-text);
    border-bottom-color: var(--color-accent);
  }

  .panel-icon {
    font-size: 14px;
  }

  .panel-actions {
    margin-left: auto;
    display: flex;
    gap: 4px;
    padding-right: 8px;
  }

  .panel-action {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    padding: 4px 8px;
    font-size: 16px;
  }

  .panel-action:hover {
    color: var(--color-text);
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .terminal-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .terminal-tabs {
    display: flex;
    align-items: center;
    background-color: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
    padding: 0 8px;
    gap: 4px;
  }

  .terminal-tab {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  .terminal-tab:hover {
    background-color: var(--color-bg-secondary);
    color: var(--color-text);
  }

  .terminal-tab.active {
    color: var(--color-text);
    border-bottom-color: var(--color-accent);
    background-color: var(--color-bg-secondary);
  }

  .terminal-tab-name {
    user-select: none;
  }

  .terminal-tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    padding: 2px;
    color: var(--color-text-secondary);
    cursor: pointer;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .terminal-tab-close:hover {
    opacity: 1;
    color: var(--color-text);
  }

  .terminal-tab-new {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .terminal-tab-new:hover {
    background-color: var(--color-bg-secondary);
    color: var(--color-text);
  }

  .terminal-content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .terminal-instance {
    width: 100%;
    height: 100%;
    position: absolute;
    top: 0;
    left: 0;
  }

  .terminal-instance.hidden {
    visibility: hidden;
    pointer-events: none;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-secondary);
    gap: 12px;
  }

  .empty-state button {
    padding: 8px 16px;
    background-color: var(--color-accent);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .empty-state button:hover {
    opacity: 0.9;
  }
</style>
