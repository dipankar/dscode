<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { clearWindowPrompt } from '../stores/windowPrompt';
  import type { WindowPrompt } from '../stores/windowPrompt';

  export let prompt: WindowPrompt;
  let primaryButton: HTMLButtonElement | null = null;

  async function choose(action?: string) {
    try {
      if (prompt.kind === 'local' && prompt.resolve) {
        prompt.resolve(action);
      } else {
        await invoke('window_message_action', {
          requestId: prompt.id,
          action,
        });
      }
    } catch (error) {
      console.error('[WindowPromptModal] Failed to respond to message:', error);
    } finally {
      clearWindowPrompt();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      choose(undefined);
    }
  }

  onMount(async () => {
    await tick();
    primaryButton?.focus();
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="prompt-overlay" role="dialog" aria-modal="true" aria-labelledby="window-prompt-title">
  <div class="prompt-modal">
    <header class={`level-${prompt.level}`}>
      <h2 id="window-prompt-title">
        {prompt.level === 'error'
          ? 'Error'
          : prompt.level === 'warning'
            ? 'Warning'
            : 'Information'}
      </h2>
    </header>
    <main>
      <p>{prompt.message}</p>
    </main>
    <footer>
      {#each prompt.actions as action (action)}
        <button bind:this={primaryButton} class="action" on:click={() => choose(action)}
          >{action}</button
        >
      {/each}
      {#if prompt.showCancelButton !== false}
        <button class="secondary" on:click={() => choose(undefined)} aria-label="Cancel prompt"
          >{prompt.cancelLabel || 'Cancel'}</button
        >
      {/if}
    </footer>
  </div>
</div>

<style>
  .prompt-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }

  .prompt-modal {
    width: min(420px, 90vw);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  header.level-info {
    background: rgba(45, 132, 255, 0.15);
  }

  header.level-warning {
    background: rgba(255, 196, 0, 0.18);
  }

  header.level-error {
    background: rgba(255, 70, 66, 0.18);
  }

  header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
  }

  main {
    padding: 18px 16px;
    color: var(--color-text);
  }

  footer {
    padding: 12px 16px;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    border-top: 1px solid var(--color-border);
  }

  button {
    border: none;
    border-radius: 4px;
    padding: 6px 12px;
    font-size: 13px;
    cursor: pointer;
  }

  button.action {
    background-color: var(--color-accent);
    color: #fff;
  }

  button.secondary {
    background: transparent;
    color: var(--color-text-secondary);
  }

  button.secondary:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  button.action:hover {
    background-color: var(--color-accent-hover, var(--color-accent));
  }
</style>
