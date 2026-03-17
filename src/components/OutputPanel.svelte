<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  export let visible = false;

  interface OutputMessage {
    timestamp: string;
    level: 'info' | 'warn' | 'error';
    message: string;
  }

  let messages: OutputMessage[] = [];
  let outputContainer: HTMLDivElement;
  let unlisten: (() => void) | null = null;
  let selectedChannel = 'Tasks';

  const channels = ['Tasks', 'Build', 'Extension Host', 'Git', 'Debug'];

  function addMessage(level: 'info' | 'warn' | 'error', message: string) {
    const timestamp = new Date().toLocaleTimeString();
    messages = [...messages, { timestamp, level, message }];

    // Auto-scroll to bottom
    setTimeout(() => {
      if (outputContainer) {
        outputContainer.scrollTop = outputContainer.scrollHeight;
      }
    }, 0);
  }

  function clearOutput() {
    messages = [];
  }

  onMount(async () => {
    // Listen for extension host messages
    unlisten = await listen('extension-output', (event: any) => {
      addMessage('info', event.payload.message);
    });

    // Add some initial messages
    addMessage('info', 'Output panel ready');
  });

  onDestroy(() => {
    if (unlisten) {
      unlisten();
    }
  });
</script>

<div class="output-panel" class:visible>
  <div class="output-header">
    <select bind:value={selectedChannel} class="channel-select">
      {#each channels as channel}
        <option value={channel}>{channel}</option>
      {/each}
    </select>

    <div class="output-actions">
      <button class="output-action" on:click={clearOutput} title="Clear Output">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8 2.146 2.854Z"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="output-content" bind:this={outputContainer}>
    {#if messages.length === 0}
      <div class="empty-state">
        <p>No output yet</p>
      </div>
    {:else}
      {#each messages as msg}
        <div class="output-line" class:error={msg.level === 'error'} class:warn={msg.level === 'warn'}>
          <span class="output-timestamp">[{msg.timestamp}]</span>
          <span class="output-message">{msg.message}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .output-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    background-color: var(--bg-primary);
  }

  .output-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border-color);
    background-color: var(--bg-secondary);
  }

  .channel-select {
    background-color: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    padding: 4px 8px;
    font-size: 12px;
    cursor: pointer;
  }

  .channel-select:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .output-actions {
    display: flex;
    gap: 4px;
  }

  .output-action {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    border-radius: 3px;
  }

  .output-action:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }

  .output-content {
    flex: 1;
    overflow-y: auto;
    font-family: 'Fira Code', 'Consolas', monospace;
    font-size: 13px;
    padding: 8px;
    line-height: 1.5;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
  }

  .output-line {
    display: flex;
    gap: 8px;
    padding: 2px 0;
    color: var(--text-primary);
  }

  .output-line.error {
    color: #f48771;
  }

  .output-line.warn {
    color: #cca700;
  }

  .output-timestamp {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .output-message {
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
