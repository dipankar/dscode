<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import * as monaco from 'monaco-editor';
  import { editorStore } from '../stores/editor';
  import { File, FileCode, FileJson, FileText } from 'lucide-svelte';

  let editorContainer: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;
  let fileWatchUnlisten: (() => void) | null = null;
  let isProgrammaticChange = false;
  let autoSaveTimeout: number | null = null;

  $: tabs = $editorStore.tabs;
  $: activeTabId = $editorStore.activeTabId;
  $: activeTab = tabs.find((t) => t.id === activeTabId);
  $: activeFile = activeTab
    ? $editorStore.openFiles.get(activeTab.path)
    : null;
  $: breadcrumbs = activeTab ? activeTab.path.split('/').filter(Boolean) : [];

  // Update editor when active file changes
  $: if (editor && activeFile) {
    isProgrammaticChange = true;
    editor.setValue(activeFile.content);
    const model = editor.getModel();
    if (model) {
      monaco.editor.setModelLanguage(model, activeFile.language);
    }
    // Reset flag after a short delay to allow the change event to process
    setTimeout(() => {
      isProgrammaticChange = false;
    }, 10);
  }

  onMount(async () => {
    // Configure TypeScript/JavaScript language defaults
    monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
      target: monaco.languages.typescript.ScriptTarget.ES2020,
      allowNonTsExtensions: true,
      moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
      module: monaco.languages.typescript.ModuleKind.CommonJS,
      noEmit: true,
      esModuleInterop: true,
      jsx: monaco.languages.typescript.JsxEmit.React,
      reactNamespace: 'React',
      allowJs: true,
      typeRoots: ['node_modules/@types'],
    });

    monaco.languages.typescript.javascriptDefaults.setCompilerOptions({
      target: monaco.languages.typescript.ScriptTarget.ES2020,
      allowNonTsExtensions: true,
      moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
      module: monaco.languages.typescript.ModuleKind.CommonJS,
      noEmit: true,
      esModuleInterop: true,
      allowJs: true,
    });

    // Enable diagnostics
    monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions({
      noSemanticValidation: false,
      noSyntaxValidation: false,
      diagnosticCodesToIgnore: [],
    });

    monaco.languages.typescript.javascriptDefaults.setDiagnosticsOptions({
      noSemanticValidation: false,
      noSyntaxValidation: false,
      diagnosticCodesToIgnore: [],
    });

    // Configure Monaco Editor
    editor = monaco.editor.create(editorContainer, {
      value: '',
      language: 'typescript',
      theme: 'vs-dark',
      automaticLayout: true,
      fontSize: 14,
      minimap: {
        enabled: true,
      },
      lineNumbers: 'on',
      roundedSelection: false,
      scrollBeyondLastLine: false,
      readOnly: false,
      cursorStyle: 'line',
      wordWrap: 'off',
      tabSize: 2,
      insertSpaces: true,
      // Enable IntelliSense features
      suggestOnTriggerCharacters: true,
      quickSuggestions: true,
      parameterHints: {
        enabled: true,
      },
      suggest: {
        showMethods: true,
        showFunctions: true,
        showConstructors: true,
        showFields: true,
        showVariables: true,
        showClasses: true,
        showStructs: true,
        showInterfaces: true,
        showModules: true,
        showProperties: true,
        showEvents: true,
        showOperators: true,
        showUnits: true,
        showValues: true,
        showConstants: true,
        showEnums: true,
        showEnumMembers: true,
        showKeywords: true,
        showWords: true,
        showColors: true,
        showFiles: true,
        showReferences: true,
        showFolders: true,
        showTypeParameters: true,
        showSnippets: true,
      },
    });

    // Store Monaco instance in store
    editorStore.setMonacoInstance(editor);

    // Listen for content changes
    editor.onDidChangeModelContent((e) => {
      // Ignore programmatic changes (like when opening/switching files)
      if (isProgrammaticChange) {
        return;
      }

      if (activeTab) {
        const content = editor.getValue();
        editorStore.updateContent(activeTab.path, content);
        editorStore.markDirty(activeTab.path);

        // Auto-save logic
        if ($editorStore.autoSaveEnabled) {
          // Clear previous timeout
          if (autoSaveTimeout !== null) {
            clearTimeout(autoSaveTimeout);
          }

          // Set new timeout for auto-save
          autoSaveTimeout = window.setTimeout(() => {
            saveCurrentFile();
          }, $editorStore.autoSaveDelay);
        }
      }
    });

    // Keyboard shortcuts
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      saveCurrentFile();
    });

    // Save All (Ctrl+K S)
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyK, () => {
      // Need to handle Ctrl+K chord
    });

    // For now, use Ctrl+Shift+S for Save All
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyS, () => {
      saveAllFiles();
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyW, () => {
      if (activeTab) {
        handleTabClose(null, activeTab.id);
      }
    });

    // Listen for file change events
    fileWatchUnlisten = await listen('file-changed', async (event: any) => {
      const { path } = event.payload;
      await handleFileChanged(path);
    });

    return () => {
      editor.dispose();
      if (fileWatchUnlisten) {
        fileWatchUnlisten();
      }
    };
  });

  async function handleFileChanged(changedPath: string) {
    // Check if this file is open
    const openFile = $editorStore.openFiles.get(changedPath);
    if (!openFile) return;

    // Skip if file has unsaved changes
    if (openFile.isDirty) {
      console.log('File changed externally but has unsaved changes:', changedPath);
      return;
    }

    try {
      // Reload file content
      const content = await invoke<string>('read_file', { path: changedPath });

      // Update content in store
      editorStore.updateContent(changedPath, content);

      // If this is the active tab, update Monaco editor
      if (activeTab && activeTab.path === changedPath) {
        const currentPosition = editor.getPosition();
        editor.setValue(content);
        if (currentPosition) {
          editor.setPosition(currentPosition);
        }
      }

      console.log('File reloaded from external change:', changedPath);
    } catch (error) {
      console.error('Failed to reload file:', error);
    }
  }

  // Watch files when tabs change
  $: {
    if (tabs.length > 0) {
      tabs.forEach(async (tab) => {
        try {
          await invoke('start_watching_file', { path: tab.path });
        } catch (error) {
          console.error('Failed to watch file:', tab.path, error);
        }
      });
    }
  }

  onDestroy(() => {
    // Clean up auto-save timeout
    if (autoSaveTimeout !== null) {
      clearTimeout(autoSaveTimeout);
    }

    // Stop watching all files
    tabs.forEach(async (tab) => {
      try {
        await invoke('stop_watching_file', { path: tab.path });
      } catch (error) {
        console.error('Failed to stop watching file:', tab.path, error);
      }
    });
  });

  async function saveCurrentFile() {
    if (!activeFile || !activeTab) return;

    try {
      const content = editor.getValue();
      await invoke('write_file', {
        path: activeTab.path,
        content,
      });

      editorStore.markClean(activeTab.path);
      console.log('File saved:', activeTab.path);
    } catch (error) {
      console.error('Failed to save file:', error);
    }
  }

  async function saveAllFiles() {
    await editorStore.saveAll(async (path: string, content: string) => {
      await invoke('write_file', { path, content });
      console.log('File saved:', path);
    });
  }

  function handleTabClick(tabId: string) {
    editorStore.switchTab(tabId);
  }

  async function handleTabClose(e: MouseEvent | null, tabId: string) {
    if (e) e.stopPropagation();

    // Find the tab
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab) return;

    // Check if tab has unsaved changes
    if (tab.isDirty) {
      const filename = tab.label;
      const response = confirm(
        `Do you want to save the changes you made to ${filename}?\n\n` +
        `Your changes will be lost if you don't save them.`
      );

      if (response === null) {
        // User cancelled
        return;
      } else if (response === true) {
        // User wants to save
        const file = $editorStore.openFiles.get(tab.path);
        if (file) {
          try {
            await invoke('write_file', {
              path: tab.path,
              content: file.content,
            });
            editorStore.markClean(tab.path);
          } catch (error) {
            console.error('Failed to save file:', error);
            alert(`Failed to save file: ${error}`);
            return;
          }
        }
      }
      // If response is false, proceed to close without saving
    }

    // Close the tab
    editorStore.closeTab(tabId, true);
  }
</script>

<div class="editor-area">
  <div class="editor-tabs">
    {#if tabs.length === 0}
      <div class="empty-tabs">
        <span class="empty-message">Open a file to start editing</span>
      </div>
    {:else}
      {#each tabs as tab}
        <button
          class="tab"
          class:active={tab.id === activeTabId}
          on:click={() => handleTabClick(tab.id)}
        >
          <span class="tab-icon">
            {#if tab.label.endsWith('.rs')}
              <FileCode size={14} />
            {:else if tab.label.endsWith('.ts')}
              <FileCode size={14} />
            {:else if tab.label.endsWith('.js')}
              <FileCode size={14} />
            {:else if tab.label.endsWith('.svelte')}
              <FileCode size={14} />
            {:else if tab.label.endsWith('.json')}
              <FileJson size={14} />
            {:else if tab.label.endsWith('.md')}
              <FileText size={14} />
            {:else}
              <File size={14} />
            {/if}
          </span>
          <span class="tab-label">{tab.label}</span>
          {#if tab.isDirty}
            <span class="dirty-indicator">●</span>
          {/if}
          <button
            class="tab-close"
            on:click={(e) => handleTabClose(e, tab.id)}
            title="Close (Ctrl+W)"
          >
            ×
          </button>
        </button>
      {/each}
    {/if}
  </div>

  {#if breadcrumbs.length > 0}
    <div class="breadcrumbs">
      {#each breadcrumbs as crumb, i}
        {#if i > 0}
          <span class="breadcrumb-separator">/</span>
        {/if}
        <span class="breadcrumb-item" class:last={i === breadcrumbs.length - 1}>
          {crumb}
        </span>
      {/each}
    </div>
  {/if}

  <div class="editor-container" bind:this={editorContainer}></div>

  {#if activeFile && activeFile.isDirty}
    <div class="save-indicator">
      Unsaved changes - Press Ctrl+S to save
    </div>
  {/if}
</div>

<style>
  .editor-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--editor-bg);
    position: relative;
  }

  .editor-tabs {
    display: flex;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    height: 35px;
    overflow-x: auto;
  }

  .empty-tabs {
    display: flex;
    align-items: center;
    padding: 0 12px;
  }

  .empty-message {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 12px;
    height: 35px;
    background-color: var(--color-bg-tertiary);
    border: none;
    border-right: 1px solid var(--color-border);
    color: var(--color-text);
    cursor: pointer;
    user-select: none;
    min-width: 120px;
    max-width: 200px;
  }

  .tab.active {
    background-color: var(--editor-bg);
  }

  .tab:hover {
    background-color: var(--color-bg-tertiary);
  }

  .tab.active:hover {
    background-color: var(--editor-bg);
  }

  .tab:hover .tab-close {
    opacity: 1;
  }

  .tab-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .tab-label {
    font-size: 13px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dirty-indicator {
    color: var(--color-accent);
    font-size: 18px;
    line-height: 1;
  }

  .tab-close {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 18px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s;
    padding: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .tab-close:hover {
    color: var(--color-text);
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  .tab.active .tab-close {
    opacity: 0.7;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
    width: 100%;
    height: 100%;
    min-height: 0;
  }

  .save-indicator {
    position: absolute;
    bottom: 8px;
    right: 8px;
    background-color: var(--color-accent);
    color: white;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 12px;
    pointer-events: none;
    animation: fadeIn 0.3s;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .breadcrumbs {
    display: flex;
    align-items: center;
    height: 22px;
    padding: 0 12px;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-text-secondary);
    overflow: hidden;
  }

  .breadcrumb-item {
    white-space: nowrap;
  }

  .breadcrumb-item.last {
    color: var(--color-text);
    font-weight: 500;
  }

  .breadcrumb-separator {
    margin: 0 6px;
    color: var(--color-text-secondary);
    opacity: 0.5;
  }
</style>
