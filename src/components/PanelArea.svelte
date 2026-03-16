<script lang="ts">
  import ProblemsPanel from './ProblemsPanel.svelte';
  import { editorStore } from '../stores/editor';

  let activePanel = 'terminal';

  const panels = [
    { id: 'problems', label: 'Problems', icon: '⚠️' },
    { id: 'output', label: 'Output', icon: '📤' },
    { id: 'debug', label: 'Debug Console', icon: '🐛' },
    { id: 'terminal', label: 'Terminal', icon: '💻' },
  ];
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
    {#if activePanel === 'terminal'}
      <div class="terminal">
        <div class="terminal-line">$ npm run tauri:dev</div>
        <div class="terminal-line terminal-success">✓ DSCode is running...</div>
        <div class="terminal-cursor">_</div>
      </div>
    {:else if activePanel === 'problems'}
      <ProblemsPanel visible={activePanel === 'problems'} editor={$editorStore.monacoInstance} />
    {/if}
  </div>
</div>

<style>
  .panel-area {
    height: 200px;
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border);
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
    overflow: auto;
    padding: 8px;
  }

  .terminal {
    font-family: 'Consolas', 'Courier New', monospace;
    font-size: 14px;
  }

  .terminal-line {
    padding: 2px 0;
    color: var(--color-text);
  }

  .terminal-success {
    color: #4ec9b0;
  }

  .terminal-cursor {
    display: inline-block;
    animation: blink 1s infinite;
  }

  @keyframes blink {
    0%,
    50% {
      opacity: 1;
    }
    51%,
    100% {
      opacity: 0;
    }
  }

  .empty-state {
    color: var(--color-text-secondary);
    text-align: center;
    padding: 20px;
  }
</style>
