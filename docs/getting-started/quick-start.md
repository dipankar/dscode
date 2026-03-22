# DSCode Quick Start

## 🚀 Get Running in 5 Minutes

### 1. Install Dependencies

```bash
npm install
```

### 2. Run Development Build

```bash
npm run tauri:dev
```

That's it! DSCode will launch with:
- ✅ Monaco Editor (same as VS Code)
- ✅ Activity Bar, Sidebar, Editor, Panels, Status Bar
- ✅ Rust backend for file operations
- ✅ Hot reload for instant feedback

## 📁 Project Structure

```
dscode/
├── src/                    # Svelte Frontend
│   ├── components/         # UI Components
│   ├── App.svelte
│   └── main.ts
├── src-tauri/              # Rust Backend
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   └── core/           # Core logic
│   └── Cargo.toml
└── docs/                   # Architecture docs
```

## 🎯 What Works Now

- ✅ Monaco Editor with TypeScript
- ✅ VS Code-like UI layout
- ✅ File read/write (Rust backend)
- ✅ Git status (placeholder)
- ✅ Activity bar navigation
- ✅ Terminal panel
- ✅ Status bar

## 🔧 Key Commands

```bash
# Development
npm run tauri:dev        # Run with hot reload

# Build
npm run tauri:build      # Production build

# Testing
npm run check            # Type checking
npm run lint             # Linting

# Rust
cd src-tauri
cargo test              # Run Rust tests
cargo build --release   # Release build
```

## 📝 Next Steps

1. **Add file explorer functionality** - Connect sidebar to real file system
2. **Implement Monaco LSP** - Hook up Language Server Protocol
3. **Add extension system** - Node.js extension host
4. **Git integration** - Real git operations with git2
5. **Terminal integration** - Embedded terminal with PTY

See [getting-started.md](getting-started.md) for detailed development guide.

## 🐛 Troubleshooting

**Monaco not loading?**
- Clear `node_modules` and reinstall

**Tauri fails to start?**
- Check platform dependencies are installed
- See GETTING_STARTED.md for platform-specific requirements

**Rust compilation errors?**
- Run `cd src-tauri && cargo clean && cargo build`

## 📚 Documentation

- [Architecture](../architecture/overview.md) - System design
- [Getting Started](getting-started.md) - Full development guide
- [Roadmap](../development/implementation-timeline.md) - What's next
