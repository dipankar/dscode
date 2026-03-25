# Code Style

Consistent code style is enforced across the entire DSCode codebase through
automated formatting and linting tools. This guide covers the rules,
conventions, and patterns used in both the Rust backend and the
TypeScript/Svelte frontend.

---

## Rust Code Style

### Formatting with rustfmt

All Rust code is formatted with `rustfmt` using the project's configuration.
Run the formatter before committing:

```bash
cd src-tauri

# Format all Rust files
cargo fmt

# Check formatting without modifying files
cargo fmt --check
```

!!! info "rustfmt configuration"
    If a `rustfmt.toml` or `.rustfmt.toml` file exists in `src-tauri/`, it
    takes precedence over the default settings. The project uses Rust edition
    2021 defaults with the stable toolchain.

### Linting with Clippy

Clippy catches common mistakes, performance issues, and non-idiomatic Rust. Run
it locally before pushing:

```bash
cd src-tauri

# Run clippy with all features enabled
cargo clippy --all-features

# Treat all warnings as errors (same as CI)
cargo clippy -- -D warnings

# Run clippy on tests too
cargo clippy --all-targets --all-features -- -D warnings
```

Common Clippy categories enforced in this project:

| Category | Example Lint | Action |
|----------|-------------|--------|
| `clippy::unwrap_used` | Prefer `?` or `.expect("reason")` over `.unwrap()` | Warn |
| `clippy::needless_return` | Omit explicit `return` at end of function | Deny |
| `clippy::clone_on_ref_ptr` | Avoid cloning `Arc`/`Rc` unnecessarily | Warn |
| `clippy::large_enum_variant` | Box large enum variants | Warn |

### Naming Conventions

Rust uses **snake_case** universally, following the official Rust naming
guidelines:

```rust
// Functions and methods: snake_case
fn read_file_contents(path: &str) -> Result<String, Error> { ... }

// Structs and enums: PascalCase
struct TestRunnerRegistry { ... }
enum TestStatus { Running, Passed, Failed }

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT: u32 = 3;
const DEFAULT_TIMEOUT_MS: u64 = 30_000;

// Modules and crate names: snake_case
mod test_runner_registry;
mod extension_ops;

// Type parameters: single uppercase letter or PascalCase
fn process<T: Serialize>(item: T) -> Result<T, Error> { ... }
```

### Error Handling

DSCode uses `thiserror` for defining error types and `anyhow` for propagating
errors in application code:

```rust
use thiserror::Error;

/// Domain-specific errors for file operations
#[derive(Debug, Error)]
pub enum FileError {
    #[error("File not found: {path}")]
    NotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

!!! tip "When to use which"
    - **`thiserror`** -- for library-style code with specific error variants that
      callers may want to match on.
    - **`anyhow`** -- for application-level code where you just need to propagate
      errors with context.
    - **`String`** -- for Tauri command return types (`Result<T, String>`) since
      Tauri serializes errors as strings over IPC.

#### Error patterns in Tauri commands

```rust
// Registry methods return Result<T, String> for Tauri compatibility
#[tauri::command]
pub async fn get_test_suite(
    suite_id: String,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<TestSuite, String> {
    registry.get_test_suite(&suite_id).await
}
```

### Module Organization: Registry + Ops Pattern

Every feature domain in `src-tauri/src/commands/` follows the **registry + ops**
pattern:

```
commands/
+-- feature_registry.rs    # State, data structures, business logic
+-- feature_ops.rs         # #[tauri::command] thin wrappers
```

**Registry file** (`*_registry.rs`):

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Manages feature state and provides business logic
pub struct FeatureRegistry {
    items: Arc<RwLock<HashMap<String, FeatureItem>>>,
}

impl FeatureRegistry {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get an item by ID
    pub async fn get_item(&self, id: &str) -> Result<FeatureItem, String> {
        self.items
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Item '{}' not found", id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureItem {
    pub id: String,
    pub name: String,
}
```

**Ops file** (`*_ops.rs`):

```rust
use super::feature_registry::*;
use tauri::State;

#[tauri::command]
pub async fn get_item(
    id: String,
    registry: State<'_, FeatureRegistry>,
) -> Result<FeatureItem, String> {
    registry.get_item(&id).await
}
```

!!! note "Why this pattern?"
    Separating state management from IPC command handlers makes registries
    independently testable without needing a Tauri runtime. It also keeps the
    ops files short and consistent.

### Serde conventions

All types that cross the IPC boundary use `#[serde(rename_all = "camelCase")]`
to match JavaScript naming conventions on the frontend:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRunResult {
    pub run_id: String,       // -> runId in JSON
    pub suite_id: String,     // -> suiteId in JSON
    pub started_at: String,   // -> startedAt in JSON
}
```

---

## TypeScript / Svelte Code Style

### ESLint

ESLint enforces code quality rules for all TypeScript, JavaScript, and Svelte
files:

```bash
# Lint the entire project
npm run lint

# Lint with auto-fix
npx eslint . --ext .js,.ts,.svelte --fix
```

Key ESLint rules enforced:

| Rule | Setting | Effect |
|------|---------|--------|
| `no-unused-vars` | error | No unused variables or imports |
| `no-console` | warn | Prefer structured logging over `console.log` |
| `@typescript-eslint/explicit-function-return-type` | warn | Encourage explicit return types |
| `@typescript-eslint/no-explicit-any` | warn | Discourage `any` type usage |
| `svelte/no-unused-svelte-ignore` | error | No stale `svelte-ignore` comments |

### Prettier

Prettier handles all formatting for TypeScript, JavaScript, Svelte, CSS, JSON,
and Markdown files:

```bash
# Format all files
npm run format

# Check formatting without modifying files
npx prettier --check .
```

Default Prettier configuration (from `package.json` or `.prettierrc`):

```json
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "all",
  "printWidth": 100,
  "plugins": ["prettier-plugin-svelte"],
  "overrides": [
    {
      "files": "*.svelte",
      "options": {
        "parser": "svelte"
      }
    }
  ]
}
```

### Naming Conventions

TypeScript/JavaScript uses **camelCase** for most identifiers, following
standard JavaScript conventions:

```typescript
// Variables and functions: camelCase
const editorState = writable<EditorState>(defaultState);
function handleFileOpen(path: string): void { ... }

// Interfaces and types: PascalCase
interface FileTreeNode {
  name: string;
  path: string;
  children?: FileTreeNode[];
}

// Constants: SCREAMING_SNAKE_CASE or camelCase
const MAX_OPEN_TABS = 50;
const defaultTheme = 'dark';

// Svelte component files: PascalCase
// EditorArea.svelte, CommandPalette.svelte, StatusBar.svelte

// Store files: camelCase
// editor.ts, workspace.ts, debug.ts

// Utility files: kebab-case
// command-dispatcher.ts, keybinding-manager.ts
```

### Svelte Component Structure

Svelte components should follow this ordering:

```svelte
<!-- 1. Script block with TypeScript -->
<script lang="ts">
  // Imports (external libraries first, then internal modules)
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { editorStore } from '../stores/editor';

  // Props (exported variables)
  export let filePath: string;
  export let readOnly = false;

  // Local state
  let isLoading = true;
  let content = '';

  // Reactive declarations
  $: fileName = filePath.split('/').pop() ?? 'untitled';
  $: isModified = content !== savedContent;

  // Lifecycle
  onMount(async () => {
    content = await invoke('read_file', { path: filePath });
    isLoading = false;
  });

  // Event handlers
  function handleSave(): void {
    invoke('write_file', { path: filePath, content });
  }
</script>

<!-- 2. Markup -->
<div class="editor-container" class:loading={isLoading}>
  {#if isLoading}
    <span>Loading...</span>
  {:else}
    <textarea bind:value={content} {readOnly} />
  {/if}
</div>

<!-- 3. Styles (scoped by default) -->
<style>
  .editor-container {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .loading {
    opacity: 0.5;
  }
</style>
```

### Svelte Store Patterns

Stores use Svelte's `writable`, `readable`, or `derived` stores with typed
interfaces:

```typescript
// src/stores/editor.ts
import { writable, derived } from 'svelte/store';

interface Tab {
  id: string;
  path: string;
  name: string;
  isDirty: boolean;
}

interface EditorState {
  openTabs: Tab[];
  activeTabId: string | null;
}

const defaultState: EditorState = {
  openTabs: [],
  activeTabId: null,
};

export const editorStore = writable<EditorState>(defaultState);

// Derived store for the active tab
export const activeTab = derived(editorStore, ($store) =>
  $store.openTabs.find((tab) => tab.id === $store.activeTabId) ?? null,
);

// Store update helpers (exported functions, not methods)
export function openTab(tab: Tab): void {
  editorStore.update((state) => ({
    ...state,
    openTabs: [...state.openTabs, tab],
    activeTabId: tab.id,
  }));
}

export function closeTab(tabId: string): void {
  editorStore.update((state) => ({
    ...state,
    openTabs: state.openTabs.filter((t) => t.id !== tabId),
    activeTabId:
      state.activeTabId === tabId
        ? state.openTabs[state.openTabs.length - 2]?.id ?? null
        : state.activeTabId,
  }));
}
```

---

## Commit Message Format

DSCode follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Use When |
|------|----------|
| `feat` | Adding a new feature |
| `fix` | Fixing a bug |
| `docs` | Documentation only changes |
| `style` | Formatting, semicolons, etc. (no logic change) |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | Performance improvement |
| `test` | Adding or updating tests |
| `build` | Build system or dependency changes |
| `ci` | CI/CD pipeline changes |
| `chore` | Maintenance tasks (updating deps, tooling) |

### Scopes

Use the area of the codebase affected:

| Scope | Area |
|-------|------|
| `editor` | Monaco editor, text editing |
| `terminal` | Integrated terminal |
| `git` | Git integration |
| `lsp` | Language Server Protocol |
| `ext` | Extension system |
| `ipc` | Tauri IPC commands |
| `ui` | Frontend UI components |
| `debug` | Debugger / DAP |
| `search` | File/text search |
| `config` | Settings and configuration |

### Examples

```
feat(editor): add multi-cursor support via Monaco API

fix(terminal): resolve PTY resize race condition on Linux

docs(contributing): add code style guide for Rust conventions

refactor(ipc): extract registry pattern from command handlers

test(ext): add integration tests for extension host IPC

perf(search): use parallel file walking for ripgrep search

build(deps): update tauri to 2.1 and serde to 1.0
```

!!! warning "Keep the subject line under 72 characters"
    The first line of the commit message should be concise. Use the body for
    detailed explanations.

---

## Documentation Style

### Rust doc comments

All public functions, structs, enums, and traits must have `///` doc comments:

```rust
/// Register a new test suite for an extension.
///
/// The suite is stored in the registry and can be executed via
/// [`start_test_run`]. Duplicate suite IDs overwrite previous entries.
///
/// # Arguments
///
/// * `suite` - The test suite to register, including all test cases.
///
/// # Returns
///
/// The suite ID on success, or an error message if registration fails.
pub async fn register_test_suite(&self, suite: TestSuite) -> Result<String, String> {
    // ...
}
```

### TypeScript JSDoc

Public functions and exported interfaces should have JSDoc comments:

```typescript
/**
 * Dispatches a command by its ID, optionally passing arguments.
 *
 * @param commandId - The unique identifier for the command
 * @param args - Optional arguments to pass to the command handler
 * @returns A promise that resolves when the command completes
 * @throws If the command ID is not registered
 */
export async function executeCommand(commandId: string, ...args: unknown[]): Promise<void> {
  // ...
}
```

---

## Common Patterns

### Async state with `Arc<RwLock<...>>`

All shared state in the Rust backend uses `Arc<RwLock<HashMap<...>>>` for
concurrent access from Tauri's async command handlers:

```rust
pub struct MyRegistry {
    items: Arc<RwLock<HashMap<String, Item>>>,
}
```

!!! tip "Read vs Write locks"
    Use `.read().await` when only reading data. Use `.write().await` only when
    mutating. This allows multiple concurrent readers.

### Event emission via Tauri `Emitter`

Registries emit events to notify the frontend of state changes:

```rust
use tauri::{AppHandle, Emitter};

if let Err(e) = self.app_handle.emit("test-run-completed", &result) {
    eprintln!("[TestRunner] Failed to emit event: {}", e);
}
```

### `#[serde(rename_all = "camelCase")]` on all IPC types

Every struct or enum that crosses the Rust-to-TypeScript boundary must use
`camelCase` serialization to match JavaScript conventions.

### Path aliases in TypeScript

The project defines path aliases in `tsconfig.json` for cleaner imports:

```typescript
// Instead of: import { editorStore } from '../../stores/editor';
// Use:
import { editorStore } from '$lib/stores';

// Aliases defined:
// $lib/* -> src/lib/*
// $components/* -> src/components/*
```

---

## Next Steps

- [Pull Request Guidelines](pull-request-guidelines.md) -- how to submit your contribution
- [Testing](testing.md) -- run tests before submitting
- [Building from Source](building-from-source.md) -- set up your build environment
