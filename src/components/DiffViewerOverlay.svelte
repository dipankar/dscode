<script lang="ts">
  import { diffStore, closeDiff } from '../stores/diffViewer';
  import DiffViewer from './DiffViewer.svelte';

  $: diffState = $diffStore;
</script>

{#if diffState}
  <div
    class="diff-overlay"
    on:click={(e) => {
      if (e.target === e.currentTarget) closeDiff();
    }}
    role="presentation"
    tabindex="-1"
  >
    <div class="diff-container" role="dialog" aria-modal="true" aria-label="File diff viewer">
      <div class="diff-toolbar">
        <span class="diff-title">{diffState.filePath}</span>
        <button class="diff-close" on:click={closeDiff} title="Close">&times;</button>
      </div>
      <div class="diff-content">
        <DiffViewer
          originalContent={diffState.originalContent}
          modifiedContent={diffState.modifiedContent}
          filePath={diffState.filePath}
          language={diffState.language}
        />
      </div>
    </div>
  </div>
{/if}

<style>
  .diff-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--modal-overlay-bg);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .diff-container {
    display: flex;
    flex-direction: column;
    width: 95vw;
    max-width: 1400px;
    height: 90vh;
    max-height: 900px;
    background: var(--modal-bg);
    color: var(--color-text);
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .diff-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-bottom: 1px solid var(--color-border);
    background-color: var(--color-bg-secondary);
  }

  .diff-title {
    font-size: 13px;
    font-weight: 500;
    font-family: 'Fira Code', 'Consolas', monospace;
    color: var(--color-text);
  }

  .diff-close {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 20px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 3px;
  }

  .diff-close:hover {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text);
  }

  .diff-content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
</style>
