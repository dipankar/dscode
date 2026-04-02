<script lang="ts">
  import { toastStore, type Toast } from '../lib/error-handler';
  import { AlertCircle, CheckCircle, Info, AlertTriangle, X } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';

  $: toasts = $toastStore;

  function getIcon(type: Toast['type']) {
    switch (type) {
      case 'error':
        return AlertCircle;
      case 'success':
        return CheckCircle;
      case 'warning':
        return AlertTriangle;
      case 'info':
        return Info;
    }
  }

  function getIconColor(type: Toast['type']) {
    const style = getComputedStyle(document.documentElement);
    switch (type) {
      case 'error':
        return style.getPropertyValue('--color-toast-error').trim() || '#f44336';
      case 'success':
        return style.getPropertyValue('--color-toast-success').trim() || '#4caf50';
      case 'warning':
        return style.getPropertyValue('--color-toast-warning').trim() || '#ff9800';
      case 'info':
        return style.getPropertyValue('--color-toast-info').trim() || '#2196f3';
    }
  }
</script>

<div class="toast-container" role="status" aria-live="polite" aria-atomic="true">
  {#each toasts as toast (toast.id)}
    <div class="toast toast-{toast.type}" transition:fly={{ y: 20, duration: 300 }}>
      <div class="toast-icon" style="color: {getIconColor(toast.type)}">
        <svelte:component this={getIcon(toast.type)} size={20} />
      </div>
      <div class="toast-content">
        <div class="toast-message">{toast.message}</div>
        {#if toast.details}
          <details class="toast-details">
            <summary>Show details</summary>
            <pre>{toast.details}</pre>
          </details>
        {/if}
      </div>
      <button class="toast-close" on:click={() => toastStore.dismiss(toast.id)} title="Dismiss">
        <X size={16} />
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 400px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 16px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: var(--shadow-md);
    pointer-events: auto;
    min-width: 300px;
  }

  .toast-error {
    border-left: 3px solid var(--color-toast-error);
  }

  .toast-success {
    border-left: 3px solid var(--color-toast-success);
  }

  .toast-warning {
    border-left: 3px solid var(--color-toast-warning);
  }

  .toast-info {
    border-left: 3px solid var(--color-toast-info);
  }

  .toast-icon {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .toast-content {
    flex: 1;
    min-width: 0;
  }

  .toast-message {
    font-size: 14px;
    color: var(--color-text);
    word-wrap: break-word;
  }

  .toast-details {
    margin-top: 8px;
    font-size: 12px;
  }

  .toast-details summary {
    cursor: pointer;
    color: var(--color-text-secondary);
    user-select: none;
  }

  .toast-details summary:hover {
    color: var(--color-accent);
  }

  .toast-details pre {
    margin-top: 8px;
    padding: 8px;
    background: var(--color-bg-tertiary);
    border-radius: 4px;
    font-size: 11px;
    color: var(--color-text-secondary);
    overflow-x: auto;
    max-height: 200px;
  }

  .toast-close {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .toast-close:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text);
  }
</style>
