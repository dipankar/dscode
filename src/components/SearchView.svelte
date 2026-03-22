<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { workspaceStore } from '../stores/workspace';
  import { editorStore } from '../stores/editor';

  interface SearchResult {
    path: string;
    line: number;
    column: number;
    text: string;
    match: string;
  }

  let searchQuery = '';
  let replaceText = '';
  let searchResults: SearchResult[] = [];
  let searching = false;
  let replacing = false;
  let caseSensitive = false;
  let useRegex = false;
  let wholeWord = false;
  let includePattern = '';
  let excludePattern = '';
  let searchTimeout: number | null = null;
  let showReplace = false;
  let selectedFiles = new Set<string>();

  $: groupedResults = groupResultsByFile(searchResults);

  function groupResultsByFile(results: SearchResult[]): Map<string, SearchResult[]> {
    const map = new Map<string, SearchResult[]>();
    for (const result of results) {
      if (!map.has(result.path)) {
        map.set(result.path, []);
      }
      map.get(result.path)!.push(result);
    }
    return map;
  }

  async function search() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath || !searchQuery.trim()) {
      searchResults = [];
      return;
    }

    // Clear any pending search
    if (searchTimeout) {
      clearTimeout(searchTimeout);
      searchTimeout = null;
    }

    searching = true;
    try {
      const results = await invoke<SearchResult[]>('search_in_files', {
        rootPath,
        options: {
          query: searchQuery,
          caseSensitive,
          useRegex,
          wholeWord,
          includePattern: includePattern || null,
          excludePattern: excludePattern || null,
          maxResults: 500, // Reduced for better performance
        }
      });

      searchResults = results;
    } catch (error) {
      console.error('[Search] Error:', error);
      searchResults = [];
    } finally {
      searching = false;
    }
  }

  function debouncedSearch() {
    // Clear existing timeout
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    // Don't search if query is empty
    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }

    // Set new timeout
    searchTimeout = window.setTimeout(() => {
      search();
    }, 300); // Wait 300ms after user stops typing
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      search();
    }
  }

  async function openResult(result: SearchResult) {
    try {
      const fullPath = `${$workspaceStore.rootPath}/${result.path}`;
      const content = await invoke<string>('read_file', { path: fullPath });
      const language = await invoke<string>('get_file_language', { path: fullPath });

      // Open the file
      editorStore.openFile(fullPath, content, language);

      // Navigate to the specific line and column
      // Add a small delay to ensure editor is ready
      setTimeout(() => {
        editorStore.navigateToLine(result.line, result.column);
      }, 100);
    } catch (error) {
      console.error('[Search] Failed to open file:', error);
    }
  }

  function clearSearch() {
    searchQuery = '';
    searchResults = [];
    selectedFiles.clear();
  }

  async function replaceAll() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath || !searchQuery.trim()) return;

    const confirmed = confirm(`Replace all occurrences of "${searchQuery}" with "${replaceText}"?`);
    if (!confirmed) return;

    replacing = true;
    try {
      const filesModified = await invoke<number>('replace_in_files', {
        rootPath,
        searchQuery,
        replaceText,
        options: {
          query: searchQuery,
          caseSensitive,
          useRegex,
          wholeWord,
          includePattern: includePattern || null,
          excludePattern: excludePattern || null,
          maxResults: 500,
        }
      });

      alert(`Replaced in ${filesModified} file(s)`);

      // Re-run search to update results
      await search();
    } catch (error) {
      console.error('[Replace] Error:', error);
      alert(`Replace failed: ${error}`);
    } finally {
      replacing = false;
    }
  }

  function toggleFileSelection(filePath: string) {
    if (selectedFiles.has(filePath)) {
      selectedFiles.delete(filePath);
    } else {
      selectedFiles.add(filePath);
    }
    selectedFiles = selectedFiles; // Trigger reactivity
  }

  function toggleAllFiles() {
    if (selectedFiles.size === groupedResults.size) {
      selectedFiles.clear();
    } else {
      selectedFiles = new Set([...groupedResults.keys()]);
    }
    selectedFiles = selectedFiles;
  }
</script>

<div class="search-view">
  <div class="search-header">
    <h3>SEARCH</h3>
    <button
      class="toggle-replace-btn"
      class:active={showReplace}
      on:click={() => showReplace = !showReplace}
      title="Toggle Replace"
    >
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8.75 1a.75.75 0 0 0-1.5 0v1.5H5.5a.75.75 0 0 0 0 1.5h1.75v1.75a.75.75 0 0 0 1.5 0V4h1.75a.75.75 0 0 0 0-1.5H8.75V1zM4 7.75A.75.75 0 0 1 4.75 7h6.5a.75.75 0 0 1 0 1.5h-6.5A.75.75 0 0 1 4 7.75zm0 3.5a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5a.75.75 0 0 1-.75-.75z"/>
      </svg>
    </button>
  </div>

  <div class="search-controls">
    <div class="search-input-container">
      <input
        type="text"
        bind:value={searchQuery}
        on:input={debouncedSearch}
        on:keydown={handleKeyDown}
        placeholder="Search in files..."
        class="search-input"
      />
      {#if searchQuery}
        <button class="clear-btn" on:click={clearSearch}>
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8 2.146 2.854Z"/>
          </svg>
        </button>
      {/if}
    </div>

    {#if showReplace}
      <div class="replace-input-container">
        <input
          type="text"
          bind:value={replaceText}
          placeholder="Replace with..."
          class="search-input"
        />
      </div>
    {/if}

    <div class="search-options">
      <button
        class="option-btn"
        class:active={caseSensitive}
        on:click={() => caseSensitive = !caseSensitive}
        title="Match Case"
      >
        Aa
      </button>
      <button
        class="option-btn"
        class:active={wholeWord}
        on:click={() => wholeWord = !wholeWord}
        title="Match Whole Word"
      >
        |ab|
      </button>
      <button
        class="option-btn"
        class:active={useRegex}
        on:click={() => useRegex = !useRegex}
        title="Use Regular Expression"
      >
        .*
      </button>
    </div>

    <div class="filter-inputs">
      <input
        type="text"
        bind:value={includePattern}
        placeholder="files to include (e.g. *.ts, src/**)"
        class="filter-input"
      />
      <input
        type="text"
        bind:value={excludePattern}
        placeholder="files to exclude (e.g. node_modules/**)"
        class="filter-input"
      />
    </div>

    <div class="action-buttons">
      <button
        class="search-btn"
        on:click={search}
        disabled={!searchQuery.trim() || searching}
      >
        {searching ? 'Searching...' : 'Search'}
      </button>
      {#if showReplace && searchResults.length > 0}
        <button
          class="replace-btn"
          on:click={replaceAll}
          disabled={replacing || !searchQuery.trim()}
        >
          {replacing ? 'Replacing...' : 'Replace All'}
        </button>
      {/if}
    </div>
  </div>

  <div class="search-results">
    {#if searching}
      <div class="searching-state">
        <p>Searching...</p>
      </div>
    {:else if searchResults.length > 0}
      <div class="results-summary">
        {searchResults.length} result{searchResults.length !== 1 ? 's' : ''} in {groupedResults.size} file{groupedResults.size !== 1 ? 's' : ''}
      </div>
      <div class="results-list">
        {#each [...groupedResults.entries()] as [path, results]}
          <div class="file-group">
            <div class="file-header">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                <path d="M4 1.75C4 .784 4.784 0 5.75 0h5.586c.464 0 .909.184 1.237.513l2.914 2.914c.329.328.513.773.513 1.237v8.586A1.75 1.75 0 0114.25 15h-9a1.75 1.75 0 01-1.75-1.75V1.75z"/>
              </svg>
              <span class="file-path">{path}</span>
              <span class="result-count">{results.length}</span>
            </div>
            <div class="file-results">
              {#each results as result}
                <button class="result-item" on:click={() => openResult(result)}>
                  <span class="line-number">{result.line}:{result.column}</span>
                  <span class="result-text">{result.text}</span>
                </button>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {:else if searchQuery}
      <div class="no-results">
        <span class="no-results-icon">🔍</span>
        <p>No results found</p>
        <p class="hint">Try a different search term</p>
      </div>
    {:else}
      <div class="empty-state">
        <span class="empty-icon">🔍</span>
        <p>Enter search term to find in files</p>
        <p class="hint">Supports regex and glob patterns</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .search-view {
    width: 100%;
    height: 100%;
    background-color: var(--color-bg-secondary);
    display: flex;
    flex-direction: column;
  }

  .search-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .search-header h3 {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.5px;
  }

  .toggle-replace-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 4px;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    transition: all 0.1s;
  }

  .toggle-replace-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .toggle-replace-btn.active {
    background: var(--accent-color);
    color: white;
    border-color: var(--accent-color);
  }

  .search-controls {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .search-input-container {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-input {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
    padding: 6px 32px 6px 8px;
    font-size: 13px;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .clear-btn {
    position: absolute;
    right: 8px;
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    border-radius: 3px;
  }

  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .search-options {
    display: flex;
    gap: 4px;
  }

  .option-btn {
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 4px 8px;
    border-radius: 3px;
    font-size: 11px;
    font-weight: 600;
    font-family: monospace;
    cursor: pointer;
    transition: all 0.1s;
  }

  .option-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .option-btn.active {
    background: var(--accent-color);
    color: white;
    border-color: var(--accent-color);
  }

  .filter-inputs {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .filter-input {
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
    padding: 4px 8px;
    font-size: 11px;
  }

  .filter-input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .action-buttons {
    display: flex;
    gap: 8px;
  }

  .search-btn,
  .replace-btn {
    flex: 1;
    background: var(--accent-color);
    color: white;
    border: none;
    border-radius: 3px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.1s;
  }

  .replace-btn {
    background: #f9c74f;
  }

  .search-btn:hover:not(:disabled),
  .replace-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .search-btn:disabled,
  .replace-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .replace-input-container {
    display: flex;
    align-items: center;
  }

  .search-results {
    flex: 1;
    overflow-y: auto;
  }

  .searching-state,
  .no-results,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-tertiary);
    text-align: center;
    padding: 20px;
  }

  .empty-icon,
  .no-results-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.5;
  }

  .empty-state p,
  .no-results p {
    margin: 4px 0;
    font-size: 13px;
  }

  .hint {
    font-size: 11px;
    opacity: 0.7;
  }

  .results-summary {
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .results-list {
    padding: 4px 0;
  }

  .file-group {
    margin-bottom: 12px;
  }

  .file-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
    font-size: 12px;
    color: var(--text-primary);
  }

  .file-path {
    flex: 1;
    font-family: 'Fira Code', 'Consolas', monospace;
  }

  .result-count {
    font-size: 10px;
    background: var(--bg-active);
    padding: 2px 6px;
    border-radius: 10px;
    color: var(--text-secondary);
  }

  .file-results {
    padding: 0;
  }

  .result-item {
    width: 100%;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 4px 12px 4px 32px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .result-item:hover {
    background: var(--bg-hover);
  }

  .line-number {
    min-width: 50px;
    font-size: 11px;
    color: var(--text-tertiary);
    font-family: 'Fira Code', 'Consolas', monospace;
  }

  .result-text {
    flex: 1;
    font-size: 12px;
    font-family: 'Fira Code', 'Consolas', monospace;
    white-space: pre;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
