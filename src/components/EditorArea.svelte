<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { editorStore } from '../stores/editor';
  import { settingsStore } from '../lib/settings-store';
  import { debugStore } from '../stores/debug';
  import { File, FileCode, FileJson, FileText } from 'lucide-svelte';
  import { initializeMonaco } from '../main';

  let editorContainer: HTMLDivElement;
  let editor: any = null; // Will be monaco.editor.IStandaloneCodeEditor
  let monaco: any = null; // Loaded dynamically
  let fileWatchUnlisten: (() => void) | null = null;
  let isProgrammaticChange = false;
  let autoSaveTimeout: number | null = null;
  let monacoLoading = true;
  let breakpointDecorations: string[] = [];
  const decorationCache = new Map<string, Map<string, any[]>>();
  const decorationHandles = new Map<string, string[]>();
  let decorationsListener: ((event: Event) => void) | null = null;
  let lastActivePath: string | null = null;

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
    // Lazy load Monaco Editor
    try {
      await initializeMonaco();
      monaco = await import('monaco-editor');
      monacoLoading = false;
    } catch (error) {
      console.error('Failed to load Monaco Editor:', error);
      return;
    }

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

    // Get current settings
    const currentSettings = $settingsStore;

    // Map theme setting to Monaco theme
    const getMonacoTheme = (theme: string) => {
      switch (theme) {
        case 'light': return 'vs-light';
        case 'dark': return 'vs-dark';
        case 'high-contrast': return 'hc-black';
        default: return 'vs-dark';
      }
    };

    // Configure Monaco Editor
    editor = monaco.editor.create(editorContainer, {
      value: '',
      language: 'typescript',
      theme: getMonacoTheme(currentSettings.theme.colorTheme),
      automaticLayout: true,
      fontSize: currentSettings.editor.fontSize,
      fontFamily: currentSettings.editor.fontFamily,
      minimap: {
        enabled: currentSettings.editor.minimap,
      },
      lineNumbers: currentSettings.editor.lineNumbers,
      rulers: currentSettings.editor.rulers,
      roundedSelection: false,
      scrollBeyondLastLine: false,
      readOnly: false,
      cursorStyle: 'line',
      wordWrap: currentSettings.editor.wordWrap,
      tabSize: currentSettings.editor.tabSize,
      insertSpaces: currentSettings.editor.insertSpaces,
      glyphMargin: true,
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

    // Subscribe to settings changes and update editor
    settingsStore.subscribe((settings) => {
      if (!editor) return;

      // Update editor options
      editor.updateOptions({
        fontSize: settings.editor.fontSize,
        fontFamily: settings.editor.fontFamily,
        tabSize: settings.editor.tabSize,
        insertSpaces: settings.editor.insertSpaces,
        wordWrap: settings.editor.wordWrap,
        lineNumbers: settings.editor.lineNumbers,
        minimap: {
          enabled: settings.editor.minimap,
        },
        rulers: settings.editor.rulers,
      });

      // Update theme
      const monacoTheme = getMonacoTheme(settings.theme.colorTheme);
      monaco.editor.setTheme(monacoTheme);
    });

    // Subscribe to breakpoint changes and update decorations
    debugStore.subscribe((debugState) => {
      if (!editor || !activeTab) return;

      const fileBreakpoints = debugState.breakpoints.get(activeTab.path) || [];

      // Create decorations for breakpoints
      const decorations = fileBreakpoints.map(bp => ({
        range: new monaco.Range(bp.line, 1, bp.line, 1),
        options: {
          isWholeLine: false,
          glyphMarginClassName: bp.enabled ? 'breakpoint-glyph' : 'breakpoint-glyph-disabled',
          glyphMarginHoverMessage: { value: bp.enabled ? 'Breakpoint' : 'Disabled breakpoint' },
        }
      }));

      // Update decorations
      breakpointDecorations = editor.deltaDecorations(breakpointDecorations, decorations);
    });

    // Add glyph margin click handler for toggling breakpoints
    editor.onMouseDown((e: any) => {
      const target = e.target;
      if (target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) {
        if (activeTab) {
          const lineNumber = target.position.lineNumber;
          debugStore.toggleBreakpoint(activeTab.path, lineNumber);

          // Sync with backend
          syncBreakpointsWithBackend(activeTab.path);
        }
      }
    });

    // Listen for content changes
  editor.onDidChangeModelContent((_e: any) => {
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

    decorationsListener = (event: Event) => {
      handleDecorationEvent(event as CustomEvent<any>);
    };
    window.addEventListener('editor-decorations', decorationsListener as EventListener);

    // Listen for file change events
    fileWatchUnlisten = await listen('file-changed', async (event: any) => {
      const { path } = event.payload;
      await handleFileChanged(path);
    });

    if (activeTab) {
      applyDecorationsForPath(activeTab.path);
      lastActivePath = activeTab.path;
    }
  });

  editor.onDidChangeCursorSelection((e: any) => {
    emitSelectionChanged(e);
  });

  editor.onDidScrollChange(() => {
    emitVisibleRangesChanged();
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

  $: if (editor) {
    const currentPath = activeTab?.path ?? null;
    if (currentPath !== lastActivePath) {
      clearAllDecorations();
      if (currentPath) {
        applyDecorationsForPath(currentPath);
      }
      lastActivePath = currentPath;
    }
  }

  onDestroy(() => {
    // Dispose Monaco editor
    if (editor) {
      editor.dispose();
    }

    if (decorationsListener) {
      window.removeEventListener('editor-decorations', decorationsListener as EventListener);
      decorationsListener = null;
    }

    clearAllDecorations();

    // Clean up file watch listener
    if (fileWatchUnlisten) {
      fileWatchUnlisten();
    }

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

  async function syncBreakpointsWithBackend(filePath: string) {
    try {
      const breakpoints = debugStore.getBreakpoints(filePath);
      const backendBreakpoints = breakpoints.map(bp => ({
        line: bp.line,
        condition: bp.condition || null,
      }));

      // Call backend to set breakpoints for this file
      await invoke('set_breakpoints', {
        filePath,
        breakpoints: backendBreakpoints,
      });

      console.log('[Debug] Synced breakpoints for', filePath, backendBreakpoints);
    } catch (error) {
      console.error('[Debug] Failed to sync breakpoints:', error);
    }
  }

  function getSelectionPayload(selection: any) {
    return {
      start: { line: selection.startLineNumber - 1, character: selection.startColumn - 1 },
      end: { line: selection.endLineNumber - 1, character: selection.endColumn - 1 },
      anchor: { line: selection.selectionStartLineNumber - 1, character: selection.selectionStartColumn - 1 },
      active: { line: selection.positionLineNumber - 1, character: selection.positionColumn - 1 },
      isReversed: selection.direction === monaco.SelectionDirection.RTL,
    };
  }

  function emitSelectionChanged(event: any) {
    if (!activeTab) return;
    const primary = getSelectionPayload(event.selection);
    const secondary = event.secondarySelections?.map(getSelectionPayload) ?? [];

    invoke('editor_selection_changed', {
      uri: activeTab.path,
      selection: primary,
      selections: [primary, ...secondary],
    }).catch((err) => console.error('Failed to send selection change:', err));
  }

  function emitVisibleRangesChanged() {
    if (!activeTab || !editor) return;
    const model = editor.getModel();
    if (!model) return;

    const ranges = editor.getVisibleRanges().map((range: any) => ({
      start: { line: range.startLineNumber - 1, character: range.startColumn - 1 },
      end: { line: range.endLineNumber - 1, character: range.endColumn - 1 },
    }));

    invoke('editor_visible_ranges_changed', {
      uri: activeTab.path,
      ranges,
    }).catch((err) => console.error('Failed to send visible ranges:', err));
  }

  function handleDecorationEvent(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail || !detail.uri || !detail.key) {
      return;
    }

    const entries = Array.isArray(detail.decorations) ? detail.decorations : [];
    let cacheForFile = decorationCache.get(detail.uri);
    if (!cacheForFile) {
      cacheForFile = new Map();
      decorationCache.set(detail.uri, cacheForFile);
    }

    cacheForFile.set(detail.key, entries);

    if (activeTab && activeTab.path === detail.uri) {
      applyDecorationKey(detail.key, entries);
    }
  }

  function applyDecorationsForPath(path: string) {
    if (!path) return;
    const cacheForFile = decorationCache.get(path);
    if (!cacheForFile) {
      return;
    }

    for (const [key, entries] of cacheForFile.entries()) {
      applyDecorationKey(key, entries);
    }
  }

  function applyDecorationKey(key: string, entries: any[]) {
    if (!editor || !monaco) return;
    const decorations = (entries || [])
      .map((entry) => {
        const rangeData = entry.range ?? entry;
        if (!rangeData || !rangeData.start || !rangeData.end) {
          return null;
        }
        const options = entry.renderOptions ?? entry.options ?? {};
        return {
          range: new monaco.Range(
            rangeData.start.line + 1,
            rangeData.start.character + 1,
            rangeData.end.line + 1,
            rangeData.end.character + 1
          ),
          options,
        };
      })
      .filter(Boolean);

    const existing = decorationHandles.get(key) ?? [];
    const newHandles = editor.deltaDecorations(existing, decorations);
    decorationHandles.set(key, newHandles);
  }

  function clearAllDecorations() {
    if (!editor) return;
    decorationHandles.forEach((handles, key) => {
      const cleared = editor.deltaDecorations(handles, []);
      decorationHandles.set(key, cleared);
    });
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

  <div class="editor-container" bind:this={editorContainer}>
    {#if monacoLoading}
      <div class="editor-loading">
        <div class="loading-spinner"></div>
        <p>Loading editor...</p>
      </div>
    {/if}
  </div>

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

  .editor-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .editor-loading p {
    color: var(--color-text-secondary);
    font-size: 14px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Breakpoint glyph margin styles */
  :global(.breakpoint-glyph) {
    background: #e51400;
    width: 10px !important;
    height: 10px !important;
    border-radius: 50%;
    margin-left: 4px;
    margin-top: 6px;
  }

  :global(.breakpoint-glyph-disabled) {
    background: #888;
    width: 10px !important;
    height: 10px !important;
    border-radius: 50%;
    margin-left: 4px;
    margin-top: 6px;
    opacity: 0.5;
  }
</style>
