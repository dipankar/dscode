# dscode-dap

Debug Adapter Protocol client, manager, and pool for DSCode.

This crate provides a complete DAP client implementation with state-machine-based
lifecycle management, a synchronous debug session registry, and an async connection
pool for managing multiple debug adapter instances.

## Features

- **DebugAdapter** -- State-machine-based DAP client that manages a debug adapter
  process (spawn, communicate, lifecycle) with 30-second timeout on pending requests.
- **DebugManager** -- Synchronous registry for debug sessions and breakpoints.
- **DebugAdapterPool** -- Async pool managing multiple debug adapter instances with
  one adapter per session.
- **Full DAP protocol support** -- initialize, launch, attach, setBreakpoints,
  continue, pause, step (next/stepIn/stepOut), disconnect.
- **Structured logging** via `tracing`.
- **Request/response tracking** -- oneshot channels map DAP response sequence numbers
  back to pending requests.

## State Machine

The `DebugAdapter` follows a strict lifecycle:

```
  Stopped ──► Starting ──► Initializing ──► Configured ──► Running
    ▲             │              │               │            │
    │             │              │               │            │
    │             ▼              ▼               ▼            ▼
    │          Crashed ◄──── Crashed ◄──── Crashed ◄──── Crashed
    │                                                        │
    │                                                        │
    │                                            ShuttingDown│
    │                                                │       │
    └────────────────────────────────────────────────┘       │
                                                     ▲        │
                                                     └────────┘
```

Transitions:
- `Stopped` -> `Starting` (start() called)
- `Starting` -> `Initializing` (process spawned, DAP initialize sent)
- `Starting` -> `Crashed` (spawn failed)
- `Initializing` -> `Configured` (initialize response received, configurationDone sent)
- `Initializing` -> `Crashed` (initialize failed or timed out)
- `Configured` -> `Running` (launch/attach response received)
- `Configured` -> `Crashed` (launch/attach failed)
- `Running` -> `ShuttingDown` (disconnect/terminate requested)
- `Running` -> `Crashed` (adapter process exited unexpectedly)
- `ShuttingDown` -> `Stopped` (adapter exited cleanly)
- `Crashed` -> `Starting` (restart attempt)

## Feature Flags

This crate has no feature flags. It depends only on async-capable crates
(`tokio`, `serde`, `serde_json`, `tracing`, `thiserror`, `uuid`).

## Usage

### Basic debug session

```rust
use dscode_dap::{DebugAdapterPool, DebugManager, DebugSession, DebugState};

async fn example() {
    // Register an adapter type
    let pool = DebugAdapterPool::new();
    pool.register_adapter(
        "cppdbg".to_string(),
        "/usr/bin/gdb".to_string(),
        vec!["--interpreter=dap".to_string()],
    ).await;

    // Create a session via the manager
    let manager = DebugManager::new();
    let session_id = manager.create_session("main".to_string(), "cppdbg".to_string()).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Start the adapter via the pool
    let adapter = pool.create_session(session).await.unwrap();

    // Use the adapter
    adapter.initialize().await.unwrap();
    adapter.launch(serde_json::json!({ "program": "/path/to/binary" })).await.unwrap();
    adapter.set_breakpoints(
        serde_json::json!({ "path": "/path/to/source.cpp" }),
        vec![dscode_dap::SourceBreakpoint {
            line: 10,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: None,
        }],
    ).await.unwrap();
}
```

### Breakpoint management via DebugManager

```rust
use dscode_dap::{DebugManager, SourceBreakpoint};

let manager = DebugManager::new();

// Set breakpoints for a file
manager.set_breakpoints("/path/to/main.rs".to_string(), vec![
    SourceBreakpoint {
        line: 15,
        column: None,
        condition: Some("x > 0".to_string()),
        hit_condition: None,
        log_message: None,
    },
]).unwrap();

// Retrieve breakpoints
let bps = manager.get_breakpoints("/path/to/main.rs").unwrap();

// Clear breakpoints for a file
manager.clear_breakpoints("/path/to/main.rs").unwrap();
```

### Session management

```rust
// List all sessions
let sessions = manager.list_sessions().unwrap();

// Update session state
manager.update_session_state(&session_id, DebugState::Running).unwrap();

// Terminate a session
manager.terminate_session(&session_id).unwrap();
```

### Pool operations

```rust
// Get pool statistics
let stats = pool.get_stats().await;
println!("Active sessions: {}", stats.total_sessions);

// List active session IDs
let session_ids = pool.list_sessions().await;

// Stop a specific session
pool.stop_session(&session_id).await.unwrap();

// Stop all sessions (for shutdown)
pool.stop_all().await.unwrap();
```

## API Surface

### DebugAdapter

| Method | Description |
|--------|-------------|
| `new(session, command, args)` | Create a new adapter instance. |
| `start()` | Spawn the adapter process and begin reading. |
| `stop()` | Kill the adapter process and clean up. |
| `is_running()` | Check if the adapter is in an active state. |
| `get_state()` | Get the current `DebugAdapterState`. |
| `initialize()` | Send DAP `initialize` request. |
| `launch(config)` | Send DAP `launch` request. |
| `attach(config)` | Send DAP `attach` request. |
| `set_breakpoints(source, bps)` | Send DAP `setBreakpoints` request. |
| `continue_execution(thread_id)` | Send DAP `continue` request. |
| `pause(thread_id)` | Send DAP `pause` request. |
| `next(thread_id)` | Send DAP `next` request (step over). |
| `step_in(thread_id)` | Send DAP `stepIn` request. |
| `step_out(thread_id)` | Send DAP `stepOut` request. |
| `disconnect()` | Send DAP `disconnect` request. |
| `send_request(command, args)` | Send a raw DAP request with 30-second timeout. |
| `send_event(event, body)` | Send a raw DAP event. |

### DebugManager

| Method | Description |
|--------|-------------|
| `new()` | Create an empty session manager. |
| `create_session(name, adapter_type)` | Create a new debug session (returns UUID). |
| `get_session(id)` | Look up a session by ID. |
| `list_sessions()` | List all sessions. |
| `update_session_state(id, state)` | Update a session's `DebugState`. |
| `terminate_session(id)` | Remove a session. |
| `set_breakpoints(path, bps)` | Store breakpoints for a file. |
| `get_breakpoints(path)` | Retrieve breakpoints for a file. |
| `clear_breakpoints(path)` | Remove breakpoints for a file. |
| `get_all_breakpoints()` | Get all breakpoints across all files. |

### DebugAdapterPool

| Method | Description |
|--------|-------------|
| `new()` | Create an empty pool. |
| `register_adapter(type, command, args)` | Register a debug adapter type. |
| `create_session(session)` | Create and start an adapter for a session. |
| `get_adapter(session_id)` | Get an existing adapter. |
| `update_state(session_id, state)` | Update a session's state in the pool. |
| `stop_session(session_id)` | Stop and remove an adapter. |
| `stop_all()` | Stop all adapters (for shutdown). |
| `list_sessions()` | List active session IDs. |
| `get_stats()` | Get pool statistics (`DebugPoolStats`). |

### DAP Types

| Type | Description |
|------|-------------|
| `DebugSession` | Session metadata: id, name, state, adapter type. |
| `DebugState` | Session state enum: `Stopped`, `Initialized`, `Running`, `Paused`, `Terminated`. |
| `Source` | DAP source reference (name, path, sourceReference). |
| `SourceBreakpoint` | Breakpoint specification (line, column, condition, hitCondition, logMessage). |
| `Breakpoint` | Verified breakpoint from the adapter. |
| `StackFrame` | Stack frame from a thread. |
| `Thread` | Thread information. |
| `Variable` | Variable from a scope. |
| `Scope` | Scope with variables reference. |
| `LaunchRequestArguments` | Launch configuration (program, args, cwd, env, stopOnEntry). |
| `DapError` | Error type for DAP operations. |

## Related Crates

- `dscode-session` -- Uses `DebugAdapterPool` as part of the IDE session.
- `dscode-extension-host` -- Extension host that may register debug adapters.

## License

MIT