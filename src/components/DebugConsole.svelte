<script lang="ts">
  import { debugConsole } from '../lib/stores';
  import { onMount, onDestroy } from 'svelte';

  export let visible = false;

  let messages: string[] = [];
  let inputValue = '';
  let consoleContainer: HTMLDivElement;
  let autoScroll = true;
  let unsubscribe: (() => void) | null = null;

  onMount(() => {
    unsubscribe = debugConsole.subscribe((msgs) => {
      messages = msgs;
      if (autoScroll) {
        scrollToBottom();
      }
    });
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });

  function scrollToBottom() {
    setTimeout(() => {
      if (consoleContainer) {
        consoleContainer.scrollTop = consoleContainer.scrollHeight;
      }
    }, 0);
  }

  function handleSubmit(event: Event) {
    event.preventDefault();
    if (inputValue.trim()) {
      // Add user input to console
      debugConsole.update((msgs) => [...msgs, `> ${inputValue}`]);

      // In a real implementation, would evaluate expression and show result
      console.log('[DebugConsole] Evaluating:', inputValue);

      // Mock response
      setTimeout(() => {
        debugConsole.update((msgs) => [...msgs, `← undefined`]);
      }, 100);

      inputValue = '';
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      handleSubmit(event);
    }
  }

  function clearConsole() {
    debugConsole.set([]);
  }

  function getMessageType(msg: string): 'input' | 'output' | 'error' | 'info' {
    if (msg.startsWith('>')) return 'input';
    if (msg.startsWith('←')) return 'output';
    if (msg.includes('Error') || msg.includes('error')) return 'error';
    return 'info';
  }
</script>

<div class="debug-console">
  <div class="header">
    <span class="title">Debug Console</span>
    <button class="clear-btn" on:click={clearConsole} title="Clear Console">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 1a7 7 0 110 14A7 7 0 018 1zM4 7h8v2H4z" />
      </svg>
    </button>
  </div>

  <div class="console-content" bind:this={consoleContainer}>
    {#if messages.length === 0}
      <div class="empty-state">
        <span class="empty-icon">💬</span>
        <p>Debug console ready</p>
        <p class="hint">Type expressions to evaluate during debugging</p>
      </div>
    {:else}
      <div class="messages">
        {#each messages as message}
          <div
            class="message"
            class:input={getMessageType(message) === 'input'}
            class:output={getMessageType(message) === 'output'}
            class:error={getMessageType(message) === 'error'}
            class:info={getMessageType(message) === 'info'}
            aria-hidden={!visible}
          >
            <pre>{message}</pre>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <form class="console-input" on:submit={handleSubmit}>
    <div class="input-wrapper">
      <span class="prompt">›</span>
      <input
        type="text"
        bind:value={inputValue}
        on:keydown={handleKeyDown}
        placeholder="Evaluate expression..."
        class="input-field"
      />
    </div>
  </form>
</div>

<style>
  .debug-console {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  .clear-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    transition: all 0.1s;
  }

  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .console-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    font-family: 'Fira Code', 'Consolas', monospace;
    font-size: 12px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    padding: 20px;
    text-align: center;
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.5;
  }

  .empty-state p {
    margin: 4px 0;
    font-size: 13px;
  }

  .hint {
    font-size: 11px;
    opacity: 0.7;
  }

  .messages {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .message {
    padding: 2px 0;
  }

  .message pre {
    margin: 0;
    font-family: inherit;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .message.input {
    color: var(--text-secondary);
  }

  .message.output {
    color: var(--color-debug-link);
  }

  .message.error {
    color: var(--color-error);
  }

  .message.info {
    color: var(--text-primary);
  }

  .console-input {
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .input-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
  }

  .prompt {
    color: var(--accent-color);
    font-size: 14px;
    font-weight: 600;
    font-family: 'Fira Code', 'Consolas', monospace;
  }

  .input-field {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-family: 'Fira Code', 'Consolas', monospace;
    font-size: 12px;
    outline: none;
  }

  .input-field::placeholder {
    color: var(--text-tertiary);
    opacity: 0.6;
  }
</style>
