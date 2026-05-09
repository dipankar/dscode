# DSCode Frontend (`src/`)

This directory contains the Svelte-based frontend for DSCode, built with Vite and integrated with the Tauri desktop framework.

## Architecture

The frontend is organized into several layers:

```
src/
├── main.ts           # Application entry point
├── App.svelte        # Root component
├── components/       # UI components (editor, sidebar, panels, status bar)
├── stores/           # Svelte stores for global state
├── lib/              # Utilities, helpers, and API clients
├── themes/           # Theme definitions and CSS custom properties
└── types/            # TypeScript type definitions
```

### State Management

Global state is managed through **Svelte stores** in `src/stores/`:

| Store | Purpose |
|-------|---------|
| `editorStore` | Open documents, active editor, cursor positions |
| `workspaceStore` | File tree, workspace roots, expanded folders |
| `settingsStore` | User preferences, theme, font size, keybindings |
| `terminalStore` | Terminal sessions, active terminal, PTY state |
| `extensionStore` | Loaded extensions, contributed commands, views |

### Component Structure

```
┌─────────────────────────────────────────┐
│              App Shell                   │
│  ┌─────────────────────────────────────┐│
│  │  Title Bar / Menu Bar               ││
│  ├─────────────────────────────────────┤│
│  │  ┌─────────┐ ┌───────────────────┐  ││
│  │  │ Sidebar │ │   Editor Area     │  ││
│  │  │ (file   │ │  (Monaco Editor   │  ││
│  │  │ tree,   │ │   + tabs)         │  ││
│  │  │ search) │ │                   │  ││
│  │  └─────────┘ └───────────────────┘  ││
│  ├─────────────────────────────────────┤│
│  │  Panel Area (terminal, problems)    ││
│  ├─────────────────────────────────────┤│
│  │  Status Bar                         ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

### Theming System

DSCode supports three built-in themes:

- **Dark** (default) -- VS Code Dark+ inspired
- **Light** -- VS Code Light+ inspired
- **High Contrast** -- Accessibility-focused high contrast

Themes are implemented via CSS custom properties defined in `src/themes/`. The theme is applied at runtime by setting CSS variables on the document root. A theme detection script runs in `index.html` before any CSS renders to prevent flash-of-unstyled-content.

### IPC Communication

The frontend communicates with the Rust backend via Tauri's IPC system:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Call a Rust command
const result = await invoke('open_file', { path: '/path/to/file.rs' });
```

Events flow bidirectionally:
- **Frontend -> Rust**: `invoke()` for commands
- **Rust -> Frontend**: `emit()` / `listen()` for events

### Build System

- **Vite** -- Development server and production bundler
- **Svelte** -- Component framework
- **TypeScript** -- Type safety
- **Vitest** -- Unit testing

### Code Splitting

The Vite config splits bundles for optimal loading:
- `monaco` -- Monaco Editor (largest chunk)
- `xterm` -- Terminal emulator
- `lucide` -- Icon library

## Getting Started

```bash
# Install dependencies
npm ci

# Start development server (requires Tauri dev backend)
npm run tauri:dev

# Type-check
npm run check

# Run unit tests
npm run test

# Build for production
npm run build
```

## License

MIT
