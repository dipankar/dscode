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

## Remaining hotspots to rearchitect

### 1. Workspace state has multiple competing sources of truth

The codebase currently models workspace concerns in at least three places:

- `src-tauri/src/session/mod.rs` stores workspace folders and uses them for extension-host requests and file watching.
- `src-tauri/src/commands/workspace_registry.rs` keeps a separate workspace folder/configuration/decorations registry and emits its own events.
- `src/stores/session.ts`, `src/stores/workspace.ts`, and `src/lib/workspace.ts` each maintain overlapping frontend workspace state.

Impact:
- Folder lists can drift between session-driven state and registry-driven state.
- The frontend has two workspace APIs with different shapes and event sources.
- Search, file decorations, configuration, and extension-host workspace behavior are not clearly owned by one backend service.

Recommended direction:
- Choose a single backend owner for workspace state.
- Make other modules depend on that owner instead of duplicating folder/configuration storage.
- Collapse the frontend to one workspace domain module with one event source and one cached state shape.

Current direction after this pass:
- `SessionManager` is the selected backend owner for workspace folders, workspace search, workspace configuration updates, and file decoration command handling.
- `workspace_ops` now acts as a thin adapter over session-owned state instead of maintaining a second live registry path.

### 2. Extension lifecycle is split across session management and command handlers

`SessionManager` now owns extension activation, extension-host IPC handling, and extension scanning, but `src-tauri/src/commands/extension_ops.rs` still performs direct NNG calls, marketplace install flow, dependency resolution, reload triggers, and backward-compatibility stubs like `start_extension_host`.

Impact:
- Extension behavior is hard to reason about end-to-end because lifecycle logic is split across session, command handlers, and extension-host IPC utilities.
- Some commands treat session as the source of truth while others treat the NNG connection as the source of truth.
- Future extension features will likely keep expanding command handlers instead of strengthening the session boundary.

Recommended direction:
- Move extension install/uninstall/reload orchestration behind a dedicated backend extension service.
- Keep Tauri command handlers thin and delegate to that service.
- Make session depend on the extension service instead of re-implementing extension behavior piecemeal.

Current direction after this pass:
- `SessionManager` now owns extension install orchestration, dependency installation, extension reload/rescan, installed-extension listing, contribution extraction, tree-view IPC, and extension command execution.
- `extension_ops` is reduced to a thin Tauri adapter plus marketplace search/details passthrough.
- The duplicate app-managed `ExtensionHostManager` and app-managed `NngIpcManager` setup were removed from bootstrap.
- `ExtensionGallery.svelte` no longer retries `list_extensions` waiting for a second IPC client to connect, because installed extensions are read from session-owned disk state instead.

### 3. Language features are too large and too repetitive on both sides of the bridge

The backend `language_features_registry.rs` and `language_features_ops.rs`, plus frontend `src/lib/language-features.ts`, are all very large and mirror the same provider concepts repeatedly.

Impact:
- Every new provider type requires touching several large files.
- Registration, lookup, invocation, and Monaco binding logic are duplicated by provider kind.
- The command surface is wide and expensive to maintain.

Recommended direction:
- Split language features by concern:
  - provider registration/state
  - provider invocation transport
  - Monaco adapters
- Introduce reusable abstractions for selector-based provider registration instead of one-off implementations for each provider type.
- Treat language feature transport to the extension host as one protocol layer rather than many ad hoc command handlers.

Current direction after this pass:
- Extension-backed language-feature invocation now routes through the session-owned NNG manager and the live `"main"` extension host instead of a disconnected `"global"` IPC path.
- `language_features_ops.rs` now uses shared extension-host request and parse helpers instead of repeating the same NNG transport boilerplate for each invoked provider.
- `language_features_registry.rs` now uses shared provider traits/helpers for selector-based lookup and owner cleanup, which removes a large amount of copy/pasted registry logic without changing the public API.
- The frontend language-features surface is now split into explicit types, backend transport, and Monaco-conversion modules under `src/lib/language-features/`, instead of keeping those concerns embedded in one file.
- The frontend language-features provider invocation layer now lives in a dedicated provider-client facade plus provider-family clients under `src/lib/language-features/provider-clients/`, so `src/lib/language-features.ts` no longer owns direct extension-host invocation and result conversion for every provider type.
- The Monaco registration layer now has its own facade plus provider-family registrars under `src/lib/language-features/registrars/`, so `src/lib/language-features.ts` is down to backend registration/diagnostics orchestration rather than owning every Monaco binding branch directly.
- Provider registration is now split into provider-family services under `src/lib/language-features/registration-services/`, and diagnostics event/listen/publish handling lives in `src/lib/language-features/diagnostics.ts`.
- The remaining structural decomposition is mostly backend-facing now: the provider registry and command surface are still broad, while the frontend manager is a compatibility facade over smaller domain services.

### 4. Frontend shell control is still concentrated in `App.svelte`

`App.svelte` currently owns:

- overlay visibility
- keyboard shortcut routing
- shell layout persistence
- session bootstrap
- extension-host startup triggering
- global window event wiring

Impact:
- The root component is still acting as application controller instead of a composition layer.
- Global UI behavior depends on raw `window.dispatchEvent(...)` calls and manual listener cleanup.
- Shortcut logic is split between `App.svelte` and `src/lib/keybinding-manager.ts`.

Recommended direction:
- Extract an `app-shell` domain for layout state, global actions, overlays, and startup orchestration.
- Move window-event based communication to a typed action bus or store-driven shell controller.
- Unify builtin shortcut handling and dynamic keybinding handling so there is one shortcut pipeline.

Current direction after this pass:
- `App.svelte` now delegates startup, persisted shell layout, global shortcut handling, overlay lazy-loading, and window-event listeners to `src/lib/app-shell/controller.ts`.
- The root component is reduced to view composition plus resize wiring.
- Builtin shell commands now execute through the same `keybinding-manager.ts` and frontend command-dispatch path used for registry-backed keybindings, instead of a second hardcoded shell keydown pipeline.
- `src/lib/command-context.ts` and `src/lib/when-clause.ts` now provide context-aware `when` evaluation for keybindings and command-palette filtering, with the palette using a snapshot of the pre-overlay context so editor/sidebar commands do not disappear when the palette input takes focus.
- The shared `ContextMenu` path now also uses the same frontend command-context and `when` evaluator, and executes through the shared command dispatcher rather than an extension-only command path.
- The explorer tree no longer ships its own private right-click menu; `TreeNode.svelte` now uses the shared context-menu path, while explorer file actions run through the central command dispatcher and trigger a shared workspace refresh event.
- The remaining shell gap is broader context coverage across more views and eventual removal of the legacy backend-only menu filtering path, not duplicate shortcut infrastructure.

### 5. The frontend/backend contract surface is still stringly typed

The current model relies heavily on:

- raw Tauri command strings in frontend libraries
- raw Tauri event names across backend registries
- raw `window.dispatchEvent(...)` names for intra-frontend coordination
- raw NNG message type strings in extension-host IPC

Impact:
- Renames are brittle and contract drift is easy.
- Payload shapes are not discoverable from one place.
- There is no single source of truth for application-level events or command contracts.

Recommended direction:
- Introduce typed wrappers for:
  - Tauri invoke commands
  - Tauri listen/emit events
  - frontend custom events, or replace them with store/action channels
  - extension-host IPC message types
- Keep shared event/command identifiers in explicit contract modules.

Current direction after this pass:
- `src/lib/contracts/commands.ts` now centralizes the active session, extension, command-registry, menu, system, and editor-save Tauri command names behind typed wrappers.
- The current shell/session path no longer depends on inline command strings for extension execution, session lifecycle, workspace folder mutation, command palette loading, activity-bar loading, menu execution, or editor saves.
- Builtin workbench/file commands are now registered on the backend and dispatched through a shared frontend command dispatcher instead of being duplicated between the shell controller and command palette.
- Raw invoke/event strings still exist in many other domains, so the contract cleanup is started rather than complete.

### 6. Registry modules are useful but overly manual

The backend has a consistent family of registries:

- commands
- menus
- keybindings
- status bar
- activity bar
- workspace
- text documents
- configuration
- themes
- tasks
- marketplace UI
- tests

That consistency is useful, but the implementations repeat the same patterns: owner-scoped storage, event emission, registration, listing, clearing, and sorting.

Impact:
- New registries are expensive to add.
- Behavior consistency depends on copy/paste.
- Cross-cutting concerns like owner cleanup and typed event emission are not centralized.

Recommended direction:
- Add shared registry helpers or traits for:
  - owner-scoped cleanup
  - emit-on-change behavior
  - list/query registration
  - conflict policy
- Group registries into higher-level domains instead of only flat command modules.

Current direction after this pass:
- The language-features registry now has reusable owner-based cleanup and selector-based provider helpers instead of fully bespoke registration and query code for each provider type.
- The broader registry family is still inconsistent; commands, menus, keybindings, status bar, activity bar, and others have not yet been migrated onto shared helpers.

## Suggested workstreams

### Workstream A: Define backend domain ownership

Do this first.

- Decide which module owns workspace state.
- Decide which module owns extension lifecycle.
- Decide which modules are transport adapters only.

This removes the biggest architectural ambiguity in the current codebase.

### Workstream B: Introduce typed contracts

Do this before more feature growth.

- Create typed wrappers around `invoke`, `listen`, and backend emit names.
- Centralize extension-host IPC message identifiers.
- Remove raw `window.dispatchEvent(...)` names from feature modules over time.

This reduces accidental breakage and makes refactors cheaper.

### Workstream C: Break up frontend shell and language features

Do this after ownership is clear.

- Extract app shell state/actions from `App.svelte`.
- Split `language-features.ts` into registration, Monaco adapters, diagnostics, and invocation helpers.
- Unify builtin and extension-contributed keybinding dispatch.

This is the largest user-facing maintainability win on the frontend.

### Workstream D: Standardize registry infrastructure

Do this after the higher-level domain boundaries stabilize.

- Build reusable registry primitives.
- Migrate the most repetitive registries first.
- Keep business rules in domain modules, not in generic registry helpers.

This reduces long-term maintenance cost without forcing a premature abstraction today.
