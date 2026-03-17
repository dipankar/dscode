<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { editorStore } from '../stores/editor';
  import { workspaceStore } from '../stores/workspace';

  let branch = 'main';
  let line = 1;
  let column = 1;
  let encoding = 'UTF-8';
  let language = 'Plain Text';
  let errorCount = 0;
  let warningCount = 0;
  let updateInterval: number;
  let cursorUpdateUnsubscribe: { dispose: () => void } | null = null;

  async function updateProblemCounts() {
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

    try {
      // Lazy load monaco only when needed
      const monaco = await import('monaco-editor');
      const markers = monaco.editor.getModelMarkers({ resource: model.uri });
      errorCount = markers.filter(m => m.severity === monaco.MarkerSeverity.Error).length;
      warningCount = markers.filter(m => m.severity === monaco.MarkerSeverity.Warning).length;
    } catch (error) {
      console.error('Failed to load monaco for status bar:', error);
    }
  }

  function togglePanel() {
    window.dispatchEvent(new Event('togglePanel'));
  }

  async function loadGitBranch() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      const status = await invoke<any>('git_status', { repoPath: rootPath });
      if (status && status.branch) {
        branch = status.branch;
      }
    } catch (error) {
      // Git not available or not a git repo
      branch = '';
    }
  }

  function updateCursorPosition() {
    const editor = $editorStore.monacoInstance;
    if (!editor) return;

    const position = editor.getPosition();
    if (position) {
      line = position.lineNumber;
      column = position.column;
    }

    // Update language
    const model = editor.getModel();
    if (model) {
      const languageId = model.getLanguageId();
      // Capitalize first letter
      language = languageId.charAt(0).toUpperCase() + languageId.slice(1);
    }
  }

  onMount(() => {
    updateProblemCounts();
    updateInterval = window.setInterval(updateProblemCounts, 1000);
    loadGitBranch();

    // Listen for cursor position changes
    const editor = $editorStore.monacoInstance;
    if (editor) {
      cursorUpdateUnsubscribe = editor.onDidChangeCursorPosition(() => {
        updateCursorPosition();
      });
      updateCursorPosition();
    }
  });

  onDestroy(() => {
    if (updateInterval) {
      clearInterval(updateInterval);
    }
    if (cursorUpdateUnsubscribe) {
      cursorUpdateUnsubscribe.dispose();
    }
  });

  $: if ($editorStore.monacoInstance) {
    updateProblemCounts();
    updateCursorPosition();

    // Setup cursor listener for new editor instance
    if (cursorUpdateUnsubscribe) {
      cursorUpdateUnsubscribe.dispose();
    }
    cursorUpdateUnsubscribe = $editorStore.monacoInstance.onDidChangeCursorPosition(() => {
      updateCursorPosition();
    });
  }

  $: if ($workspaceStore.rootPath) {
    loadGitBranch();
  }
</script>

<div class="status-bar">
  <div class="status-left">
    {#if branch}
      <button class="status-item" title="Source Control">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M11.5 1a3.5 3.5 0 1 1 0 7 3.5 3.5 0 0 1 0-7zm0 1a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5zM4.5 8a3.5 3.5 0 1 1 0 7 3.5 3.5 0 0 1 0-7zm0 1a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5zM8 5.5a.5.5 0 0 1 .5.5v5a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5z"/>
        </svg>
        {branch}
      </button>
    {/if}

    <button class="status-item" title="Toggle Problems Panel" on:click={togglePanel}>
      {#if errorCount > 0}
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z"/>
          <path d="M7.002 11a1 1 0 1 1 2 0 1 1 0 0 1-2 0zM7.1 4.995a.905.905 0 1 1 1.8 0l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 4.995z"/>
        </svg>
        {errorCount}
      {/if}
      {#if warningCount > 0}
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8.982 1.566a1.13 1.13 0 0 0-1.96 0L.165 13.233c-.457.778.091 1.767.98 1.767h13.713c.889 0 1.438-.99.98-1.767L8.982 1.566zM8 5c.535 0 .954.462.9.995l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 5.995A.905.905 0 0 1 8 5zm.002 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z"/>
        </svg>
        {warningCount}
      {/if}
      {#if errorCount === 0 && warningCount === 0}
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M10.97 4.97a.75.75 0 0 1 1.07 1.05l-3.99 4.99a.75.75 0 0 1-1.08.02L4.324 8.384a.75.75 0 1 1 1.06-1.06l2.094 2.093 3.473-4.425a.267.267 0 0 1 .02-.022z"/>
        </svg>
        No Problems
      {/if}
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

    <button class="status-item" title="Feedback">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M2 1a1 1 0 0 0-1 1v8a1 1 0 0 0 1 1h9.586a2 2 0 0 1 1.414.586l2 2V2a1 1 0 0 0-1-1H2zm12-1a2 2 0 0 1 2 2v12.793a.5.5 0 0 1-.854.353l-2.853-2.853a1 1 0 0 0-.707-.293H2a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2h12z"/>
      </svg>
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
