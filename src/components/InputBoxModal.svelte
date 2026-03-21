<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { clearInputBox, type InputBoxState } from '../stores/inputBox';

  export let request: InputBoxState;

  let value = '';
  let submitting = false;
  let inputEl: HTMLInputElement | null = null;
  let lastRequestId = '';

$: if (request.id !== lastRequestId) {
    lastRequestId = request.id;
    value = request.value ?? '';
    submitting = false;
    queueMicrotask(() => {
      if (!inputEl) {
        return;
      }
      inputEl.focus();
      if (request.password) {
        inputEl.setSelectionRange(value.length, value.length);
        return;
      }
      if (request.valueSelection && Array.isArray(request.valueSelection)) {
        const [start, end] = request.valueSelection;
        inputEl.setSelectionRange(start ?? 0, end ?? value.length);
      } else {
        inputEl.select();
      }
    });
  }

  async function submit() {
    if (submitting) {
      return;
    }
    submitting = true;
    try {
      await invoke('window_input_box_submit', {
        requestId: request.id,
        value,
      });
    } catch (error) {
      console.error('[InputBoxModal] Failed to submit value:', error);
    } finally {
      clearInputBox();
    }
  }

  async function cancel() {
    if (submitting) {
      return;
    }
    submitting = true;
    try {
      await invoke('window_input_box_submit', {
        requestId: request.id,
        value: null,
      });
    } catch (error) {
      console.error('[InputBoxModal] Failed to cancel input box:', error);
    } finally {
      clearInputBox();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      void submit();
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      void cancel();
    }
  }
</script>

<div class="inputbox-overlay" role="dialog" aria-modal="true">
  <div class="inputbox-modal">
    {#if request.prompt}
      <header>
        <h2>{request.prompt}</h2>
      </header>
    {/if}

    <main>
      <input
        bind:this={inputEl}
        type={request.password ? 'password' : 'text'}
        placeholder={request.placeHolder ?? ''}
        bind:value
        on:keydown={handleKeydown}
      />
    </main>

    <footer>
      <button on:click={() => void submit()} disabled={submitting}>OK</button>
      <button class="secondary" on:click={() => void cancel()} disabled={submitting}>Cancel</button>
    </footer>
  </div>
</div>

<style>
  .inputbox-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
    padding: 24px;
  }

  .inputbox-modal {
    width: min(380px, 90vw);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  header {
    padding: 16px 18px 0;
  }

  header h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text);
  }

  main {
    padding: 18px;
  }

  main input {
    width: 100%;
    padding: 8px 12px;
    border-radius: 4px;
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 14px;
    outline: none;
  }

  main input:focus {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 1px var(--color-accent);
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
  }

  button {
    padding: 6px 14px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-size: 13px;
  }

  button[disabled] {
    cursor: not-allowed;
    opacity: 0.6;
  }

  button:not(.secondary) {
    background: var(--color-accent);
    color: #fff;
  }

  button.secondary {
    background: transparent;
    color: var(--color-text-secondary);
  }

  button.secondary:hover:not([disabled]) {
    background: rgba(255, 255, 255, 0.06);
  }
</style>
