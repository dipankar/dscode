<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { clearQuickPick, type QuickPickEntry, type QuickPickState } from '../stores/quickPick';
  import { focusTrap } from '../lib/focus-trap';

  export let request: QuickPickState;

  let filter = '';
  let selectedIndex = 0;
  let submitting = false;
  let searchInput: HTMLInputElement | null = null;
  let selections = new Set<number>();
  let items: QuickPickEntry[] = [];
  let lastRequestId = '';
  $: items = request.items;
  $: if (request.id !== lastRequestId) {
    lastRequestId = request.id;
    filter = '';
    selectedIndex = 0;
    selections = new Set<number>();
    submitting = false;
    initializeSelection();
    queueMicrotask(() => {
      searchInput?.focus();
    });
  }

  $: filteredItems = items.filter((item) => matchesFilter(item, filter));
  $: ensureSelectionWithinBounds();

  onMount(() => {
    const handle = window.setTimeout(() => {
      searchInput?.focus();
    }, 0);

    return () => window.clearTimeout(handle);
  });

  function ensureSelectionWithinBounds() {
    if (selectedIndex >= filteredItems.length) {
      selectedIndex = filteredItems.length > 0 ? filteredItems.length - 1 : 0;
    }
  }

  function initializeSelection() {
    if (request.canPickMany) {
      const picked = items
        .filter((item) => item.raw && typeof item.raw === 'object' && (item.raw as any).picked)
        .map((item) => item.id);
      selections = new Set<number>(picked);
    } else {
      const pickedIndex = items.findIndex(
        (item) => item.raw && typeof item.raw === 'object' && (item.raw as any).picked
      );
      selectedIndex = pickedIndex >= 0 ? pickedIndex : 0;
    }
  }

  function matchesFilter(item: QuickPickEntry, value: string): boolean {
    if (!value.trim()) {
      return true;
    }

    const target = value.toLowerCase();
    if (item.label.toLowerCase().includes(target)) {
      return true;
    }

    if (request.matchOnDescription && item.description?.toLowerCase().includes(target)) {
      return true;
    }

    if (request.matchOnDetail && item.detail?.toLowerCase().includes(target)) {
      return true;
    }

    return false;
  }

  async function chooseSingle(index: number) {
    selectedIndex = index;
    await submitSelection();
  }

  function toggleSelection(index: number) {
    const entry = filteredItems[index];
    if (!entry) {
      return;
    }

    if (selections.has(entry.id)) {
      selections.delete(entry.id);
    } else {
      selections.add(entry.id);
    }
  }

  async function submitSelection() {
    if (submitting) {
      return;
    }
    submitting = true;

    try {
      if (request.canPickMany) {
        const selected = items.filter((item) => selections.has(item.id)).map((item) => item.raw);
        await invoke('window_quick_pick_select', {
          requestId: request.id,
          selection: selected.length > 0 ? selected : null,
        });
      } else {
        const entry = filteredItems[selectedIndex];
        await invoke('window_quick_pick_select', {
          requestId: request.id,
          selection: entry ? entry.raw : null,
        });
      }
    } catch (error) {
      console.error('[QuickPickModal] Failed to resolve quick pick:', error);
    } finally {
      clearQuickPick();
    }
  }

  async function cancel() {
    if (submitting) {
      return;
    }
    submitting = true;
    try {
      await invoke('window_quick_pick_select', {
        requestId: request.id,
        selection: null,
      });
    } catch (error) {
      console.error('[QuickPickModal] Failed to cancel quick pick:', error);
    } finally {
      clearQuickPick();
    }
  }

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      void cancel();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      if (request.canPickMany) {
        void submitSelection();
      } else {
        void chooseSingle(selectedIndex);
      }
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      if (filteredItems.length === 0) {
        return;
      }
      selectedIndex = selectedIndex <= 0 ? filteredItems.length - 1 : selectedIndex - 1;
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      if (filteredItems.length === 0) {
        return;
      }
      selectedIndex = selectedIndex >= filteredItems.length - 1 ? 0 : selectedIndex + 1;
      return;
    }

    if (event.key === ' ' && request.canPickMany) {
      event.preventDefault();
      if (filteredItems.length === 0) {
        return;
      }
      toggleSelection(selectedIndex);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="quickpick-overlay" role="presentation" tabindex="-1" on:click={handleOverlayClick} use:focusTrap={{ onEscape: () => void cancel() }}>
  <div
    class="quickpick-modal"
    role="dialog"
    aria-modal="true"
    aria-label={request.title ?? 'Quick pick'}
  >
    {#if request.title}
      <header>
        <h2>{request.title}</h2>
      </header>
    {/if}

    <div class="quickpick-input">
      <input
        bind:this={searchInput}
        type="text"
        placeholder={request.placeHolder ?? 'Select an item'}
        bind:value={filter}
      />
    </div>

    <ul class="quickpick-list" role="listbox" aria-label={request.title ?? 'Quick pick options'}>
      {#if filteredItems.length === 0}
        <li class="empty">No matches</li>
      {:else}
        {#each filteredItems as item, index}
          <li>
            <button
              type="button"
              class:selected={!request.canPickMany && index === selectedIndex}
              class:checked={request.canPickMany && selections.has(item.id)}
              on:mouseenter={() => (selectedIndex = index)}
              on:click={() =>
                request.canPickMany ? toggleSelection(index) : void chooseSingle(index)}
              role="option"
              aria-selected={!request.canPickMany && index === selectedIndex}
              aria-checked={request.canPickMany ? selections.has(item.id) : undefined}
            >
              <div class="primary-line">
                <span class="label">{item.label}</span>
                {#if item.detail}
                  <span class="detail">{item.detail}</span>
                {/if}
              </div>
              {#if item.description}
                <div class="description">{item.description}</div>
              {/if}
            </button>
          </li>
        {/each}
      {/if}
    </ul>

    {#if request.canPickMany}
      <footer>
        <button on:click={() => void submitSelection()} disabled={submitting}>Select</button>
        <button class="secondary" on:click={() => void cancel()} disabled={submitting}
          >Cancel</button
        >
      </footer>
    {:else}
      <footer>
        <button class="secondary" on:click={() => void cancel()} disabled={submitting}
          >Cancel</button
        >
      </footer>
    {/if}
  </div>
</div>

<style>
  .quickpick-overlay {
    position: fixed;
    inset: 0;
    background: var(--color-modal-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
    padding: 24px;
  }

  .quickpick-modal {
    width: min(520px, 90vw);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  header {
    padding: 12px 16px 0 16px;
  }

  header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
  }

  .quickpick-input {
    padding: 16px;
  }

  .quickpick-input input {
    width: 100%;
    padding: 8px 12px;
    border-radius: 4px;
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 14px;
    outline: none;
  }

  .quickpick-input input:focus {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 1px var(--color-accent);
  }

  .quickpick-list {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 280px;
    overflow-y: auto;
  }

  .quickpick-list li {
    margin: 0;
    padding: 0;
  }

  .quickpick-list li > button {
    width: 100%;
    padding: 10px 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    cursor: pointer;
    border-top: 1px solid transparent;
    border-bottom: 1px solid transparent;
    border-left: none;
    border-right: none;
    background: transparent;
    color: var(--color-text);
    text-align: left;
  }

  .quickpick-list li > button:hover {
    background: var(--color-surface-focus);
  }

  .quickpick-list li > button.selected,
  .quickpick-list li > button.checked {
    background: var(--color-selection, rgba(46, 134, 222, 0.18));
    color: var(--color-selection-foreground, var(--color-text));
  }

  .quickpick-list li .primary-line {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  .quickpick-list li .label {
    font-size: 14px;
    font-weight: 500;
  }

  .quickpick-list li .detail {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .quickpick-list li .description {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .quickpick-list li.empty {
    padding: 18px 16px;
    text-align: center;
    color: var(--color-text-secondary);
    cursor: default;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
  }

  button {
    padding: 6px 12px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-size: 13px;
  }

  button[disabled] {
    cursor: not-allowed;
    opacity: 0.6;
  }

  button:not(.secondary) {
    background: var(--color-accent);
    color: var(--color-text-on-accent);
  }

  button.secondary {
    background: transparent;
    color: var(--color-text-secondary);
  }

  button.secondary:hover:not([disabled]) {
    background: var(--color-surface-hover);
  }
</style>
