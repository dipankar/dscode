# Session Manager Architecture

## Problem Statement

The UI does not react properly to backend state changes, particularly for extension deletion. The application lacks a centralized control process to:
- Manage application-wide state
- Coordinate between Extension Host, LSP, and DAP systems
- Emit events to the UI when state changes
- Provide a single source of truth for extension lifecycle

## Solution: Session Manager

A centralized coordinator that acts as the control plane for the entire application session.

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                        Svelte UI                              │
│                                                               │
│  ┌─────────────────┐        ┌──────────────────┐            │
│  │ Session Store   │◄──────►│ Extension Views  │            │
│  │  (Reactive)     │  Event │  (Auto-update)   │            │
│  └────────┬────────┘  Bus   └──────────────────┘            │
└───────────┼───────────────────────────────────────────────────┘
            │
            │ Tauri Event System
            │ (session-event)
            ▼
┌──────────────────────────────────────────────────────────────┐
│                   Session Manager (Rust)                      │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                   Session State                       │   │
│  │  • workspace_folders: Vec<PathBuf>                   │   │
│  │  • active_extensions: Vec<ExtensionInfo>             │   │
│  │  • installed_extensions: Vec<ExtensionInfo>          │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  Coordinates:                                                 │
│  ┌────────────────┐  ┌────────────────┐  ┌───────────────┐  │
│  │  Ext Host Pool │  │   LSP Pool     │  │   DAP Pool    │  │
│  └────────────────┘  └────────────────┘  └───────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Session Manager (`src-tauri/src/session/mod.rs`)

**Responsibilities:**
- **State Management**: Maintains authoritative state for extensions and workspace
- **Lifecycle Coordination**: Manages extension load/unload/delete operations
- **Event Emission**: Broadcasts state changes to UI via Tauri events
- **Resource Coordination**: Orchestrates Extension Host Pool, LSP, and DAP

**Core Methods:**
```rust
// Initialization
initialize() -> Result<(), String>
scan_extensions() -> Result<(), String>

// Extension Lifecycle
load_extension(id: &str) -> Result<(), String>
unload_extension(id: &str) -> Result<(), String>
delete_extension(id: &str) -> Result<(), String>

// State Queries
get_state() -> SessionState
get_installed_extensions() -> Vec<ExtensionInfo>
get_active_extensions() -> Vec<ExtensionInfo>

// Workspace Management
add_workspace_folder(path: PathBuf) -> Result<(), String>
remove_workspace_folder(path: &PathBuf) -> Result<(), String>

// Shutdown
shutdown() -> Result<(), String>
```

### 2. Session Events

Events emitted to the UI for reactive updates:

```rust
pub enum SessionEvent {
    ExtensionInstalled { extension_id: String },
    ExtensionLoaded { extension_id: String },
    ExtensionUnloaded { extension_id: String },
    ExtensionDeleted { extension_id: String },
    ExtensionsChanged { extensions: Vec<ExtensionInfo> },
    WorkspaceFolderAdded { path: PathBuf },
    WorkspaceFolderRemoved { path: PathBuf },
    StateChanged { state: SessionState },
}
```

### 3. Tauri Commands (`src-tauri/src/commands/session_ops.rs`)

```rust
// Exposed to frontend
initialize_session() -> Result<(), String>
get_session_state() -> Result<SessionState, String>
get_installed_extensions() -> Result<Vec<ExtensionInfo>, String>
get_active_extensions() -> Result<Vec<ExtensionInfo>, String>
session_load_extension(id: String) -> Result<(), String>
session_unload_extension(id: String) -> Result<(), String>
session_delete_extension(id: String) -> Result<(), String>
add_workspace_folder(path: String) -> Result<(), String>
remove_workspace_folder(path: String) -> Result<(), String>
```

## Integration Steps

### 1. Backend Integration (main.rs)

Add to imports:
```rust
use session::SessionManager;
use std::sync::Arc;
use tokio::sync::RwLock;
```

In `main()` function setup:
```rust
.setup(|app| {
    // ... existing tray setup ...

    // Initialize Session Manager
    let extensions_dir = std::env::current_dir()
        .unwrap()
        .join("extensions");

    let session_manager = Arc::new(RwLock::new(
        SessionManager::new(app.handle().clone(), extensions_dir)
    ));

    // Store in app state
    app.manage(session_manager.clone());

    // Initialize session asynchronously
    let session = session_manager.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = session.read().await.initialize().await {
            eprintln!("[App] Failed to initialize session: {}", e);
        }
    });

    Ok(())
})
```

Add to invoke_handler:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    initialize_session,
    get_session_state,
    get_installed_extensions,
    get_active_extensions,
    session_load_extension,
    session_unload_extension,
    session_delete_extension,
    add_workspace_folder,
    remove_workspace_folder,
])
```

### 2. Frontend Integration

#### Create Session Store (`src/stores/session.ts`):

```typescript
import { writable, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';

export interface ExtensionInfo {
    id: string;
    name: string;
    version: string;
    publisher: string;
    description?: string;
    enabled: boolean;
    active: boolean;
}

export interface SessionState {
    workspace_folders: string[];
    active_extensions: ExtensionInfo[];
    installed_extensions: ExtensionInfo[];
}

// Session state store
export const sessionState = writable<SessionState>({
    workspace_folders: [],
    active_extensions: [],
    installed_extensions: [],
});

// Initialize session and listen for events
export async function initializeSession() {
    // Load initial state
    const state = await invoke<SessionState>('get_session_state');
    sessionState.set(state);

    // Listen for session events
    await listen('session-event', (event: any) => {
        const { type, data } = event.payload;

        switch (type) {
            case 'StateChanged':
                sessionState.set(data.state);
                break;

            case 'ExtensionsChanged':
                sessionState.update(s => ({
                    ...s,
                    installed_extensions: data.extensions
                }));
                break;

            case 'ExtensionDeleted':
                // Extension deleted - UI will auto-update via StateChanged
                console.log('Extension deleted:', data.extension_id);
                break;

            // ... handle other events
        }
    });
}

// Derived stores for convenience
export const installedExtensions = derived(
    sessionState,
    $state => $state.installed_extensions
);

export const activeExtensions = derived(
    sessionState,
    $state => $state.active_extensions
);

// Extension operations
export async function loadExtension(id: string) {
    await invoke('session_load_extension', { extensionId: id });
}

export async function unloadExtension(id: string) {
    await invoke('session_unload_extension', { extensionId: id });
}

export async function deleteExtension(id: string) {
    await invoke('session_delete_extension', { extensionId: id });
    // UI will auto-update via session events
}
```

#### Update Main App Component:

```svelte
<script lang="ts">
import { onMount } from 'svelte';
import { initializeSession } from './stores/session';

onMount(async () => {
    await initializeSession();
});
</script>
```

#### Update Extension Components:

```svelte
<script lang="ts">
import { installedExtensions, deleteExtension } from './stores/session';

async function handleDelete(extensionId: string) {
    await deleteExtension(extensionId);
    // No manual UI update needed - store will update automatically
}
</script>

{#each $installedExtensions as ext}
    <div class="extension">
        <h3>{ext.name}</h3>
        <button on:click={() => handleDelete(ext.id)}>Delete</button>
    </div>
{/each}
```

## Benefits

### 1. **Reactive UI**
- UI automatically updates when backend state changes
- No manual refresh needed
- Single source of truth

### 2. **Proper State Management**
- Session Manager is authoritative for all state
- Extensions, LSP, DAP all coordinated
- Prevents state desynchronization

### 3. **Event-Driven Architecture**
- Backend emits events via Tauri's event system
- Frontend subscribes and reacts
- Decoupled components

### 4. **Lifecycle Management**
- Proper shutdown sequence
- Resource cleanup
- Extension unload before delete

### 5. **Extensibility**
- Easy to add new state
- Easy to add new events
- Easy to add new lifecycle hooks

## Current Status

**✅ Implemented:**
- Session Manager core (`src-tauri/src/session/mod.rs`)
- Session commands (`src-tauri/src/commands/session_ops.rs`)
- Command module registration

**⏳ Pending:**
- Main.rs integration (SessionManager initialization)
- Invoke handler registration
- Frontend session store
- Frontend component updates

## Next Steps

1. **Complete Backend Integration**
   - Update main.rs to initialize SessionManager
   - Add commands to invoke_handler

2. **Create Frontend Store**
   - Create `src/stores/session.ts`
   - Initialize in App.svelte

3. **Update Extension UI**
   - Replace direct commands with session store
   - Use derived stores for reactive updates

4. **Test Extension Lifecycle**
   - Verify load/unload/delete flow
   - Verify UI updates automatically
   - Verify event emission

## Testing Strategy

1. **Extension Deletion**
   - Delete extension → Verify ExtensionDeleted event → Verify UI updates → Verify filesystem cleanup

2. **Extension Loading**
   - Load extension → Verify ExtensionLoaded event → Verify appears in active list

3. **State Synchronization**
   - Refresh page → Verify state persists → Verify UI matches backend

4. **Multiple Operations**
   - Load, unload, delete in sequence → Verify all events fire → Verify UI stays in sync
