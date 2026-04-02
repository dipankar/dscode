# dscode-terminal

[![docs.rs](https://img.shields.io/docsrs/dscode-terminal)](https://docs.rs/dscode-terminal)
[![crates.io](https://img.shields.io/crates/v/dscode-terminal.svg)](https://crates.io/crates/dscode-terminal)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Terminal manager and PTY lifecycle management for DSCode.

This crate provides a portable PTY-backed terminal manager that can be used
independently of any UI framework. Event forwarding is abstracted behind the
`TerminalEventSender` trait, so you can integrate with any event system.

## Install

```bash
cargo add dscode-terminal
```

## Features

- **Profile management** -- register, unregister, and list shell profiles with
  custom arguments, environment variables, and working directories.
- **PTY lifecycle** -- spawn, resize, write to, and close terminal instances
  backed by `portable-pty`.
- **Pluggable event forwarding** -- implement `TerminalEventSender` to route
  PTY output and close events to any consumer (Tauri frontend, web socket,
  log file, etc.).
- **Tauri integration** -- optional `TauriEventSender` behind the `tauri`
  feature flag for drop-in use with Tauri applications.
- **Graceful shutdown** -- `close_all()` and `Drop` implementations ensure PTY
  resources and reader threads are cleaned up properly.
- **Thread-safe** -- all state is guarded by `std::sync::Mutex`; reader threads
  communicate via `Arc<dyn TerminalEventSender>` and `AtomicBool` shutdown
  signals.

## State Machine

Each `TerminalInstance` follows a strict lifecycle:

```
  Created ──────► Running ──────► ShuttingDown ──────► Closed
                     │                                    ▲
                     │ (process exit,                     │
                     │  read error)                       │
                     └────────────────────────────────────┘
```

Transitions:
- `Created` -> `Running` (start signal sent via `start_reading()`)
- `Running` -> `ShuttingDown` (shutdown flag set by `close_terminal()`)
- `Running` -> `Closed` (PTY process exits, reader thread detects EOF)
- `ShuttingDown` -> `Closed` (reader thread sees shutdown flag, exits loop)

Concurrency:
- PTY I/O uses `std::sync::Mutex` (not tokio) because PTY operations are
  synchronous. The reader thread runs on a dedicated OS thread (not a tokio
  task). `ShutdownSignal` uses `AtomicBool` for lock-free cross-thread
  signaling. State transitions are synchronized through `TerminalManager`'s
  `terminals` mutex.

## Feature Flags

| Feature  | Default | Description |
|----------|---------|-------------|
| `tauri`  | off     | Enables `TauriEventSender`, which forwards events through the Tauri event system |

## Usage

### Basic terminal creation

```rust
use dscode_terminal::{TerminalManager, TerminalEventSender};

// Implement the trait for your event system
struct MySender;
impl TerminalEventSender for MySender {
    fn send_output(&self, terminal_id: &str, data: &str) {
        println!("[{}] {}", terminal_id, data);
    }
    fn send_close(&self, terminal_id: &str) {
        println!("[{}] closed", terminal_id);
    }
}

let manager = TerminalManager::new(Box::new(MySender));

// Create a terminal with default shell
let id = manager.create_terminal(None, None, None)?;

// Or use the options builder
let options = TerminalOptions {
    name: Some("Build".into()),
    shell_path: Some("/bin/bash".into()),
    shell_args: vec!["-l".into()],
    cwd: Some("/project".into()),
    env: HashMap::new(),
    profile_id: None,
};
let id = manager.create_terminal_with_options(options)?;

// Start reading PTY output
manager.start_reading(&id)?;

// Write to the terminal
manager.write_to_terminal(&id, "ls -la\n")?;

// Resize
manager.resize_terminal(&id, 120, 40)?;

// Close a single terminal
manager.close_terminal(&id)?;

// Close all terminals (also called automatically on Drop)
manager.close_all();
```

### Using profiles

```rust
let profile = TerminalProfile {
    id: "wsl".into(),
    name: "WSL Ubuntu".into(),
    shell: "/usr/bin/wsl".into(),
    args: vec!["-d".into(), "Ubuntu".into()],
    env: HashMap::new(),
    cwd: None,
    icon: Some("terminal-wsl".into()),
    color: Some("#4CAF50".into()),
};

manager.register_profile(profile)?;

let options = TerminalOptions {
    name: Some("WSL Build".into()),
    shell_path: None,
    shell_args: vec![],
    cwd: None,
    env: HashMap::new(),
    profile_id: Some("wsl".into()),
};
let id = manager.create_terminal_with_options(options)?;
```

### Tauri integration

Enable the `tauri` feature and use `TauriEventSender`:

```toml
[dependencies]
dscode-terminal = { version = "0.1.0", features = ["tauri"] }
```

```rust
use dscode_terminal::{TerminalManager, TauriEventSender};

let sender = TauriEventSender::new(app_handle);
let manager = TerminalManager::new(Box::new(sender));
```

`TauriEventSender` emits the following events:

| Event name                    | Payload            |
|-------------------------------|--------------------|
| `terminal-data:{terminal_id}` | `{ "data": "..." }` |
| `terminal-closed`             | `terminal_id`      |

## API Surface

### Types

| Type | Description |
|------|-------------|
| `TerminalManager` | Main entry point. Manages profiles and terminal instances. |
| `TerminalInfo` | Metadata about a running terminal (id, name, shell, cwd). |
| `TerminalInstance` | Internal PTY state (writer, reader handle, shutdown signal). |
| `TerminalOptions` | Configuration for creating a new terminal. |
| `TerminalProfile` | Saved shell configuration with id, shell path, args, env, cwd. |
| `TerminalState` | Lifecycle state enum: `Created`, `Running`, `ShuttingDown`, `Closed`. |
| `TerminalEventSender` | Trait for receiving PTY output and close events. |
| `TauriEventSender` | Tauri-backed implementation of `TerminalEventSender` (feature: `tauri`). |
| `TerminalError` | Error type for terminal operations. |

### TerminalManager Methods

| Method | Description |
|--------|-------------|
| `new(event_sender)` | Create a manager with a custom event sender. |
| `register_profile(profile)` | Register a shell profile. |
| `unregister_profile(id)` | Remove a shell profile. |
| `get_profile(id)` | Look up a profile by id. |
| `list_profiles()` | List all registered profiles. |
| `create_terminal(name, shell, cwd)` | Create a terminal with simple options. |
| `create_terminal_with_options(opts)` | Create a terminal with full `TerminalOptions`. |
| `start_reading(id)` | Signal the reader thread to start forwarding output. |
| `write_to_terminal(id, data)` | Write data to the terminal's stdin. |
| `resize_terminal(id, cols, rows)` | Resize the terminal's PTY. |
| `close_terminal(id)` | Close a single terminal and clean up resources. |
| `close_all()` | Close all terminals. Called automatically on `Drop`. |
| `list_terminals()` | List `TerminalInfo` for all active terminals. |

## Related Crates

- `dscode-session` -- Uses `TerminalManager` as part of the IDE session.
- `dscode-extension-host` -- Extension host that may create terminals via the session.

## Full API Docs

<https://docs.rs/dscode-terminal>

## License

MIT