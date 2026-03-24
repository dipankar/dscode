<script lang="ts">
  import { onMount, onDestroy, beforeUpdate } from 'svelte';
  import * as monaco from 'monaco-editor';

  export let originalContent: string = '';
  export let modifiedContent: string = '';
  export let filePath: string = '';
  export let language: string = 'plaintext';

  let container: HTMLElement;
  let diffEditor: monaco.editor.IStandaloneDiffEditor | null = null;
  let currentOriginalModel: monaco.editor.ITextModel | null = null;
  let currentModifiedModel: monaco.editor.ITextModel | null = null;

  onMount(() => {
    if (!container) return;

    diffEditor = monaco.editor.createDiffEditor(container, {
      enableSplitViewResizing: true,
      renderSideBySide: true,
      readOnly: true,
      automaticLayout: true,
      theme: 'vs-dark',
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
    });

    updateDiff();
  });

  onDestroy(() => {
    // Dispose models first to prevent memory leaks
    if (currentOriginalModel) {
      currentOriginalModel.dispose();
      currentOriginalModel = null;
    }
    if (currentModifiedModel) {
      currentModifiedModel.dispose();
      currentModifiedModel = null;
    }
    if (diffEditor) {
      diffEditor.dispose();
      diffEditor = null;
    }
  });

  function updateDiff() {
    if (!diffEditor) return;

    // Dispose previous models to prevent memory leaks
    if (currentOriginalModel) {
      currentOriginalModel.dispose();
    }
    if (currentModifiedModel) {
      currentModifiedModel.dispose();
    }

    // Create new models and track references
    currentOriginalModel = monaco.editor.createModel(originalContent, language);
    currentModifiedModel = monaco.editor.createModel(modifiedContent, language);

    diffEditor.setModel({
      original: currentOriginalModel,
      modified: currentModifiedModel,
    });
  }

  // Update when props change
  $: if (diffEditor && (originalContent || modifiedContent)) {
    updateDiff();
  }
</script>

<div class="diff-viewer">
  <div class="diff-header">
    <div class="file-path">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path
          d="M1.5 1.75V13.5h13.75a.75.75 0 0 1 0 1.5H.75a.75.75 0 0 1-.75-.75V1.75a.75.75 0 0 1 1.5 0zm14.28 2.53-5.25 5.25a.75.75 0 0 1-1.06 0L7 7.06 4.28 9.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.25-3.25a.75.75 0 0 1 1.06 0L10 7.94l4.72-4.72a.751.751 0 0 1 1.042.018.751.751 0 0 1 .018 1.042z"
        />
      </svg>
      <span>{filePath}</span>
    </div>
  </div>
  <div class="diff-container" bind:this={container}></div>
</div>

<style>
  .diff-viewer {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
  }

  .diff-header {
    padding: 8px 16px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .file-path {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    font-family: 'Fira Code', 'Consolas', monospace;
  }

  .diff-container {
    flex: 1;
    overflow: hidden;
  }
</style>
