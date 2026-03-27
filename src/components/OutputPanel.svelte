<script lang="ts">
  import { tick } from 'svelte';
  import {
    outputChannelList,
    activeOutputChannel,
    setActiveOutputChannel,
    clearOutputChannel,
  } from '../stores/outputChannels';

  export let visible = false;

  let outputContainer: HTMLDivElement | null = null;

  $: channels = $outputChannelList;
  $: activeChannelId = $activeOutputChannel ?? channels[0]?.id ?? null;
  $: currentChannel = channels.find((channel) => channel.id === activeChannelId) ?? null;

  $: if (
    channels.length > 0 &&
    (!activeChannelId || !channels.some((channel) => channel.id === activeChannelId))
  ) {
    setActiveOutputChannel(channels[0].id);
  }

  $: if (outputContainer && currentChannel) {
    tick().then(() => {
      if (outputContainer) {
        outputContainer.scrollTop = outputContainer.scrollHeight;
      }
    });
  }

  function handleChannelChange(event: Event) {
    const value = (event.target as HTMLSelectElement).value;
    if (value && value !== activeChannelId) {
      setActiveOutputChannel(value);
    }
  }

  function handleClear() {
    if (currentChannel) {
      clearOutputChannel(currentChannel.id);
    }
  }
</script>

<div class="output-panel" class:visible>
  <div class="output-header">
    <select class="channel-select" value={activeChannelId ?? ''} on:change={handleChannelChange}>
      {#each channels as channel}
        <option value={channel.id}>{channel.id}</option>
      {/each}
    </select>

    <div class="output-actions">
      <button
        class="output-action"
        on:click={handleClear}
        title="Clear Output"
        disabled={!currentChannel}
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8 2.146 2.854Z"
          />
        </svg>
      </button>
    </div>
  </div>

  <div class="output-content" bind:this={outputContainer}>
    {#if !currentChannel}
      <div class="empty-state">
        <p>No output channels</p>
      </div>
    {:else if currentChannel.lines.length === 0}
      <div class="empty-state">
        <p>No output yet</p>
      </div>
    {:else}
      {#each currentChannel.lines as line, index}
        <div class="output-line">
          <span class="output-message">{line}</span>
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
    padding: 2px 0;
    color: var(--text-primary);
  }

  .output-message {
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
