# Modularity Assessment

## Current strengths

- The codebase already separates the main runtime domains into `src-tauri/`, `src/`, and `extension-host/`, which is the right top-level split for a Tauri editor.
- Many backend features follow a `*_registry.rs` plus `*_ops.rs` pattern, which gives each Tauri command a natural home even if the overall wiring has become centralized.
- The extension host is kept in a separate process, which is the most important extensibility boundary in the project today.

## Primary architectural issues

### 1. Bootstrap and dependency wiring are too centralized

`src-tauri/src/main.rs` had accumulated tray setup, app directory initialization, session startup, and feature registry registration in one function, plus a very large `invoke_handler` list. That makes new features easy to bolt on, but hard to reason about as a system.

Impact:
- Changes to unrelated startup concerns collide in one file.
- The app has no explicit backend composition root beyond `main.rs`.
- Extending startup behavior increases merge pressure and hidden coupling.

### 2. Session management is a god object

`src-tauri/src/session/mod.rs` is responsible for extension lifecycle, UI event emission, workspace watching, message prompts, output channels, configuration coordination, and editor notifications.

Impact:
- It is difficult to test or evolve one responsibility without loading many others.
- Extension-facing behavior and application-session behavior are tightly coupled.
- Future features are likely to keep landing in the same file because it is already the central coordinator.

Recommended split:
- `session/extensions.rs`
- `session/workspace_watch.rs`
- `session/ui_events.rs`
- `session/window_prompts.rs`
- `session/output_channels.rs`

### 3. Frontend shell logic is concentrated in `App.svelte`

`src/App.svelte` owns layout state, persistence, lazy overlay loading, extension-host startup, and a custom keyboard shortcut layer. The component is acting as both root view and shell controller.

Impact:
- UI shell behavior is difficult to reuse or test.
- Shortcut behavior is fragmented because there is also a dedicated keybinding manager in `src/lib/keybinding-manager.ts`.
- Adding global UI features increases pressure on the root component instead of feature modules.

Recommended split:
- `src/lib/app-shell/state.ts`
- `src/lib/app-shell/shortcuts.ts`
- `src/lib/app-shell/layout.ts`
- thin `App.svelte` as view composition only

### 4. Cross-module communication is stringly typed

The frontend relies heavily on raw `window.dispatchEvent(...)`, raw Tauri event names, and raw invoke command strings across `src/`, `src/lib/`, and `src/stores/`.

Impact:
- Renames are brittle.
- Event contracts are hard to discover.
- There is limited compile-time protection for payload shape drift.

Recommended direction:
- Introduce typed event constants and thin wrapper APIs for Tauri `invoke`/`listen`.
- Treat command names and event names as shared contracts rather than inline strings.

### 5. The registry pattern is useful, but duplicated

The many `*_registry.rs` and `*_ops.rs` pairs are conceptually consistent, but they repeat the same lifecycle patterns: state storage, event emission, CRUD commands, and clearing by owner.

Impact:
- New registries are expensive to add.
- Behavior consistency depends on copy/paste discipline.
- Cross-cutting concerns like ownership cleanup or event telemetry are spread everywhere.

Recommended direction:
- Standardize a small set of reusable registry traits/helpers for owner-scoped cleanup, event emission, and query behavior.
- Group registries by domain rather than only by command surface.

## Immediate refactor completed in this pass

- Extracted backend bootstrap orchestration from `src-tauri/src/main.rs` into `src-tauri/src/bootstrap.rs`.
- Split `src-tauri/src/session/mod.rs` so document operations, extension lifecycle logic, and extension-host IPC handling now live in:
  - `src-tauri/src/session/documents.rs`
  - `src-tauri/src/session/extensions.rs`
  - `src-tauri/src/session/ipc.rs`

This does not finish the session decomposition, but it creates clearer backend seams for:
- core service registration
- tray setup
- session manager initialization
- feature registry registration
- document editing helpers
- extension lifecycle behavior
- extension host request handling

## Suggested next sequence

1. Continue breaking `session/mod.rs` into focused submodules without changing public command behavior.
2. Introduce typed frontend command/event wrappers so UI modules stop depending on raw strings.
3. Move shell state and shortcut orchestration out of `App.svelte`.
4. Consolidate registry common behavior into shared abstractions once the session and shell boundaries are cleaner.
