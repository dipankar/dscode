<script lang="ts">
  import { resolveLocalInputBox } from '../stores/windowPrompt';
  import type { InputBoxPrompt } from '../stores/windowPrompt';
  import { focusTrap } from '../lib/focus-trap';

  export let request: InputBoxPrompt;

  let value: string = request.value ?? '';
  let inputEl: HTMLInputElement | null = null;

  $: if (request) {
    value = request.value ?? '';
  }

  import { onMount } from 'svelte';

  onMount(() => {
    setTimeout(() => {
      inputEl?.focus();
      inputEl?.select();
    }, 50);
  });

  function handleSubmit() {
    resolveLocalInputBox(value);
  }

  function handleCancel() {
    resolveLocalInputBox(null);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      handleCancel();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="input-overlay" on:click={handleCancel} role="presentation" tabindex="-1" use:focusTrap>
  <div class="input-modal" role="dialog" aria-modal="true" on:click|stopPropagation>
    <div class="input-header">
      <h2>{request.prompt}</h2>
    </div>

    <div class="input-body">
      <input
        bind:this={inputEl}
        type="text"
        bind:value
        placeholder={request.placeholder ?? ''}
        on:keydown={handleKeydown}
      />
    </div>

    <div class="input-footer">
      <button class="btn-secondary" on:click={handleCancel}>Cancel</button>
      <button class="btn-primary" on:click={handleSubmit}>OK</button>
    </div>
  </div>
</div>

<style>
  .input-overlay {
    position: fixed;
    inset: 0;
    background: var(--color-modal-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
    padding: 24px;
  }

  .input-modal {
    width: min(400px, 90vw);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .input-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .input-header h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text);
  }

  .input-body {
    padding: 16px;
  }

  .input-body input {
    width: 100%;
    padding: 8px 12px;
    border-radius: 4px;
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 14px;
    outline: none;
  }

  .input-body input:focus {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 1px var(--color-accent);
  }

  .input-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
  }

  .btn-primary,
  .btn-secondary {
    padding: 6px 14px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-size: 13px;
  }

  .btn-primary {
    background: var(--color-accent);
    color: var(--color-text-on-accent);
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-secondary {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .btn-secondary:hover {
    background: var(--color-surface-hover);
  }
</style>
