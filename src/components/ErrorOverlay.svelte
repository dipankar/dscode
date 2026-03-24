<script lang="ts">
  import { clearAppError, type AppErrorState } from '../stores/appError';

  export let error: AppErrorState;

  function reloadApp() {
    window.location.reload();
  }
</script>

<div class="error-overlay" role="alertdialog" aria-modal="true" aria-labelledby="app-error-title">
  <div class="error-panel">
    <h2 id="app-error-title">{error.title}</h2>
    <p>{error.message}</p>
    {#if error.stack}
      <pre>{error.stack}</pre>
    {/if}
    <div class="actions">
      <button on:click={clearAppError}>Dismiss</button>
      <button class="primary" on:click={reloadApp}>Reload</button>
    </div>
  </div>
</div>

<style>
  .error-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.82);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 5000;
  }

  .error-panel {
    width: min(720px, 92vw);
    max-height: 80vh;
    overflow: auto;
    background: var(--color-bg-secondary);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 20px;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.4);
  }

  h2 {
    margin: 0 0 12px;
  }

  p {
    margin: 0 0 12px;
  }

  pre {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 12px;
    overflow: auto;
    white-space: pre-wrap;
    margin: 0 0 16px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  button {
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    border-radius: 4px;
    padding: 8px 12px;
    cursor: pointer;
  }

  button.primary {
    background: var(--color-accent);
    color: #fff;
    border-color: var(--color-accent);
  }
</style>
