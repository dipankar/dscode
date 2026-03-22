# Getting Started with DSCode Development

Welcome to DSCode! This guide will help you set up the development environment and start contributing.

## Prerequisites

Before you begin, ensure you have the following installed:

### Required
- **Node.js** (v18+) and **npm** or **pnpm**
- **Rust** (latest stable) - Install from [rustup.rs](https://rustup.rs/)
- **Tauri CLI**

### Platform-Specific Dependencies

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Windows
- Install [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/dscode.git
cd dscode
```

### 2. Install Dependencies

```bash
# Install Node.js dependencies
npm install

# Install Tauri CLI globally (optional but recommended)
npm install -g @tauri-apps/cli
```

### 3. Build Rust Backend

```bash
cd src-tauri
cargo build
cd ..
```

## Development

### Run Development Server

```bash
# Start both Vite dev server and Tauri
npm run tauri:dev
```

This will:
1. Start Vite dev server on `http://localhost:1420`
2. Launch Tauri app with hot-reload enabled
3. Watch for changes in both frontend and backend

### Project Structure

```
dscode/
├── src/                        # Frontend (Svelte + TypeScript)
│   ├── components/             # Svelte components
│   │   ├── ActivityBar.svelte
│   │   ├── Sidebar.svelte
│   │   ├── EditorArea.svelte  # Monaco editor integration
│   │   ├── PanelArea.svelte
│   │   └── StatusBar.svelte
│   ├── lib/                    # Shared utilities
│   ├── stores/                 # Svelte stores (state management)
│   ├── App.svelte              # Root component
│   └── main.ts                 # Entry point
│
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── main.rs             # Tauri app entry
│   │   ├── commands/           # Tauri commands (IPC handlers)
│   │   │   ├── file_ops.rs     # File operations
│   │   │   ├── git_ops.rs      # Git operations
│   │   │   └── app.rs          # App utilities
│   │   └── core/               # Core functionality
│   │       ├── text_buffer.rs  # Rope-based text buffer
│   │       └── syntax.rs       # Tree-sitter integration
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
│
├── docs/                       # Documentation
├── index.html                  # HTML entry point
├── package.json                # Node.js dependencies
├── vite.config.ts              # Vite configuration
└── tailwind.config.js          # Tailwind CSS config
```

## Key Technologies

### Frontend
- **Svelte** - Reactive UI framework
- **Monaco Editor** - VS Code's editor (same one!)
- **TypeScript** - Type safety
- **Tailwind CSS** - Styling
- **Vite** - Build tool

### Backend (Rust)
- **Tauri** - Desktop app framework
- **ropey** - Text buffer (rope data structure)
- **tree-sitter** - Syntax parsing
- **tower-lsp** - Language Server Protocol
- **git2** - Git integration
- **nng** - IPC for backend services

## Making Changes

### Adding a New Tauri Command

1. Create command in `src-tauri/src/commands/`:

```rust
// src-tauri/src/commands/my_feature.rs
#[tauri::command]
pub async fn my_command(arg: String) -> Result<String, String> {
    Ok(format!("Hello, {}!", arg))
}
```

2. Register in `src-tauri/src/main.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_command,
])
```

3. Call from frontend:

```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('my_command', { arg: 'World' });
```

### Adding a New Component

1. Create component in `src/components/`:

```svelte
<!-- src/components/MyComponent.svelte -->
<script lang="ts">
  export let prop: string;
</script>

<div class="my-component">
  {prop}
</div>

<style>
  .my-component {
    /* styles */
  }
</style>
```

2. Import and use in parent component:

```svelte
<script>
  import MyComponent from './components/MyComponent.svelte';
</script>

<MyComponent prop="value" />
```

## Testing

```bash
# Run frontend tests
npm test

# Run Rust tests
cd src-tauri
cargo test

# Check TypeScript types
npm run check

# Lint code
npm run lint
```

## Building for Production

```bash
# Build for current platform
npm run tauri:build

# Output will be in src-tauri/target/release/bundle/
```

## Debugging

### Frontend Debugging
- Use browser DevTools (F12 in dev mode)
- Console logs will appear in terminal and DevTools

### Backend Debugging
- Use `println!()` or `dbg!()` macros
- Logs appear in terminal
- Use `RUST_LOG=debug npm run tauri:dev` for detailed logs

## Common Tasks

### Open File in Editor

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function openFile(path: string) {
  const content = await invoke('read_file', { path });
  // Update Monaco editor with content
}
```

### Listen for File Changes

```rust
// Backend (Tauri command)
use notify::{Watcher, RecursiveMode};

#[tauri::command]
async fn watch_file(path: String, window: tauri::Window) {
    let mut watcher = notify::recommended_watcher(move |res| {
        window.emit("file-changed", &path).ok();
    }).unwrap();

    watcher.watch(Path::new(&path), RecursiveMode::NonRecursive).ok();
}
```

```typescript
// Frontend
import { listen } from '@tauri-apps/api/event';

listen('file-changed', (event) => {
  console.log('File changed:', event.payload);
});
```

## Next Steps

1. **Read Architecture Docs**: See `docs/ARCHITECTURE.md`
2. **Check Implementation Roadmap**: See `docs/IMPLEMENTATION_ROADMAP.md`
3. **Join Discord**: [Link to Discord server]
4. **Pick an Issue**: Check GitHub Issues with "good first issue" label

## Troubleshooting

### Monaco Editor Not Loading
- Check console for errors
- Ensure `monaco-editor` is in dependencies
- Verify Vite config includes Monaco optimization

### Tauri Commands Not Working
- Check command is registered in `main.rs`
- Verify function signature matches invocation
- Check for Rust compilation errors

### Build Failures
- Clear caches: `rm -rf node_modules dist src-tauri/target`
- Reinstall: `npm install && cd src-tauri && cargo build`
- Check platform-specific dependencies are installed

## Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Svelte Tutorial](https://svelte.dev/tutorial)
- [Monaco Editor API](https://microsoft.github.io/monaco-editor/api/index.html)
- [DSCode Architecture](../architecture/overview.md)

Happy coding! 🚀
