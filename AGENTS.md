# AGENTS.md

Project-specific instructions for AI coding assistants.

## Build & Check Commands

```bash
# Frontend TypeScript check
npx tsc --noEmit

# Svelte component type check
npm run check

# Rust backend type check
cd src-tauri && cargo check

# Extension host TypeScript check
cd extension-host && npx tsc --noEmit

# Vite production build
npx vite build

# Full Tauri development mode
npm run tauri:dev

# Full Tauri production build
npm run tauri:build

# Lint
npm run lint
```

Before committing changes, always run:

1. `npx tsc --noEmit` (frontend TS)
2. `cd src-tauri && cargo check` (Rust)
3. `cd extension-host && npx tsc --noEmit` (extension host TS)
4. `npx vite build` (production build)

## Project Architecture

- **Frontend**: Svelte 4 + TypeScript + Monaco Editor + xterm.js
- **Backend**: Tauri 2.1 (Rust) with tokio async runtime
- **Extension Host**: Separate Node.js process communicating via NNG IPC (REQ/REP pattern)
- **IPC**: Tauri IPC for frontend<->Rust, NNG + serde_json for Rust<->extension-host/LSP
- **Styling**: CSS custom properties (VS Code theme compat), NO Tailwind
- **State**: Svelte stores in `src/stores/`
- **Components**: Svelte components in `src/components/`, lazy-loaded overlays in `src/lib/app-shell/controller.ts`

## Key Conventions

- Use `tokio::sync::Mutex` (NOT `std::sync::Mutex`) in any async context
- Use `tokio::task::spawn_blocking` for filesystem I/O in Tauri command handlers
- Extension host NNG recv runs in a worker thread (never blocks the Node.js event loop)
- All filesystem IPC operations go through `PathValidator`
- Use `showConfirmPrompt()`/`showAlertPrompt()` from `stores/windowPrompt`, never `confirm()`/`alert()`
- Components must unsubscribe from stores in `onDestroy()` to prevent memory leaks
- No comments in code unless explicitly requested

## File Structure

```
src-tauri/src/
  main.rs              # Tauri app entry, command registration
  bootstrap.rs         # App initialization
  session/
    mod.rs              # SessionManager (central state coordinator)
    ipc.rs              # Extension host IPC handler + path validation
    extensions.rs       # Extension lifecycle, VSIX extraction
    workspace.rs        # Workspace folder management
  extension_host/
    manager.rs          # Process lifecycle + crash recovery
    nng_ipc.rs          # NNG IPC with spawn_blocking
    nng_manager.rs      # NNG connection manager
    sandbox.rs          # Deny-by-default sandboxing
    path_validator.rs   # Path validation + traversal prevention
    secrets.rs          # SecretStorage with OS keyring
  lsp/
    client.rs           # LSP client (tokio::sync::Mutex)
    manager.rs          # LSP manager
    pool.rs             # LSP connection pool
  commands/             # Tauri command handlers (sync, std::sync::Mutex OK)

extension-host/src/
  main.ts               # Extension host entry point
  nng-ipc.ts            # NNG IPC with worker thread
  bridge.ts             # Request/response handlers (18+ providers)
  api/languages.ts       # Language provider registry
  extensions/manager.ts  # Extension lifecycle

src/
  App.svelte            # Root component + error boundary
  app.css               # Global styles + CSS custom properties
  main.ts               # Entry point, lazy component loading
  components/           # 34 Svelte components
  stores/               # 11 Svelte stores
  lib/                  # Shared logic (app-shell, command-dispatcher, contracts)
```
