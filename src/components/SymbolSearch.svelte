<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { editorStore } from '../stores/editor';
  import { Hash, FileText } from 'lucide-svelte';

  export let visible = false;
  export let onClose: () => void;

  interface Symbol {
    name: string;
    kind: monaco.languages.SymbolKind;
    containerName?: string;
    location: {
      uri: string;
      range: monaco.IRange;
    };
  }

  let searchInput: HTMLInputElement;
  let searchQuery = '';
  let symbols: Symbol[] = [];
  let selectedIndex = 0;

  async function searchSymbols(query: string) {
    if (!query.trim() || !$editorStore.monacoInstance) {
      symbols = [];
      return;
    }

    const model = $editorStore.monacoInstance.getModel();
    if (!model) {
      symbols = [];
      return;
    }

    try {
      // TODO: Implement proper document symbol search
      // Monaco's executeDocumentSymbolProvider is only available via editor extensions
      // For now, return empty results - symbol search needs proper language server integration
      symbols = [];
      selectedIndex = 0;
    } catch (error) {
      console.error('Error searching symbols:', error);
      symbols = [];
    }
  }

  function getSymbolKindIcon(kind: monaco.languages.SymbolKind) {
    switch (kind) {
      case monaco.languages.SymbolKind.File:
        return FileText;
      case monaco.languages.SymbolKind.Module:
      case monaco.languages.SymbolKind.Namespace:
      case monaco.languages.SymbolKind.Package:
        return Hash;
      case monaco.languages.SymbolKind.Class:
      case monaco.languages.SymbolKind.Method:
      case monaco.languages.SymbolKind.Property:
      case monaco.languages.SymbolKind.Field:
      case monaco.languages.SymbolKind.Constructor:
      case monaco.languages.SymbolKind.Enum:
      case monaco.languages.SymbolKind.Interface:
      case monaco.languages.SymbolKind.Function:
      case monaco.languages.SymbolKind.Variable:
      case monaco.languages.SymbolKind.Constant:
      case monaco.languages.SymbolKind.String:
      case monaco.languages.SymbolKind.Number:
      case monaco.languages.SymbolKind.Boolean:
      case monaco.languages.SymbolKind.Array:
      default:
        return Hash;
    }
  }

  function getSymbolKindLabel(kind: monaco.languages.SymbolKind): string {
    return monaco.languages.SymbolKind[kind] || 'Symbol';
  }

  async function selectSymbol(symbol: Symbol) {
    if (!$editorStore.monacoInstance) return;

    // Navigate to symbol location
    $editorStore.monacoInstance.setPosition({
      lineNumber: symbol.location.range.startLineNumber,
      column: symbol.location.range.startColumn,
    });
    $editorStore.monacoInstance.revealLineInCenter(symbol.location.range.startLineNumber);
    $editorStore.monacoInstance.focus();

    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;

    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, symbols.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        e.preventDefault();
        if (symbols[selectedIndex]) {
          selectSymbol(symbols[selectedIndex]);
        }
        break;
    }
  }

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  $: if (visible && searchInput) {
    searchInput.focus();
    searchQuery = '';
    symbols = [];
    selectedIndex = 0;
  }

  $: searchSymbols(searchQuery);

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

{#if visible}
  <div
    class="symbol-search-overlay"
    on:click={handleOverlayClick}
    on:keydown={handleKeydown}
    role="presentation"
    tabindex="-1"
  >
    <div class="symbol-search-container" role="dialog" aria-modal="true" aria-label="Symbol search">
      <input
        bind:this={searchInput}
        bind:value={searchQuery}
        type="text"
        class="symbol-search-input"
        placeholder="Search symbols in current file... (Ctrl+T)"
        autocomplete="off"
        spellcheck="false"
        aria-label="Search symbols in current file"
        aria-controls="symbol-search-results"
      />

      <div class="symbol-results" id="symbol-search-results" role="listbox" aria-label="Symbols">
        {#if symbols.length === 0 && searchQuery.trim()}
          <div class="no-results">No symbols found</div>
        {:else if symbols.length === 0}
          <div class="no-results">
            Type to search for symbols (functions, classes, variables...)
          </div>
        {:else}
          {#each symbols as symbol, index}
            <button
              class="symbol-result-item"
              class:selected={index === selectedIndex}
              on:click={() => selectSymbol(symbol)}
              on:mouseenter={() => (selectedIndex = index)}
              role="option"
              aria-selected={index === selectedIndex}
            >
              <div class="symbol-icon">
                <svelte:component this={getSymbolKindIcon(symbol.kind)} size={16} />
              </div>
              <div class="symbol-info">
                <div class="symbol-name">{symbol.name}</div>
                {#if symbol.containerName}
                  <div class="symbol-container">{symbol.containerName}</div>
                {/if}
              </div>
              <div class="symbol-kind">{getSymbolKindLabel(symbol.kind)}</div>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .symbol-search-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--modal-overlay-bg);
    display: flex;
    justify-content: center;
    padding-top: 100px;
    z-index: 1000;
  }

  .symbol-search-container {
    width: 600px;
    max-height: 500px;
    background-color: var(--modal-bg);
    border: 1px solid var(--modal-border);
    border-radius: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .symbol-search-input {
    width: 100%;
    padding: 14px 16px;
    background-color: var(--color-bg);
    border: none;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-text);
    font-size: 14px;
    outline: none;
  }

  .symbol-results {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .symbol-result-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .symbol-result-item:hover,
  .symbol-result-item.selected {
    background-color: var(--color-bg-tertiary);
  }

  .symbol-icon {
    flex-shrink: 0;
    color: var(--color-accent);
  }

  .symbol-info {
    flex: 1;
    min-width: 0;
  }

  .symbol-name {
    font-size: 14px;
    font-weight: 500;
  }

  .symbol-container {
    font-size: 12px;
    color: var(--color-text-secondary);
    margin-top: 2px;
  }

  .symbol-kind {
    flex-shrink: 0;
    font-size: 11px;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .no-results {
    padding: 40px 20px;
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 14px;
  }
</style>
