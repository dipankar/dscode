# UI System Design

## Overview

DSCode's UI is built with **Svelte 4** running inside Tauri's WebView, providing:

- **Familiar web technologies**: HTML, CSS, TypeScript
- **Reactive state management**: Svelte stores with compile-time optimisations
- **VS Code theme compatibility**: CSS custom properties mapped to VS Code color tokens
- **VS Code UI parity**: Same layout, panels, and interactions as VS Code
- **Accessibility**: ARIA roles, keyboard navigation, focus management
- **Lazy loading**: Modals and overlays loaded on demand via dynamic imports

## Component Architecture

### Main Layout

```
┌──────────────────────────────────────────────────────────────┐
│ Title Bar                                                    │
├──┬───────────────────────────────────────────────────────────┤
│A │ Tabs: file1.rs | file2.rs [+]                            │
│c ├───────────────────────────────────────────────────────────┤
│t │ Breadcrumbs: src > file1.rs                              │
│i ├───────────────────────────────────────────────────────────┤
│v │ Editor Group (Monaco)                                    │
│i │                                                          │
│t ├──────────────── split ──────────────────────────────────  │
│y │ Editor Group 2                                           │
│B ├───────────────────────────────────────────────────────────┤
│a │ Panel: Terminal | Problems | Output | Debug Console      │
│r └───────────────────────────────────────────────────────────┘
│ Status Bar: Ln 42, Col 16 | UTF-8 | Rust | main             │
└──────────────────────────────────────────────────────────────┘
```

### Component Tree

```
App.svelte (root + error boundary)
├── ActivityBar.svelte      (icon sidebar)
├── Sidebar.svelte          (Explorer, Search, Git, Extensions)
│   ├── TreeItemView.svelte
│   ├── SearchView.svelte
│   ├── GitView.svelte
│   │   └── BranchSwitcher.svelte
│   └── ExtensionView.svelte
├── EditorArea.svelte       (tab bar + Monaco editors)
│   └── DiffViewer.svelte
├── PanelArea.svelte        (terminal, problems, output, debug)
│   ├── DebugConsole.svelte
│   └── ProblemsPanel.svelte
├── StatusBar.svelte
└── Lazy-loaded overlays:
    ├── CommandPalette.svelte
    ├── QuickOpen.svelte
    ├── SymbolSearch.svelte
    ├── QuickPickModal.svelte
    ├── WindowPromptModal.svelte
    ├── SettingsModal.svelte
    ├── ExtensionGallery.svelte
    └── ResourceMetrics.svelte
```

## Key Components

### ActivityBar

```svelte
<script lang="ts">
  import { activityBarItems, activePanel } from '../stores/sidebar';
  import { onDestroy } from 'svelte';

  const unsubscribe = activityBarItems.subscribe(() => {});
  onDestroy(() => unsubscribe());
</script>

<nav class="activity-bar" role="navigation" aria-label="Activity Bar">
  {#each $activityBarItems as item}
    <button
      class:active={item.id === $activePanel}
      on:click={() => activePanel.set(item.id)}
      aria-label={item.label}
      title={item.label}
    >
      <svelte:component this={item.icon} size={24} />
    </button>
  {/each}
</nav>
```

### EditorArea (Monaco Integration)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { invoke } from '@tauri-apps/api/core';
  import { editorState } from '../stores/editor';

  let container: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;

  onMount(() => {
    editor = monaco.editor.create(container, {
      value: $editorState.content,
      language: $editorState.language,
      theme: 'vs-dark',
      automaticLayout: true,
    });

    editor.onDidChangeModelContent(async (e) => {
      await invoke('text_document_changed', {
        uri: $editorState.uri,
        changes: e.changes,
      });
    });
  });

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div bind:this={container} class="editor-container" />
```

### CommandPalette (Lazy-loaded Overlay)

```svelte
<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';

  export let visible = false;
  let query = '';
  let selectedIndex = 0;
  let inputEl: HTMLInputElement;

  $: filtered = commands.filter((cmd) => cmd.label.toLowerCase().includes(query.toLowerCase()));

  onMount(() => {
    inputEl?.focus();
  });

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
    }
    if (e.key === 'ArrowUp') {
      selectedIndex = Math.max(selectedIndex - 1, 0);
    }
    if (e.key === 'Enter' && filtered[selectedIndex]) {
      dispatch('select', filtered[selectedIndex]);
    }
    if (e.key === 'Escape') {
      visible = false;
    }
  }

  const dispatch = createEventDispatcher();
</script>

{#if visible}
  <div
    class="overlay"
    on:click={() => (visible = false)}
    role="dialog"
    aria-label="Command Palette"
  >
    <div class="palette" on:click|stopPropagation>
      <input
        bind:this={inputEl}
        bind:value={query}
        on:keydown={handleKey}
        role="combobox"
        aria-expanded="true"
        aria-activedescendant="item-{selectedIndex}"
      />
      <ul role="listbox">
        {#each filtered as cmd, i}
          <li
            class:active={i === selectedIndex}
            on:click={() => dispatch('select', cmd)}
            role="option"
            id="item-{i}"
            aria-selected={i === selectedIndex}
          >
            {cmd.label} <kbd>{cmd.keybinding || ''}</kbd>
          </li>
        {/each}
      </ul>
    </div>
  </div>
{/if}
```

### WindowPromptModal (Custom Confirm/Alert)

```svelte
<script lang="ts">
  import { windowPromptStore } from '../stores/windowPrompt';
  import { onMount, onDestroy } from 'svelte';

  let previousFocus: HTMLElement | null = null;

  $: prompt = $windowPromptStore;

  onMount(() => {
    previousFocus = document.activeElement as HTMLElement;
    // Focus the modal for accessibility
  });

  onDestroy(() => {
    previousFocus?.focus();
  });
</script>

{#if prompt}
  <div class="overlay" role="dialog" aria-modal="true" aria-label={prompt.level}>
    <p>{prompt.message}</p>
    <div class="actions">
      {#each prompt.actions as action}
        <button on:click={() => prompt.resolve?.(action)}>{action}</button>
      {/each}
      {#if prompt.showCancelButton}
        <button on:click={() => prompt.resolve?.()}>{prompt.cancelLabel || 'Cancel'}</button>
      {/if}
    </div>
  </div>
{/if}
```

## Theme System

### CSS Custom Properties

All theming uses CSS custom properties mapped to VS Code color tokens, defined in `src/app.css`:

```css
:root {
  --bg-primary: #1e1e1e;
  --bg-secondary: #252526;
  --bg-tertiary: #2d2d2d;
  --text-primary: #cccccc;
  --text-secondary: #858585;
  --border-color: #3e3e42;
  --accent-color: #007acc;
  --error-color: #f44747;
  --warning-color: #cca700;
  --success-color: #4ec9b0;
}
```

### Theme Switching

Themes are applied by updating the CSS custom properties on `:root`. This is compatible with VS Code themes—colors from a VS Code `theme.json` are mapped to the custom property names.

### Svelte Scoped Styles

Each component uses Svelte's scoped `<style>` blocks, which automatically scope CSS to the component instance:

```svelte
<div class="sidebar">
  <slot />
</div>

<style>
  .sidebar {
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    width: 250px;
    overflow-y: auto;
  }
</style>
```

## State Management

### Svelte Stores

State is managed via Svelte stores in `src/stores/`:

| Store             | Purpose                                 |
| ----------------- | --------------------------------------- |
| `editor.ts`       | Open files, active tab, cursor position |
| `git.ts`          | Repository status, changes, branches    |
| `sidebar.ts`      | Active panel, visibility                |
| `windowPrompt.ts` | Modal confirm/alert state               |
| `appError.ts`     | Global error boundary state             |
| `terminal.ts`     | Terminal instances                      |
| `debug.ts`        | Debug sessions, breakpoints             |

### Store Lifecycle

Components that subscribe to stores must unsubscribe on destroy to prevent memory leaks:

```svelte
<script>
  import { editorState } from '../stores/editor';
  import { onDestroy } from 'svelte';

  const unsubscribe = editorState.subscribe((state) => {
    // handle state
  });

  onDestroy(() => unsubscribe());
</script>
```

## Lazy Loading

Heavy overlay components are loaded on demand via the app shell controller:

```typescript
// src/lib/app-shell/controller.ts
async function lazyLoad() {
  const [cp, qo, ss, rm, eg, sm] = await Promise.all([
    import('../../components/CommandPalette.svelte'),
    import('../../components/QuickOpen.svelte'),
    import('../../components/SymbolSearch.svelte'),
    import('../../components/ResourceMetrics.svelte'),
    import('../../components/ExtensionGallery.svelte'),
    import('../../components/SettingsModal.svelte'),
  ]);
  // Assign to shellState for <svelte:component this={...}>
}
```

This ensures chart.js (in ResourceMetrics), language mode workers, and other heavy dependencies are only loaded when the user opens those panels.

## Error Boundaries

A global error boundary prevents unhandled errors from crashing the UI:

```svelte
<!-- App.svelte -->
<script>
  import ErrorOverlay from './components/ErrorOverlay.svelte';
  import { appErrorStore } from './stores/appError';

  onMount(() => {
    window.onerror = (message, source, lineno, colno, error) => {
      appErrorStore.set({ message: String(message), error });
    };
    window.addEventListener('unhandledrejection', (e) => {
      appErrorStore.set({ message: String(e.reason) });
    });
  });
</script>

{#if $appErrorStore}
  <ErrorOverlay message={$appErrorStore.message} on:dismiss={() => appErrorStore.set(null)} />
{/if}
```

## Extension UI Contributions

Extensions can contribute UI through:

1. **Tree Views**: Custom sidebar panels via `vscode.window.registerTreeDataProvider`
2. **Webviews**: Full HTML/CSS/JS panels via Tauri's webview API
3. **Status Bar Items**: Custom status information via `vscode.window.createStatusBarItem`
4. **Commands**: Registered commands appear in the command palette
5. **Menus**: Context menus, editor menus contributed via `vscode.languages.registerCodeLensProvider`

See [extension-system.md](extension-system.md) for details.
