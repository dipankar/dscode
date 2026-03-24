<script lang="ts">
  import { onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { debugSession, debugState } from '../lib/stores';
  import type { DebugSession } from '../lib/types';

  let sessionId: string | null = null;
  let currentState: 'Stopped' | 'Running' | 'Paused' | 'Terminated' = 'Stopped';

  const unsubscribeDebugSession = debugSession.subscribe((session: DebugSession | null) => {
    sessionId = session?.id || null;
    currentState = session?.state || 'Stopped';
  });

  onDestroy(() => {
    unsubscribeDebugSession();
  });

  async function startDebug() {
    if (!sessionId) {
      // Create new session
      try {
        const id = await invoke<string>('create_debug_session', {
          name: 'Debug Session',
          adapterType: 'node',
        });
        sessionId = id;

        await invoke('start_debugging', {
          sessionId: id,
          config: {
            type: 'node',
            request: 'launch',
            name: 'Debug',
            program: '${workspaceFolder}/index.js',
          },
        });

        const session = await invoke<DebugSession>('get_debug_session', { sessionId: id });
        debugSession.set(session);
        console.log('[Debug] Started session:', id);
      } catch (error) {
        console.error('[Debug] Failed to start:', error);
      }
    } else {
      // Continue existing session
      try {
        await invoke('continue_debugging', { sessionId });
        const session = await invoke<DebugSession>('get_debug_session', { sessionId });
        debugSession.set(session);
        console.log('[Debug] Continued session');
      } catch (error) {
        console.error('[Debug] Failed to continue:', error);
      }
    }
  }

  async function pauseDebug() {
    if (!sessionId) return;
    try {
      await invoke('pause_debugging', { sessionId });
      const session = await invoke<DebugSession>('get_debug_session', { sessionId });
      debugSession.set(session);
      console.log('[Debug] Paused');
    } catch (error) {
      console.error('[Debug] Failed to pause:', error);
    }
  }

  async function stopDebug() {
    if (!sessionId) return;
    try {
      await invoke('stop_debugging', { sessionId });
      debugSession.set(null);
      sessionId = null;
      console.log('[Debug] Stopped');
    } catch (error) {
      console.error('[Debug] Failed to stop:', error);
    }
  }

  async function stepOver() {
    if (!sessionId) return;
    try {
      await invoke('step_over', { sessionId });
      console.log('[Debug] Step over');
    } catch (error) {
      console.error('[Debug] Failed to step over:', error);
    }
  }

  async function stepInto() {
    if (!sessionId) return;
    try {
      await invoke('step_into', { sessionId });
      console.log('[Debug] Step into');
    } catch (error) {
      console.error('[Debug] Failed to step into:', error);
    }
  }

  async function stepOut() {
    if (!sessionId) return;
    try {
      await invoke('step_out', { sessionId });
      console.log('[Debug] Step out');
    } catch (error) {
      console.error('[Debug] Failed to step out:', error);
    }
  }

  $: canStart = currentState === 'Stopped' || currentState === 'Terminated';
  $: canPause = currentState === 'Running';
  $: canContinue = currentState === 'Paused';
  $: canStop = sessionId !== null && currentState !== 'Stopped' && currentState !== 'Terminated';
  $: canStep = currentState === 'Paused';
</script>

<div class="debug-toolbar">
  <div class="controls">
    <button
      class="btn"
      class:active={canContinue}
      disabled={!canStart && !canContinue}
      on:click={startDebug}
      title={canContinue ? 'Continue (F5)' : 'Start Debugging (F5)'}
    >
      {#if canContinue}
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M3 3v10l9-5z" />
        </svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M3 3v10l9-5z" />
        </svg>
      {/if}
    </button>

    <button class="btn" disabled={!canPause} on:click={pauseDebug} title="Pause (F6)">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M4 3h3v10H4zM9 3h3v10H9z" />
      </svg>
    </button>

    <button class="btn stop" disabled={!canStop} on:click={stopDebug} title="Stop (Shift+F5)">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <rect x="4" y="4" width="8" height="8" />
      </svg>
    </button>

    <div class="separator"></div>

    <button class="btn" disabled={!canStep} on:click={stepOver} title="Step Over (F10)">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M14 5.5l-4-3v2H7a3 3 0 00-3 3v1h1v-1a2 2 0 012-2h3v2l4-2.5z" />
        <path d="M4 9h1v4h6v-1h1v1a1 1 0 01-1 1H5a1 1 0 01-1-1V9z" />
      </svg>
    </button>

    <button class="btn" disabled={!canStep} on:click={stepInto} title="Step Into (F11)">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 10l-4-4h2.5V2h3v4H12l-4 4z" />
        <path d="M4 12h8v2H4z" />
      </svg>
    </button>

    <button class="btn" disabled={!canStep} on:click={stepOut} title="Step Out (Shift+F11)">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 6l-4 4h2.5v4h3v-4H12l-4-4z" />
        <path d="M4 2h8v2H4z" />
      </svg>
    </button>
  </div>

  {#if sessionId}
    <div class="status">
      <span
        class="status-dot"
        class:running={currentState === 'Running'}
        class:paused={currentState === 'Paused'}
      ></span>
      <span class="status-text">{currentState}</span>
    </div>
  {/if}
</div>

<style>
  .debug-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    height: 35px;
  }

  .controls {
    display: flex;
    gap: 2px;
    align-items: center;
  }

  .btn {
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.1s;
  }

  .btn:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn:active:not(:disabled) {
    background: var(--bg-active);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn.active {
    color: var(--accent-color);
  }

  .btn.stop:not(:disabled) {
    color: #f48771;
  }

  .separator {
    width: 1px;
    height: 16px;
    background: var(--border-color);
    margin: 0 4px;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-tertiary);
  }

  .status-dot.running {
    background: #89d185;
    animation: pulse 1.5s ease-in-out infinite;
  }

  .status-dot.paused {
    background: #f9c74f;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
