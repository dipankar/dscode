<script lang="ts">
  import DebugToolbar from './DebugToolbar.svelte';
  import VariablesPanel from './VariablesPanel.svelte';
  import CallStackPanel from './CallStackPanel.svelte';
  import DebugConsole from './DebugConsole.svelte';

  let selectedPanel: 'variables' | 'callstack' | 'console' = 'variables';
</script>

<div class="debug-view" role="region" aria-label="Debug">
  <div class="debug-header">
    <h3>RUN AND DEBUG</h3>
  </div>

  <DebugToolbar />

  <div class="debug-content">
    <div class="panel-tabs">
      <button
        class="tab"
        class:active={selectedPanel === 'variables'}
        on:click={() => selectedPanel = 'variables'}
      >
        Variables
      </button>
      <button
        class="tab"
        class:active={selectedPanel === 'callstack'}
        on:click={() => selectedPanel = 'callstack'}
      >
        Call Stack
      </button>
      <button
        class="tab"
        class:active={selectedPanel === 'console'}
        on:click={() => selectedPanel = 'console'}
      >
        Debug Console
      </button>
    </div>

    <div class="panel-content">
      {#if selectedPanel === 'variables'}
        <VariablesPanel />
      {:else if selectedPanel === 'callstack'}
        <CallStackPanel />
      {:else if selectedPanel === 'console'}
        <DebugConsole />
      {/if}
    </div>
  </div>
</div>

<style>
  .debug-view {
    width: 100%;
    height: 100%;
    background-color: var(--color-bg-secondary);
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--color-border);
  }

  .debug-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .debug-header h3 {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.5px;
  }

  .debug-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-tabs {
    display: flex;
    gap: 0;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .tab {
    padding: 8px 16px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    border-bottom: 2px solid transparent;
    transition: all 0.1s;
  }

  .tab:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tab.active {
    color: var(--text-primary);
    border-bottom-color: var(--accent-color);
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
  }
</style>
