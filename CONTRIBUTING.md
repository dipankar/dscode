# Contributing to DSCode

First off, thanks for taking the time to contribute! DSCode is a community-driven project, and every contribution matters.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- **Node.js** 20+ and npm
- **Rust** stable (1.75+) with cargo
- **Platform dependencies**:
  - **Linux**: `libwebkit2gtk-4.0-dev`, `libssl-dev`, `libgtk-3-dev`, `librsvg2-dev`
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Windows**: Microsoft Visual C++ Build Tools

### Development Setup

1. **Clone the repository**:

   ```bash
   git clone https://github.com/dscode-dev/dscode.git
   cd dscode
   ```

2. **Install frontend dependencies**:

   ```bash
   npm install
   ```

3. **Start development mode** (hot reload):
   ```bash
   npm run tauri:dev
   ```

## Build Commands

Run these checks before submitting a pull request:

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

# Full Tauri production build
npm run tauri:build

# Lint
npm run lint
```

### Required pre-commit checks

All of the following must pass before committing:

1. `npx tsc --noEmit` — frontend TypeScript
2. `cd src-tauri && cargo check` — Rust
3. `cd extension-host && npx tsc --noEmit` — extension host TypeScript
4. `npx vite build` — production build

## Code Style

### Rust

- Format with `cargo fmt`
- Lint with `cargo clippy -- -D warnings`
- Use `tokio::sync::Mutex` (never `std::sync::Mutex`) in async contexts
- Use `tokio::task::spawn_blocking` for filesystem I/O in Tauri command handlers
- No comments unless explicitly requested

### TypeScript & Svelte

- Format with Prettier: `npm run format`
- Lint with ESLint: `npm run lint`
- Type-check with `npm run check` (svelte-check)
- Use `showConfirmPrompt()`/`showAlertPrompt()` from `stores/windowPrompt` instead of `confirm()`/`alert()`
- Components must unsubscribe from stores in `onDestroy()` to prevent memory leaks
- No comments unless explicitly requested

### Styling

- CSS custom properties only (VS Code theme-compatible)
- No Tailwind

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
type(scope): description

[optional body]
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`, `build`

Examples:

- `feat(editor): add multi-cursor support`
- `fix(terminal): resolve PTY close race condition`
- `refactor(session): extract workspace state into separate module`

## Pull Request Process

1. **Fork** the repository and create a branch from `main`
2. **Make changes** with clear, descriptive commits following conventional commits
3. **Run all pre-commit checks** (listed above)
4. **Open a PR** against `main` with a clear description of the change
5. **Ensure CI passes** — all type checks, lints, and builds must succeed
6. **Address review feedback** promptly
7. **One approval required** for merge (may increase as the project grows)

### PR Title

Use the same conventional commit format: `type(scope): description`

## Architecture Overview

DSCode has three main layers:

- **Frontend**: Svelte 4 + TypeScript + Monaco Editor + xterm.js
- **Backend**: Tauri 2.1 (Rust) with tokio async runtime
- **Extension Host**: Separate Node.js process communicating via NNG IPC

See [docs/architecture/overview.md](docs/architecture/overview.md) for the full architecture document and [docs/](docs/) for additional design docs.

## Testing

```bash
# Rust tests
cd src-tauri && cargo test

# Extension host tests
cd extension-host && npm test

# Svelte component type check
npm run check
```

## Questions?

Open an issue on [GitHub](https://github.com/dscode-dev/dscode/issues) or start a discussion.
