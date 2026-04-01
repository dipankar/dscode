<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { clearWindowPrompt } from '../stores/windowPrompt';
  import type { WindowPrompt } from '../stores/windowPrompt';
  import { focusTrap } from '../lib/focus-trap';

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

  onMount(async () => {
    await tick();
    primaryButton?.focus();
  });
</script>

<div class="prompt-overlay" role="dialog" aria-modal="true" aria-labelledby="window-prompt-title" use:focusTrap={{ onEscape: () => choose(undefined) }}>
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
        <button
          class="secondary"
          on:click={() => choose(prompt.cancelLabel)}
          aria-label="Cancel prompt">{prompt.cancelLabel || 'Cancel'}</button
        >
      {/if}
      {#if prompt.secondaryLabel}
        <button class="tertiary" on:click={() => choose(undefined)} aria-label="Cancel"
          >{prompt.secondaryLabel}</button
        >
      {/if}
    </footer>
  </div>
</div>

<style>
  .prompt-overlay {
    position: fixed;
    inset: 0;
    background: var(--color-modal-overlay);
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
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  header.level-info {
    background: var(--color-btn-primary-bg);
  }

  header.level-warning {
    background: var(--color-btn-secondary-bg);
  }

  header.level-error {
    background: var(--color-btn-danger-bg);
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
    color: var(--color-text-on-accent);
  }

  button.secondary {
    background: transparent;
    color: var(--color-text-secondary);
  }

  button.secondary:hover {
    background: var(--color-surface-hover);
  }

  button.tertiary {
    background: transparent;
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border);
  }

  button.tertiary:hover {
    background: var(--color-surface-hover);
  }

  button.action:hover {
    background-color: var(--color-accent-hover, var(--color-accent));
  }
</style>
