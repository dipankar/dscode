<script lang="ts">
  import { callStack } from '../lib/stores';
  import type { StackFrame } from '../lib/types';

  let frames: StackFrame[] = [];
  let selectedFrameId: number | null = null;

  callStack.subscribe((stack) => {
    frames = stack;
    if (stack.length > 0 && selectedFrameId === null) {
      selectedFrameId = stack[0].id;
    }
  });

  function selectFrame(frame: StackFrame) {
    selectedFrameId = frame.id;
    // In a real implementation, would emit event to update editor position
    console.log('[CallStack] Selected frame:', frame);
  }

  function getFrameLabel(frame: StackFrame): string {
    if (frame.source?.name) {
      return `${frame.name} (${frame.source.name}:${frame.line})`;
    }
    return frame.name;
  }

  function getFrameLocation(frame: StackFrame): string {
    if (frame.source?.path) {
      return `${frame.source.path}:${frame.line}:${frame.column}`;
    }
    return `Line ${frame.line}, Column ${frame.column}`;
  }
</script>

<div class="call-stack-panel">
  <div class="header">
    <span class="title">Call Stack</span>
  </div>

  <div class="content">
    {#if frames.length === 0}
      <div class="empty-state">
        <span class="empty-icon">📚</span>
        <p>No call stack to display</p>
        <p class="hint">Pause execution to view call stack</p>
      </div>
    {:else}
      <div class="stack-list">
        {#each frames as frame, index}
          <button
            class="stack-frame"
            class:selected={selectedFrameId === frame.id}
            on:click={() => selectFrame(frame)}
          >
            <div class="frame-header">
              <span class="frame-index">{index}</span>
              <span class="frame-name">{frame.name}</span>
            </div>
            <div class="frame-location">
              {#if frame.source?.name}
                <span class="file-name">{frame.source.name}</span>
                <span class="position">:{frame.line}:{frame.column}</span>
              {:else}
                <span class="position">Line {frame.line}</span>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .call-stack-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .header {
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  .content {
    flex: 1;
    overflow-y: auto;
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

  .stack-list {
    padding: 4px 0;
  }

  .stack-frame {
    width: 100%;
    padding: 6px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 4px;
    border-left: 3px solid transparent;
    transition: all 0.1s;
  }

  .stack-frame:hover {
    background: var(--bg-hover);
  }

  .stack-frame.selected {
    background: var(--bg-active);
    border-left-color: var(--accent-color);
  }

  .frame-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .frame-index {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    background: var(--bg-secondary);
    border-radius: 3px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .stack-frame.selected .frame-index {
    background: var(--accent-color);
    color: white;
  }

  .frame-name {
    font-weight: 500;
    font-family: 'Fira Code', 'Consolas', monospace;
  }

  .frame-location {
    font-size: 11px;
    color: var(--text-secondary);
    padding-left: 28px;
    font-family: 'Fira Code', 'Consolas', monospace;
  }

  .file-name {
    color: var(--text-tertiary);
  }

  .position {
    color: var(--text-tertiary);
    opacity: 0.8;
  }
</style>
