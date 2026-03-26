# Performance Architecture

DSCode is designed for measurable performance advantages over Electron-based editors. This document covers the memory allocator strategy, resource monitoring, IPC optimizations, and frontend performance techniques.

---

## Performance Targets

| Metric | Target | How |
|--------|--------|-----|
| Cold start | <100ms | Lazy initialization, no bundled Chromium |
| Idle memory | ~50 MB | Native webview, Rust backend, no V8 in main process |
| Keystroke latency | <5ms | Direct Monaco events, no IPC for local edits |
| File open (1 MB) | <50ms | Async Rust I/O with ropey text buffer |
| IPC round-trip | <1ms | NNG sockets with efficient serialization |

---

## Memory Allocators

**Source:** `src-tauri/src/main.rs`

DSCode supports compile-time selection of the memory allocator via Cargo feature flags. The choice of allocator affects allocation throughput, fragmentation behavior, and memory overhead.

### Available Allocators

| Allocator | Feature Flag | Crate | Best For |
|-----------|-------------|-------|----------|
| **jemalloc** | `jemalloc` | `tikv-jemallocator` | General use; low fragmentation under concurrent workloads |
| **mimalloc** | `mimalloc-allocator` | `mimalloc` | High-throughput allocation; small objects |
| **System** | `system-allocator` | (default Rust) | Minimal dependencies; debugging with system tools |

### Configuration

```toml title="Cargo.toml"
[features]
default = ["jemalloc"]
jemalloc = ["tikv-jemallocator"]
mimalloc-allocator = ["mimalloc"]
system-allocator = []
```

```rust title="main.rs"
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(feature = "mimalloc-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

!!! tip "Choosing an allocator"
    **jemalloc** is the default and recommended for production builds. It provides the best balance of performance and memory efficiency for DSCode's mixed workload (many small allocations from IPC, large buffers from file I/O). Use **mimalloc** if you need maximum allocation throughput. Use **system** for profiling with platform-native tools (e.g., Instruments on macOS, Valgrind on Linux).

---

## Resource Monitoring

**Source:** `src-tauri/src/monitoring/resource_monitor.rs`

The `ResourceMonitor` tracks CPU and memory usage for both the main process and the extension host process using the `sysinfo` crate.

### Metrics Collected

```rust
pub struct ResourceMetrics {
    pub timestamp: u64,
    pub main_process: ProcessMetrics,
    pub extension_host: Option<ProcessMetrics>,
    pub system: SystemMetrics,
}

pub struct ProcessMetrics {
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub pid: u32,
}

pub struct SystemMetrics {
    pub total_cpu_percent: f32,
    pub total_memory_mb: f64,
    pub used_memory_mb: f64,
    pub cpu_count: usize,
}
```

### How It Works

1. The `ResourceMonitor` is initialized at startup with the main process PID.
2. When the extension host starts, its PID is registered via `set_extension_host_pid()`.
3. On each `get_metrics()` call, the monitor refreshes only the relevant processes (not all system processes) for efficiency.
4. Metrics include per-process CPU and memory usage, plus system-wide totals.

### Usage

Resource metrics are exposed through a Tauri command and can be viewed in the developer tools or a future status bar indicator:

```typescript
const metrics = await invoke('get_resource_metrics');
console.log(`Main: ${metrics.main_process.memory_mb}MB`);
console.log(`Ext Host: ${metrics.extension_host?.memory_mb}MB`);
```

---

## IPC Performance

### Tauri IPC (Frontend ↔ Backend)

Communication between the Svelte frontend and Rust backend uses Tauri's built-in IPC mechanism:

- **Protocol:** JSON serialization over platform-native IPC
- **Latency:** <1ms for simple commands
- **Throughput:** Sufficient for UI-driven operations (file open, save, settings)

### NNG IPC (Backend ↔ Extension Host)

Communication between the Rust backend and the Node.js extension host uses NNG (nanomsg-next-gen) sockets:

- **Pattern:** Request/Reply
- **Serialization:** JSON (rkyv zero-copy serialization planned)
- **Latency:** Sub-millisecond for typical API calls
- **Benefits:** No TCP overhead, kernel-optimized, supports async operations

```mermaid
graph LR
    Rust["Rust Backend"] <-->|"NNG req/rep<br/>< 1ms"| Node["Extension Host<br/>(Node.js)"]
```

---

## Frontend Optimizations

### Debouncing

High-frequency events (typing, scrolling, resizing) are debounced before triggering IPC calls:

| Event | Debounce Interval | Purpose |
|-------|------------------|---------|
| Text changes | 300ms | Batch document sync to extension host |
| Window resize | 100ms | Avoid excessive layout recalculations |
| Search input | 200ms | Reduce search queries while typing |
| Settings changes | 500ms | Batch configuration update events |

### Caching

The frontend caches frequently accessed data to avoid redundant IPC calls:

- **File content cache** -- Recently opened file contents with LRU eviction
- **Configuration cache** -- Settings values cached in Svelte stores
- **Extension metadata cache** -- Installed extension info cached at startup
- **Theme cache** -- Resolved theme colors cached to avoid re-parsing

### Request Cancellation

Long-running operations support cancellation to prevent stale results:

- Completion requests are cancelled when the user continues typing
- Search operations are cancelled when the query changes
- Hover requests are cancelled when the cursor moves away

```typescript
// Example: cancellable completion request
let controller = new AbortController();
const results = await invoke('get_completions', {
    signal: controller.signal,
    ...params
});
```

---

## Startup Optimization

### Lazy Initialization

DSCode minimizes cold start time through aggressive lazy initialization:

| Component | When Initialized |
|-----------|-----------------|
| Monaco Editor | Immediately (required for UI) |
| File Watcher | On first folder open |
| Language Servers | On first file of that language opened |
| Extension Host | On first extension activation |
| Git Operations | On first SCM view open |
| Debug Manager | On first debug session |
| Resource Monitor | On first metrics request |

### Bootstrap Sequence

The application bootstrap (`bootstrap.rs`) registers all feature registries and command modules at startup, but actual resource initialization is deferred:

1. **Phase 1 (immediate):** Window creation, UI rendering, registry scaffolding
2. **Phase 2 (deferred):** Extension host startup, language server registration
3. **Phase 3 (on-demand):** Individual language server startup, debug adapter launch

---

## Benchmarking

To benchmark DSCode performance on your system:

```bash
# Measure cold start time
time dscode --wait --new-window /tmp/empty-folder

# Profile memory usage
dscode --status  # Shows process info and memory usage

# Enable performance logging
dscode --log-level=trace --log-file=/tmp/dscode-perf.log
```

---

## See Also

- [Architecture Overview](overview.md) -- system design and process model
- [IPC Design](ipc-design.md) -- detailed IPC protocol documentation
- [Rust Backend](rust-backend.md) -- backend module architecture
