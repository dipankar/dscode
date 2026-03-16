<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { editorStore } from '../stores/editor';

  let branch = 'main';
  let line = 42;
  let column = 16;
  let encoding = 'UTF-8';
  let language = 'TypeScript';
  let errorCount = 0;
  let warningCount = 0;
  let updateInterval: number;

  function updateProblemCounts() {
    const editor = $editorStore.monacoInstance;
    if (!editor) {
      errorCount = 0;
      warningCount = 0;
      return;
    }

    const model = editor.getModel();
    if (!model) {
      errorCount = 0;
      warningCount = 0;
      return;
    }

    const markers = monaco.editor.getModelMarkers({ resource: model.uri });
    errorCount = markers.filter(m => m.severity === monaco.MarkerSeverity.Error).length;
    warningCount = markers.filter(m => m.severity === monaco.MarkerSeverity.Warning).length;
  }

  function togglePanel() {
    window.dispatchEvent(new Event('togglePanel'));
  }

  onMount(() => {
    updateProblemCounts();
    updateInterval = window.setInterval(updateProblemCounts, 1000);
  });

  onDestroy(() => {
    if (updateInterval) {
      clearInterval(updateInterval);
    }
  });

  $: if ($editorStore.monacoInstance) {
    updateProblemCounts();
  }
</script>

<div class="status-bar">
  <div class="status-left">
    <button class="status-item" title="Source Control">
      🔀 {branch}
    </button>

    <button class="status-item" title="Toggle Problems Panel" on:click={togglePanel}>
      ⚠️ {warningCount}  ❌ {errorCount}
    </button>
  </div>

  <div class="status-right">
    <button class="status-item" title="Go to Line/Column">
      Ln {line}, Col {column}
    </button>

    <button class="status-item" title="Select Encoding">
      {encoding}
    </button>

    <button class="status-item" title="Select Language Mode">
      {language}
    </button>

    <button class="status-item" title="Notifications">
      🔔
    </button>
  </div>
</div>

<style>
  .status-bar {
    height: 22px;
    background-color: var(--color-accent);
    color: white;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px;
    font-size: 12px;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-item {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    padding: 0 6px;
    height: 100%;
    display: flex;
    align-items: center;
    white-space: nowrap;
  }

  .status-item:hover {
    background-color: var(--color-accent-hover);
  }
</style>
