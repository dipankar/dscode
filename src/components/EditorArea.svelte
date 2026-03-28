<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { editorStore, isDocumentDirty } from '../stores/editor';
  import { settingsStore } from '../lib/settings-store';
  import { debugStore } from '../stores/debug';
  import { File, FileCode, FileJson, FileText } from 'lucide-svelte';
  import { initializeMonaco } from '../main';
  import ContextMenu from './ContextMenu.svelte';
  import { systemCommands } from '../lib/contracts/commands';
  import { WindowEventName } from '../lib/contracts/events';
  import { languageFeaturesManager } from '../lib/language-features';
  import { showAlertPrompt, showConfirmPrompt } from '../stores/windowPrompt';

  let editorContainer: HTMLDivElement;
  let editorTabsContainer: HTMLDivElement;
  let editor: any = null;
  let monaco: any = null;
  let fileWatchUnlisten: (() => void) | null = null;
  let isProgrammaticChange = false;
  let autoSaveTimeout: number | null = null;
  let monacoLoading = true;
  let breakpointDecorations: string[] = [];
  const decorationCache = new Map<string, Map<string, any[]>>();
  const decorationHandles = new Map<string, string[]>();
  let decorationsListener: ((event: Event) => void) | null = null;
  let lastActivePath: string | null = null;
  let settingsUnsubscribe: (() => void) | null = null;
  let debugUnsubscribe: (() => void) | null = null;
  let watchedFiles = new Set<string>();
  const modelCache = new Map<string, any>();
  const viewStates = new Map<string, any>();
  const documentVersions: Record<string, number> = {};

  // Context menu state
  let contextMenuVisible = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let contextMenuContext: any = {};

  $: tabs = $editorStore.tabs;
  $: activeTabId = $editorStore.activeTabId;
  $: activeTab = tabs.find((t) => t.id === activeTabId);
  $: activeFile = activeTab ? $editorStore.openFiles.get(activeTab.path) : null;
  $: breadcrumbs = activeTab ? activeTab.path.split('/').filter(Boolean) : [];

  function getOrCreateModel(path: string, content: string, language: string): any {
    const uri = monaco.Uri.parse(`file://${path}`);
    let model = monaco.editor.getModel(uri);
    if (!model) {
      model = monaco.editor.createModel(content, language, uri);
    }
    return model;
  }

  // Update editor when active file changes
  $: if (editor && monaco && activeFile && activeTab) {
    const model = getOrCreateModel(activeTab.path, activeFile.content, activeFile.language);
    const currentModel = editor.getModel();
    if (currentModel !== model) {
      const savedViewState = editor.saveViewState();
      if (currentModel) {
        viewStates.set(currentModel.uri.toString(), savedViewState);
      }
      editor.setModel(model);
      const cached = viewStates.get(model.uri.toString());
      if (cached) {
        editor.restoreViewState(cached);
      }
    }
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
        case 'light':
          return 'vs-light';
        case 'dark':
          return 'vs-dark';
        case 'high-contrast':
          return 'hc-black';
        default:
          return 'vs-dark';
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

    // Initialize language features manager
    try {
      await languageFeaturesManager.initialize(editor);
      console.log('[EditorArea] Language features initialized');
    } catch (error) {
      console.error('[EditorArea] Failed to initialize language features:', error);
    }

    // Subscribe to settings changes and update editor
    settingsUnsubscribe = settingsStore.subscribe((settings) => {
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
    debugUnsubscribe = debugStore.subscribe((debugState) => {
      if (!editor || !activeTab) return;

      const fileBreakpoints = debugState.breakpoints.get(activeTab.path) || [];

      // Create decorations for breakpoints
      const decorations = fileBreakpoints.map((bp) => ({
        range: new monaco.Range(bp.line, 1, bp.line, 1),
        options: {
          isWholeLine: false,
          glyphMarginClassName: bp.enabled ? 'breakpoint-glyph' : 'breakpoint-glyph-disabled',
          glyphMarginHoverMessage: { value: bp.enabled ? 'Breakpoint' : 'Disabled breakpoint' },
        },
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

    // Handle context menu (right-click)
    editor.onContextMenu((e: any) => {
      e.event.preventDefault();

      // Get editor selection state
      const selection = editor.getSelection();
      const hasSelection = selection && !selection.isEmpty();

      // Get file extension
      const fileExt = activeTab?.path.split('.').pop();

      // Build context menu context
      contextMenuContext = {
        has_selection: hasSelection,
        editor_focused: true,
        explorer_focused: false,
        resource_extension: fileExt,
        resource_path: activeTab?.path,
        language_id: activeFile?.language,
        in_debug_mode: false,
      };

      // Show context menu at mouse position
      contextMenuX = e.event.posx;
      contextMenuY = e.event.posy;
      contextMenuVisible = true;
    });

    // Listen for content changes
    editor.onDidChangeModelContent((e: any) => {
      if (activeTab) {
        const content = editor.getValue();
        editorStore.updateContent(activeTab.path, content);
        editorStore.markDirty(activeTab.path);

        if (!isProgrammaticChange) {
          const changes = e.changes.map((change: any) => ({
            range: change.range
              ? {
                  start: {
                    line: change.range.startLineNumber - 1,
                    character: change.range.startColumn - 1,
                  },
                  end: {
                    line: change.range.endLineNumber - 1,
                    character: change.range.endColumn - 1,
                  },
                }
              : null,
            rangeOffset: change.rangeOffset,
            rangeLength: change.rangeLength,
            text: change.text,
          }));
          const version = documentVersions[activeTab.path] || 0;
          const newVersion = version + 1;
          documentVersions[activeTab.path] = newVersion;
          invoke('update_text_document', {
            uri: activeTab.path,
            version: newVersion,
            contentChanges: changes,
          }).catch((err: any) => {
            console.warn('[Editor] Failed to sync document change:', err);
          });
        }

        if ($editorStore.autoSaveEnabled) {
          if (autoSaveTimeout !== null) {
            clearTimeout(autoSaveTimeout);
          }
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
    window.addEventListener(
      WindowEventName.editorDecorations,
      decorationsListener as EventListener
    );

    // Listen for file change events
    fileWatchUnlisten = await listen('file-changed', async (event: any) => {
      const { path } = event.payload;
      await handleFileChanged(path);
    });

    if (activeTab) {
      applyDecorationsForPath(activeTab.path);
      lastActivePath = activeTab.path;
    }

    // Set up editor event listeners
    editor.onDidChangeCursorSelection((e: any) => {
      emitSelectionChanged(e);
    });

    editor.onDidScrollChange(() => {
      emitVisibleRangesChanged();
    });
  });

  async function handleFileChanged(changedPath: string) {
    const openFile = $editorStore.openFiles.get(changedPath);
    if (!openFile) return;

    if (isDocumentDirty(openFile.state)) return;

    try {
      const content = await invoke<string>('read_file', { path: changedPath });
      editorStore.updateContent(changedPath, content);

      const uri = monaco.Uri.parse(`file://${changedPath}`);
      const model = monaco.editor.getModel(uri);
      if (model) {
        model.setValue(content);
      }
    } catch (error) {
      console.error('Failed to reload file:', error);
    }
  }

  // Watch files when tabs change (only new files)
  $: {
    const currentPaths = new Set(tabs.map((t) => t.path));
    for (const path of currentPaths) {
      if (!watchedFiles.has(path)) {
        watchedFiles.add(path);
        invoke('start_watching_file', { path }).catch((err: any) => {
          console.error('Failed to watch file:', path, err);
        });
      }
    }
    for (const path of watchedFiles) {
      if (!currentPaths.has(path)) {
        invoke('stop_watching_file', { path }).catch((err: any) => {
          console.error('Failed to stop watching file:', path, err);
        });
      }
    }
    watchedFiles = currentPaths;
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

  $: if (activeTabId && editorTabsContainer) {
    requestAnimationFrame(() => {
      const activeEl = editorTabsContainer.querySelector('.tab.active');
      if (activeEl) {
        activeEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
      }
    });
  }

  onDestroy(() => {
    if (editor) {
      const currentModel = editor.getModel();
      if (currentModel) {
        viewStates.set(currentModel.uri.toString(), editor.saveViewState());
      }
      editor.dispose();
    }

    if (decorationsListener) {
      window.removeEventListener(
        WindowEventName.editorDecorations,
        decorationsListener as EventListener
      );
      decorationsListener = null;
    }

    clearAllDecorations();

    if (fileWatchUnlisten) {
      fileWatchUnlisten();
    }

    if (settingsUnsubscribe) settingsUnsubscribe();
    if (debugUnsubscribe) debugUnsubscribe();

    if (autoSaveTimeout !== null) {
      clearTimeout(autoSaveTimeout);
    }

    for (const path of watchedFiles) {
      invoke('stop_watching_file', { path }).catch(() => {});
    }

    modelCache.forEach((model: any) => {
      if (model && model.dispose) model.dispose();
    });
    modelCache.clear();
    viewStates.clear();
  });

  async function saveCurrentFile() {
    if (!activeFile || !activeTab) return;

    try {
      const content = editor.getValue();
      await systemCommands.writeFile(activeTab.path, content);
      editorStore.markClean(activeTab.path);
    } catch (error) {
      await showAlertPrompt(`Failed to save file: ${error}`, { level: 'error' });
    }
  }

  async function saveAllFiles() {
    await editorStore.saveAll(async (path: string, content: string) => {
      await systemCommands.writeFile(path, content);
      console.log('File saved:', path);
    });
  }

  function handleTabClick(tabId: string) {
    editorStore.switchTab(tabId);
  }

  async function handleTabClose(e: MouseEvent | null, tabId: string) {
    if (e) e.stopPropagation();

    const tab = tabs.find((t) => t.id === tabId);
    if (!tab) return;

    if (tab.isDirty) {
      const filename = tab.label;
      const action = await showConfirmPrompt(
        `Do you want to save the changes you made to ${filename}?\n\nYour changes will be lost if you don't save them.`,
        { level: 'warning', confirmLabel: 'Save', cancelLabel: "Don't Save" }
      );

      if (action === true) {
        const file = $editorStore.openFiles.get(tab.path);
        if (file) {
          try {
            await systemCommands.writeFile(tab.path, file.content);
            editorStore.markClean(tab.path);
          } catch (error) {
            await showAlertPrompt(`Failed to save file: ${error}`, { level: 'error' });
            return;
          }
        }
      } else if (action === false) {
        // "Don't Save" — proceed to close without saving
      } else {
        // Cancelled (undefined) — don't close
        return;
      }
    }

    if (editor) {
      const currentModel = editor.getModel();
      if (currentModel) {
        viewStates.set(currentModel.uri.toString(), editor.saveViewState());
      }
    }

    editorStore.closeTab(tabId, true);
  }

  async function syncBreakpointsWithBackend(filePath: string) {
    try {
      const breakpoints = debugStore.getBreakpoints(filePath);
      const backendBreakpoints = breakpoints.map((bp) => ({
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
      anchor: {
        line: selection.selectionStartLineNumber - 1,
        character: selection.selectionStartColumn - 1,
      },
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
  <div class="editor-tabs" bind:this={editorTabsContainer}>
    {#if tabs.length === 0}
      <div class="empty-tabs">
        <span class="empty-message">Open a file to start editing</span>
      </div>
    {:else}
      {#each tabs as tab}
        <div
          class="tab"
          class:active={tab.id === activeTabId}
          role="tab"
          tabindex="0"
          on:click={() => handleTabClick(tab.id)}
          on:keydown={(e) => {
            if (e.key === 'Enter') handleTabClick(tab.id);
          }}
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
            <span class="dirty-indicator">&#9679;</span>
          {/if}
          <button
            class="tab-close"
            on:click|stopPropagation={(e) => handleTabClose(e, tab.id)}
            title="Close (Ctrl+W)"
          >
            &times;
          </button>
        </div>
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

  {#if activeFile && isDocumentDirty(activeFile.state)}
    <div class="save-indicator">Unsaved changes - Press Ctrl+S to save</div>
  {/if}
</div>

<!-- Context Menu -->
<ContextMenu
  visible={contextMenuVisible}
  x={contextMenuX}
  y={contextMenuY}
  location="editor/context"
  context={contextMenuContext}
  onClose={() => (contextMenuVisible = false)}
/>

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
    font-size: 13px;
    font-family: inherit;
  }

  .tab:hover .tab-close {
    opacity: 1;
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
    background-color: var(--color-surface-hover);
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
    color: var(--color-text-on-accent);
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
    background: var(--color-badge-error);
    width: 10px !important;
    height: 10px !important;
    border-radius: 50%;
    margin-left: 4px;
    margin-top: 6px;
  }

  :global(.breakpoint-glyph-disabled) {
    background: var(--color-badge-muted);
    width: 10px !important;
    height: 10px !important;
    border-radius: 50%;
    margin-left: 4px;
    margin-top: 6px;
    opacity: 0.5;
  }
</style>
