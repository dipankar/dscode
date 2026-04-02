---
title: Roadmap
description: The development roadmap for DSCode, from the current alpha to v1.0 and beyond.
---

# Roadmap

DSCode is under active development. This roadmap outlines the major phases of work, what has been completed, what is in progress, and what is planned for future releases.

!!! info "Current Version: v0.1.0-alpha"
    DSCode is in its initial alpha release. Core editing, terminal, git, and extension hosting are functional. Expect rough edges and breaking changes as development continues.

---

## Phase Overview

| Phase | Name | Status | Target |
|:---:|---|:---:|---|
| 1 | [Core Editor](#phase-1-core-editor) | :material-check-circle:{ .status-full } **Done** | Foundation |
| 2 | [Language Intelligence](#phase-2-language-intelligence) | :material-check-circle:{ .status-full } **Done** | v0.1.0-alpha |
| 3 | [Extension System](#phase-3-extension-system) | :material-check-circle:{ .status-full } **Done** | v0.1.0-alpha |
| 4 | [Git Integration](#phase-4-git-integration) | :material-check-circle:{ .status-full } **Done** | v0.1.0-alpha |
| 5 | [Terminal](#phase-5-terminal) | :material-check-circle:{ .status-full } **Done** | v0.1.0-alpha |
| 6 | [Search](#phase-6-search) | :material-circle-half-full:{ .status-partial } **In Progress** | v0.2.0-alpha |
| 7 | [Customization](#phase-7-customization) | :material-circle-half-full:{ .status-partial } **In Progress** | v0.2.0-alpha |
| 8 | [Polish & Release](#phase-8-polish--release) | :material-circle-outline:{ .status-planned } **Planned** | v1.0.0 |
| -- | [Future](#future) | :material-circle-outline:{ .status-planned } **Planned** | Post v1.0 |

---

## Phase 1: Core Editor

:material-check-circle:{ .status-full } **Done**

The foundational editing experience, establishing the application shell and core editor functionality.

| Feature | Status | Details |
|---|:---:|---|
| Monaco Editor integration | :material-check-circle:{ .status-full } | Full editor component with IntelliSense, multi-cursor, minimap |
| File operations (open, save, rename, delete) | :material-check-circle:{ .status-full } | Rust backend with async file I/O |
| Tab management | :material-check-circle:{ .status-full } | Multi-tab editing with dirty indicators, tab reordering |
| Activity bar and sidebar | :material-check-circle:{ .status-full } | Explorer, Search, SCM, Extensions, Debug views |
| Status bar | :material-check-circle:{ .status-full } | Language mode, encoding, line/column, git branch |
| Command palette | :material-check-circle:{ .status-full } | Fuzzy search across all commands |
| Basic UI shell (Svelte) | :material-check-circle:{ .status-full } | Responsive layout, split views, panel management |
| Tauri v2 backend scaffold | :material-check-circle:{ .status-full } | 48 Rust command modules, IPC bridge |
| Cross-platform builds | :material-check-circle:{ .status-full } | macOS, Linux, Windows |

---

## Phase 2: Language Intelligence

:material-check-circle:{ .status-full } **Done**

Intelligent language features powered by LSP (and Tree-sitter, partially scaffolded), enabling rich editing experiences across languages.

| Feature | Status | Details |
|---|:---:|---|
| LSP client (tower-lsp) | :material-check-circle:{ .status-full } | Multi-server support, auto-discovery |
| 4 built-in language servers + 24 provider registration types | :material-check-circle:{ .status-full } | Python (Pyright), Rust (rust-analyzer), Go (gopls), JSON; 24 language provider types in the extension API |
| Auto-completion | :material-check-circle:{ .status-full } | LSP-driven completions with documentation |
| Go-to-definition / references | :material-check-circle:{ .status-full } | Peek and navigate across files |
| Hover documentation | :material-check-circle:{ .status-full } | Type info, documentation, signatures |
| Diagnostics (errors, warnings) | :material-check-circle:{ .status-full } | Inline squiggles, problems panel |
| Tree-sitter integration | :material-circle-half-full:{ .status-partial } | Scaffolded (stub); `parse()` returns empty results pending full integration |
| Semantic highlighting | :material-check-circle:{ .status-full } | LSP-driven semantic tokens |

---

## Phase 3: Extension System

:material-check-circle:{ .status-full } **Done**

A VS Code-compatible extension host enabling the use of existing extensions with minimal or no modification.

| Feature | Status | Details |
|---|:---:|---|
| Node.js extension host | :material-check-circle:{ .status-full } | Separate process, NNG IPC with rkyv serialization |
| VS Code API compatibility (~75%) | :material-check-circle:{ .status-full } | Core namespaces: languages, workspace, window, commands |
| Open VSX marketplace integration | :material-check-circle:{ .status-full } | Search, install, update from the registry |
| VSIX package installation | :material-check-circle:{ .status-full } | Install extensions from local `.vsix` files |
| Activation events | :material-check-circle:{ .status-full } | onLanguage, onCommand, workspaceContains, etc. |
| Extension contributions | :material-check-circle:{ .status-full } | Commands, keybindings, menus, themes, snippets |
| Webview API | :material-check-circle:{ .status-full } | Extension-hosted webview panels |
| Tree view API | :material-check-circle:{ .status-full } | Custom sidebar tree views |

---

## Phase 4: Git Integration

:material-check-circle:{ .status-full } **Done**

Native git operations powered by the `git2` Rust crate, providing fast source control without external dependencies.

| Feature | Status | Details |
|---|:---:|---|
| Repository detection | :material-check-circle:{ .status-full } | Automatic `.git` discovery in workspace |
| Status tracking | :material-check-circle:{ .status-full } | Modified, staged, untracked file indicators |
| Staging and unstaging | :material-check-circle:{ .status-full } | Stage individual files, hunks, or all changes |
| Committing | :material-check-circle:{ .status-full } | Commit message editor with amend support |
| Diff viewer | :material-check-circle:{ .status-full } | Inline and side-by-side diff views |
| Branch management | :material-check-circle:{ .status-full } | Create, switch, delete branches |
| Remote operations | :material-check-circle:{ .status-full } | Push, pull, fetch with credential support |
| SCM sidebar view | :material-check-circle:{ .status-full } | Source control panel with change groups |

---

## Phase 5: Terminal

:material-check-circle:{ .status-full } **Done**

A fully featured integrated terminal for running commands without leaving the editor.

| Feature | Status | Details |
|---|:---:|---|
| xterm.js rendering | :material-check-circle:{ .status-full } | Full terminal emulation with GPU-accelerated rendering |
| portable-pty backend | :material-check-circle:{ .status-full } | Cross-platform PTY management |
| Multiple terminal instances | :material-check-circle:{ .status-full } | Tabbed terminal interface |
| Shell auto-detection | :material-check-circle:{ .status-full } | bash, zsh, fish, PowerShell, cmd |
| Copy/paste and selection | :material-check-circle:{ .status-full } | Standard terminal interactions |
| Customizable appearance | :material-check-circle:{ .status-full } | Font, colors, cursor style, scrollback |

---

## Phase 6: Search

:material-circle-half-full:{ .status-partial } **In Progress**

Comprehensive search and replace across files, powered by ripgrep for maximum performance.

| Feature | Status | Details |
|---|:---:|---|
| Find in files | :material-check-circle:{ .status-full } | Full-text search across workspace |
| Regex search | :material-check-circle:{ .status-full } | Regular expression pattern matching |
| Search results view | :material-check-circle:{ .status-full } | Grouped results with file context |
| Replace in files | :material-circle-half-full:{ .status-partial } | Single and bulk replace operations |
| Search filters | :material-circle-half-full:{ .status-partial } | Include/exclude patterns, file type filters |
| Search history | :material-circle-outline:{ .status-planned } | Recent search queries |
| Saved searches | :material-circle-outline:{ .status-planned } | Bookmark frequently used searches |

---

## Phase 7: Customization

:material-circle-half-full:{ .status-partial } **In Progress**

User-facing customization tools for themes, keybindings, and settings management.

| Feature | Status | Details |
|---|:---:|---|
| Color theme support | :material-check-circle:{ .status-full } | VS Code-compatible color themes |
| Icon theme support | :material-check-circle:{ .status-full } | File icon themes from extensions |
| Dark and light mode | :material-check-circle:{ .status-full } | System-aware theme switching |
| Keybinding editor | :material-circle-half-full:{ .status-partial } | Visual keybinding editor with conflict detection |
| Settings UI | :material-circle-half-full:{ .status-partial } | Searchable settings editor (currently JSON only) |
| Workspace settings | :material-check-circle:{ .status-full } | Per-workspace configuration overrides |
| Profile system | :material-circle-outline:{ .status-planned } | Save and switch between configuration profiles |
| Settings sync | :material-circle-outline:{ .status-planned } | Synchronize settings across machines |

---

## Phase 8: Polish & Release

:material-circle-outline:{ .status-planned } **Planned**

Final refinements, performance optimization, and quality improvements for the v1.0 stable release.

| Feature | Status | Details |
|---|:---:|---|
| Performance optimization | :material-circle-outline:{ .status-planned } | Startup time, memory profiling, rendering benchmarks |
| Accessibility (a11y) | :material-circle-outline:{ .status-planned } | Screen reader support, keyboard navigation, ARIA labels |
| Localization (i18n) | :material-circle-outline:{ .status-planned } | Multi-language UI support |
| Comprehensive documentation | :material-circle-outline:{ .status-planned } | User guide, API reference, extension development guide |
| Automated testing | :material-circle-outline:{ .status-planned } | E2E tests, integration tests, CI/CD pipeline |
| Auto-update mechanism | :material-circle-outline:{ .status-planned } | In-app update notifications and installation |
| Telemetry (opt-in) | :material-circle-outline:{ .status-planned } | Anonymous usage statistics for improving DSCode |
| Extension API expansion (~95%) | :material-circle-outline:{ .status-planned } | Closing API gaps based on extension compatibility reports |

---

## Future

:material-circle-outline:{ .status-planned } **Post v1.0**

Long-term features planned for releases beyond v1.0.

| Feature | Details |
|---|---|
| **Remote development (SSH)** | Connect to remote machines and edit files with a local UI |
| **WSL integration** | Seamless development inside Windows Subsystem for Linux |
| **Container targets** | Develop inside Docker / Podman containers |
| **Notebook support** | Jupyter-style interactive notebooks |
| **AI integration** | Code completion, chat, and refactoring powered by LLMs |
| **Plugin marketplace** | Community-driven marketplace with publishing tools |
| **Collaborative editing** | Real-time multi-user editing sessions |
| **Custom layout system** | Drag-and-drop panel arrangement and saved layouts |
| **Task runner improvements** | Enhanced `tasks.json` support, task output parsing |
| **Workspace trust** | Security model for untrusted workspaces |

---

## How to Influence the Roadmap

The DSCode roadmap is shaped by community feedback and contributions. Here is how you can participate:

### Vote on Features

Visit [GitHub Discussions -- Feature Requests](https://github.com/dipankar/dscode/discussions/categories/feature-requests) to browse proposed features. Add a :thumbsup: reaction to features you want prioritized.

### Report Compatibility Issues

If a VS Code extension does not work in DSCode, [open an issue](https://github.com/dipankar/dscode/issues) with the extension name and the error. This directly informs which APIs are prioritized for implementation.

### Contribute Code

Pull requests are the fastest way to move the roadmap forward. See the [Contributing Guide](contributing/building-from-source.md) to get started. Issues labeled `good first issue` and `help wanted` are great entry points.

### Join the Discussion

Participate in design discussions, RFC proposals, and architecture decisions in [GitHub Discussions](https://github.com/dipankar/dscode/discussions). Your input helps shape the direction of the project.

!!! tip "Stay Updated"
    Watch the [DSCode repository](https://github.com/dipankar/dscode) on GitHub and enable notifications for releases to stay informed about new versions and milestones.
