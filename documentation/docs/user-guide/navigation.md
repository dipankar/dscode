# Navigation

DSCode's UI is designed for fast, keyboard-driven navigation. The layout consists of an Activity Bar, Sidebar, Editor Area, Panel Area, and Status Bar -- all implemented as Svelte components that communicate through reactive stores and Tauri IPC commands.

---

## Activity Bar

The Activity Bar is the vertical icon strip on the far left of the window, implemented in `ActivityBar.svelte`. It provides quick access to the primary views:

| Icon | View | Description |
|---|---|---|
| Files | **Explorer** | Browse and manage your project files |
| Search | **Search** | Search across files in the workspace |
| Git Branch | **Source Control** | View and manage Git changes |
| Bug | **Debug** | Configure and run debuggers |
| Blocks | **Extensions** | Browse, install, and manage extensions |

Click an icon to open its corresponding panel in the Sidebar. Click the same icon again to toggle the Sidebar closed.

!!! tip
    Extensions can register additional Activity Bar items through the `activity_bar_registry` in the Rust backend. If you install an extension that adds a new view (e.g., a test runner or database browser), its icon will appear in the Activity Bar automatically.

---

## Sidebar

The Sidebar (`Sidebar.svelte`) displays the content panel for whichever Activity Bar item is currently selected. Each view is a dedicated Svelte component:

- **Explorer** -- file tree rendered by `TreeNode.svelte` with lazy-loading via the `filesystem_ops` Rust commands
- **Search** -- `SearchView.svelte` powered by the `search_ops` backend using `grep-regex` and `ignore` crates
- **Source Control** -- `GitView.svelte` with staging, diffs, and branch management via `git_ops`
- **Debug** -- `DebugView.svelte` with breakpoints, call stack, and variables panels
- **Extensions** -- `ExtensionView.svelte` and `ExtensionGallery.svelte` for the marketplace

### Resizing the Sidebar

Drag the right edge of the Sidebar to resize it. The resize handle is provided by `ResizeHandle.svelte`. You can also toggle the Sidebar visibility:

- **Toggle Sidebar**: ++ctrl+b++

---

## Command Palette

The Command Palette is the primary way to access every command in DSCode. Open it with:

- ++ctrl+shift+p++ (or ++cmd+shift+p++ on macOS)

The Command Palette (`CommandPalette.svelte`) queries the Rust `command_registry` for all registered commands, including those contributed by extensions. Begin typing to filter, then press ++enter++ to execute.

```
> Format Document
> Toggle Word Wrap
> Git: Commit
> Debug: Start Debugging
```

!!! note
    The `>` prefix is automatically inserted when you open the Command Palette. If you remove it, the input switches to Quick Open mode (file search). You can also open Quick Open directly with ++ctrl+p++.

---

## Quick Open

Quick Open lets you navigate to any file in your workspace by typing part of its name. Open it with:

- ++ctrl+p++ (or ++cmd+p++ on macOS)

The `QuickOpen.svelte` component performs fuzzy file matching against the workspace file index. Results update as you type.

### Quick Open Prefixes

You can use special prefixes in the Quick Open input to switch modes:

| Prefix | Action | Shortcut |
|---|---|---|
| *(none)* | Go to file | ++ctrl+p++ |
| `>` | Run command | ++ctrl+shift+p++ |
| `@` | Go to symbol in file | ++ctrl+shift+o++ |
| `@:` | Go to symbol in file (grouped by kind) | ++ctrl+shift+o++ then type `:` |
| `#` | Go to symbol in workspace | ++ctrl+t++ |
| `:` | Go to line | ++ctrl+g++ |

!!! tip
    You can chain file navigation with symbol search. Type a file name in Quick Open, then add `@` to search for a symbol within that file -- for example, `app.ts@handleClick`.

---

## Breadcrumbs

Breadcrumbs appear at the top of the editor and show the file path and symbol hierarchy for the current cursor position. They provide a fast way to navigate up through folders and between sibling symbols.

- Click any segment to see a dropdown of siblings at that level.
- Click a folder segment to see other files in that directory.
- Click a symbol segment to see other symbols in the current file.

Enable or disable breadcrumbs in settings:

```json
{
  "breadcrumbs.enabled": true,
  "breadcrumbs.symbolPath": "on",
  "breadcrumbs.filePath": "on"
}
```

---

## Go to Symbol

Navigate to a symbol (function, class, variable, etc.) within the current file.

- **Go to Symbol in File**: ++ctrl+shift+o++ (or ++cmd+shift+o++ on macOS)

The `SymbolSearch.svelte` component displays a filtered list of symbols provided by the language server. Type to filter and press ++enter++ to jump to the selected symbol.

To group symbols by kind (functions, classes, variables), type `:` after the `@` prefix:

```
@:function
@:class
```

---

## Go to Line

Jump directly to a specific line number in the current file:

- ++ctrl+g++ (or ++cmd+g++ on macOS)

Type the line number and press ++enter++. You can also append a column number separated by a colon:

```
42        → line 42, column 1
42:15     → line 42, column 15
```

---

## Tab Management

DSCode uses a tabbed editor interface. Each open file is represented by a tab at the top of the Editor Area.

### Tab Operations

| Action | Shortcut |
|---|---|
| Open new tab (from Quick Open) | ++ctrl+p++ |
| Close current tab | ++ctrl+w++ |
| Close all tabs | ++ctrl+k++ ++ctrl+w++ |
| Reopen closed tab | ++ctrl+shift+t++ |
| Switch to next tab | ++ctrl+tab++ |
| Switch to previous tab | ++ctrl+shift+tab++ |
| Switch to tab N (1-9) | ++alt+1++ through ++alt+9++ |

### Preview Mode

Single-clicking a file in the Explorer opens it in *preview mode* (indicated by an italic tab title). Preview tabs are reused when you open another file. To pin a file and keep it open, double-click it in the Explorer or start editing it.

!!! tip
    You can disable preview mode in settings with `"workbench.editor.enablePreview": false` if you prefer every file click to open a persistent tab.

---

## Editor Groups and Split View

DSCode supports splitting the editor into multiple groups so you can view files side by side.

### Splitting the Editor

| Action | Shortcut |
|---|---|
| Split editor right | ++ctrl+backslash++ |
| Split editor down | ++ctrl+k++ ++ctrl+backslash++ |
| Focus left editor group | ++ctrl+1++ |
| Focus right editor group | ++ctrl+2++ |
| Focus Nth editor group | ++ctrl+1++ through ++ctrl+9++ |
| Move editor to next group | ++ctrl+alt+right++ |
| Move editor to previous group | ++ctrl+alt+left++ |

### Arranging Editor Groups

You can also drag and drop tabs between editor groups, or drag a tab to the edge of the Editor Area to create a new split.

!!! note
    Editor group layouts are preserved across sessions through the `window-persistence.ts` module, which saves and restores the layout state using the Rust `session_ops` backend.

---

## Keyboard Navigation Summary

Below is a consolidated reference of the navigation shortcuts covered in this guide:

| Action | Shortcut |
|---|---|
| Command Palette | ++ctrl+shift+p++ |
| Quick Open | ++ctrl+p++ |
| Go to Symbol in File | ++ctrl+shift+o++ |
| Go to Symbol in Workspace | ++ctrl+t++ |
| Go to Line | ++ctrl+g++ |
| Toggle Sidebar | ++ctrl+b++ |
| Toggle Panel | ++ctrl+j++ |
| Focus Editor | ++ctrl+1++ |
| Focus Terminal | ++ctrl+grave++ |
| Close Editor | ++ctrl+w++ |
| Split Editor | ++ctrl+backslash++ |
| Next Tab | ++ctrl+tab++ |
| Previous Tab | ++ctrl+shift+tab++ |

!!! tip "macOS Users"
    On macOS, substitute ++cmd++ for ++ctrl++ in most shortcuts. For example, ++cmd+shift+p++ opens the Command Palette and ++cmd+p++ opens Quick Open.
