# dscode-extension-host

[![docs.rs](https://img.shields.io/docsrs/dscode-extension-host)](https://docs.rs/dscode-extension-host)
[![crates.io](https://img.shields.io/crates/v/dscode-extension-host.svg)](https://crates.io/crates/dscode-extension-host)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Extension host process management, IPC, sandbox, and security for DSCode.

## Install

```bash
cargo add dscode-extension-host
```

## Features

- **Process lifecycle management** — State machine for spawning, monitoring, restarting, and stopping Node.js extension host processes
- **Bidirectional IPC** — Length-prefixed JSON messaging over Unix domain sockets with request/response semantics and timeouts
- **Path validation** — Prevents filesystem traversal attacks by restricting access to allowed directories only
- **Capability-based permissions** — Fine-grained permission model for extensions (file system, network, clipboard, secrets, UI, terminal, etc.)
- **Per-extension rate limiting** — Token bucket algorithm via `governor` to prevent DOS from misbehaving extensions
- **Multi-platform sandboxing** — macOS (resource limits), Linux (bubblewrap or rlimit fallback), Windows (no-op)
- **OS keyring secret storage** — Secure credential storage using the system keychain (Keychain/Credential Manager/Secret Service)

## Feature Flags

| Feature  | Default | Description |
|----------|---------|-------------|
| `tauri`  | off     | Enables Tauri `AppHandle` integration in the extension host manager |

## Architecture

### IPC (Inter-Process Communication)

The IPC system provides bidirectional communication between the DSCode backend and Node.js extension host processes:

- **Outgoing**: `ExtensionIpc` connects to extension host processes, sends requests with 30-second timeouts, and tracks pending responses via oneshot channels
- **Incoming**: `IncomingIpc` listens on a Unix domain socket for extension-initiated requests, dispatching them to a configurable handler
- **Manager**: `IpcManager` coordinates multiple connections with automatic retry on connect (up to 10 attempts with linear backoff)

Messages use a length-prefixed JSON protocol: 4-byte big-endian length header followed by a JSON body containing `id`, `type`, and `payload` fields.

### Sandbox

The sandbox module applies platform-specific process isolation:

- **Linux**: Uses `bubblewrap` (bwrap) if available for full namespace isolation (mount, PID, network, UTS). Falls back to `setrlimit` for memory constraints
- **macOS**: Sets `NODE_ENV=production` and applies `setrlimit` for memory limits
- **Windows**: No-op (future: Job Object API)

Default sandbox config: network disabled, file write disabled, 512MB memory limit, 50% CPU limit.

### Permissions

Extensions declare required permissions in their manifest using dot-notation (e.g., `fileSystem.read`, `network.http`). The `ExtensionPermissions` struct parses and enforces these at runtime via `check_permission()`.

### Secrets

Secrets are stored in the OS keyring via the `keyring` crate, with an in-memory cache for performance. Each secret is namespaced by extension ID.

## Usage

```rust
use dscode_extension_host::{
    ExtensionHostManager, ExtensionHostState,
    IpcManager, PathValidator, ExtensionPermissions, Permission,
    RateLimiter, SandboxConfig, SecretStorage,
};

// Create an extension host manager and start the process
let mut host = ExtensionHostManager::new("my-extension".to_string());
host.start("/path/to/extension-host.js", "ipc://out.sock", "ipc://in.sock")?;

// Check state
assert_eq!(host.state(), ExtensionHostState::Running);

// Set up IPC
let ipc = IpcManager::new();
ipc.connect_outgoing("my-extension", "ipc:///tmp/out.sock").await?;

// Validate paths
let mut validator = PathValidator::new();
validator.add_workspace_folder(std::env::current_dir()?);
let safe_path = validator.validate_path("file:///workspace/src/main.rs")?;

// Check permissions
let perms = ExtensionPermissions::from_manifest(
    "my-extension".to_string(),
    Some(vec!["fileSystem.read".to_string()]),
)?;
perms.check_permission(&Permission::FileSystemRead)?; // Ok
perms.check_permission(&Permission::FileSystemWrite)?; // Err

// Rate limiting
let limiter = RateLimiter::new();
limiter.check_rate_limit("my-extension")?;

// Secret storage
let secrets = SecretStorage::new();
secrets.set("my-extension", "api_key", "secret-value")?;
let value = secrets.get("my-extension", "api_key")?;

// Graceful shutdown
host.shutdown();
```

## Security Model

1. **Path validation** — All filesystem access from extensions is validated against an allowlist of directories (workspace, extensions, storage, logs, temp). Path traversal attacks are blocked by canonicalizing paths before checking. Error messages are sanitized to avoid leaking filesystem structure.

2. **Sandbox** — Extension host processes run in sandboxed environments with restricted filesystem access, disabled network (by default), and memory/CPU limits.

3. **Permissions** — Extensions must explicitly declare required capabilities in their manifest. No permissions are granted by default.

4. **Rate limiting** — Each extension is limited to 100 requests/second by default (configurable). This prevents a misbehaving extension from overwhelming the host process.

5. **Secret storage** — Secrets never touch disk unencrypted. They are stored in the OS keyring and cached in memory only.

## State Machine

`ExtensionHostManager` uses a strict state machine for process lifecycle:

```
  Stopped ──────> Starting ──────> Running
    ^                |                |
    |    (spawn fail)|   (unexpected) |  (stop() called)
    |                v      (exit)    v
    |             Crashed <──── Unhealthy      Stopping
    |                ^              |              |
    |   (max retries)|              | (within      |
    |                |              |  budget)     |
    |                |              v              |
    |                └──────── Restarting          |
    |                                             |
    └─────────────────────────────────────────────┘
```

- **Stopped**: Safe resting state, no child process
- **Starting**: Spawning child process
- **Running**: Normal operation, process alive
- **Unhealthy**: Process exited unexpectedly, awaiting restart decision
- **Restarting**: Waiting for exponential backoff (1s, 2s, 4s) before re-spawn
- **Stopping**: Graceful shutdown in progress
- **Crashed**: Terminal state after 3 failed restart attempts

## Full API Docs

<https://docs.rs/dscode-extension-host>

## License

MIT