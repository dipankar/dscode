<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { AlertCircle, AlertTriangle, Info, X } from 'lucide-svelte';
  import { problemCountsStore } from '../stores/problems';

  export let visible = true;
  export let editor: monaco.editor.IStandaloneCodeEditor | null;

  interface Problem {
    severity: monaco.MarkerSeverity;
    message: string;
    resource: string;
    startLineNumber: number;
    startColumn: number;
    endLineNumber: number;
    endColumn: number;
  }

  let problems: Problem[] = [];
  let errorCount = 0;
  let warningCount = 0;
  let infoCount = 0;
  let markersListener: monaco.IDisposable | null = null;

  function updateProblems() {
    if (!editor) return;

    const allMarkers = monaco.editor.getModelMarkers({});
    const model = editor.getModel();
    const currentUri = model ? model.uri.toString() : null;

    problems = allMarkers.map((marker) => ({
      severity: marker.severity,
      message: marker.message,
      resource: marker.resource.path,
      startLineNumber: marker.startLineNumber,
      startColumn: marker.startColumn,
      endLineNumber: marker.endLineNumber,
      endColumn: marker.endColumn,
    }));

    errorCount = problems.filter((p) => p.severity === monaco.MarkerSeverity.Error).length;
    warningCount = problems.filter((p) => p.severity === monaco.MarkerSeverity.Warning).length;
    infoCount = problems.filter(
      (p) => p.severity === monaco.MarkerSeverity.Info || p.severity === monaco.MarkerSeverity.Hint
    ).length;

    problemCountsStore.set({ errors: errorCount, warnings: warningCount, infos: infoCount });
  }

  function getSeverityIcon(severity: monaco.MarkerSeverity) {
    switch (severity) {
      case monaco.MarkerSeverity.Error:
        return AlertCircle;
      case monaco.MarkerSeverity.Warning:
        return AlertTriangle;
      default:
        return Info;
    }
  }

  function getSeverityClass(severity: monaco.MarkerSeverity): string {
    switch (severity) {
      case monaco.MarkerSeverity.Error:
        return 'error';
      case monaco.MarkerSeverity.Warning:
        return 'warning';
      default:
        return 'info';
    }
  }

  function handleProblemClick(problem: Problem) {
    if (!editor) return;

    // Navigate to the problem location
    editor.setPosition({
      lineNumber: problem.startLineNumber,
      column: problem.startColumn,
    });
    editor.revealLineInCenter(problem.startLineNumber);
    editor.focus();
  }

  onMount(() => {
    updateProblems();
    markersListener = monaco.editor.onDidChangeMarkers(() => {
      updateProblems();
    });
  });

  onDestroy(() => {
    if (markersListener) {
      markersListener.dispose();
    }
  });

  $: if (editor) {
    updateProblems();
  }
</script>

{#if visible}
  <div class="problems-panel">
    <div class="problems-header">
      <div class="problems-title">
        <span>PROBLEMS</span>
        <div class="problem-counts">
          {#if errorCount > 0}
            <span class="count error-count">
              <AlertCircle size={14} />
              {errorCount}
            </span>
          {/if}
          {#if warningCount > 0}
            <span class="count warning-count">
              <AlertTriangle size={14} />
              {warningCount}
            </span>
          {/if}
          {#if infoCount > 0}
            <span class="count info-count">
              <Info size={14} />
              {infoCount}
            </span>
          {/if}
        </div>
      </div>
    </div>

    <div class="problems-list">
      {#if problems.length === 0}
        <div class="no-problems">
          <p>No problems detected</p>
        </div>
      {:else}
        {#each problems as problem}
          <button
            class="problem-item {getSeverityClass(problem.severity)}"
            on:click={() => handleProblemClick(problem)}
          >
            <span class="problem-icon">
              <svelte:component this={getSeverityIcon(problem.severity)} size={16} />
            </span>
            <div class="problem-details">
              <div class="problem-message">{problem.message}</div>
              <div class="problem-location">
                {problem.resource}:{problem.startLineNumber}:{problem.startColumn}
              </div>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>
{/if}

<style>
  .problems-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border);
  }

  .problems-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .problems-title {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.5px;
  }

  .problem-counts {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .count {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 500;
  }

  .error-count {
    color: var(--color-error);
  }

  .warning-count {
    color: var(--color-warning);
  }

  .info-count {
    color: var(--color-info);
  }

  .problems-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .no-problems {
    padding: 20px;
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 13px;
  }

  .problem-item {
    width: 100%;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 6px 12px;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .problem-item:hover {
    background-color: var(--color-bg-tertiary);
  }

  .problem-icon {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .problem-item.error .problem-icon {
    color: var(--color-error);
  }

  .problem-item.warning .problem-icon {
    color: var(--color-warning);
  }

  .problem-item.info .problem-icon {
    color: var(--color-info);
  }

  .problem-details {
    flex: 1;
    min-width: 0;
  }

  .problem-message {
    font-size: 13px;
    line-height: 1.4;
    margin-bottom: 2px;
  }

  .problem-location {
    font-size: 11px;
    color: var(--color-text-secondary);
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  }
</style>
