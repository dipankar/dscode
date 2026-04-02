---
title: Frequently Asked Questions
description: Common questions about DSCode, its features, compatibility, performance, and development status.
---

# Frequently Asked Questions

Find answers to the most common questions about DSCode below. Each question expands to reveal a detailed answer.

---

## General

??? question "What is DSCode?"

    DSCode is a modern, open-source code editor built from the ground up with **Tauri v2** (Rust), **Svelte**, and the **Monaco Editor**. Unlike VS Code, which runs on Electron, DSCode uses a native Rust backend for file operations, git, terminal management, and more -- delivering dramatically lower memory usage and faster startup times while preserving the editing experience developers already know.

    DSCode is **not** a fork of VS Code. It is a clean-room reimplementation that reuses the Monaco Editor component and provides compatibility with VS Code's extension API.

??? question "How is DSCode different from VS Code?"

    The key differences are architectural:

    | Aspect | VS Code | DSCode |
    |---|---|---|
    | **Runtime** | Electron (Chromium + Node.js) | Tauri v2 (native webview + Rust) |
    | **Backend** | Node.js | Rust (48 command modules) |
    | **Memory usage** | ~300--500 MB | ~50 MB |
    | **Cold start** | 1--3 seconds | < 100 ms |
    | **Install size** | ~350 MB | < 30 MB |
    | **Extension host** | Built-in Node.js | Separate Node.js process (NNG IPC) |

    DSCode retains the same **Monaco Editor**, familiar keybindings, and similar UI layout. The result is a faster, lighter editor that feels like VS Code but runs on a native Rust foundation.

??? question "Is DSCode free and open source?"

    Yes. DSCode is fully open source and free to use. The project is dual-licensed under the **MIT License** and **Apache License 2.0**, giving you flexibility to use, modify, and distribute it in both personal and commercial contexts.

    The source code is available on [GitHub](https://github.com/dipankar/dscode).

??? question "What platforms does DSCode support?"

    DSCode supports the three major desktop platforms:

    - **macOS** -- Intel and Apple Silicon (universal binary)
    - **Linux** -- x86_64 and aarch64 (.deb, .rpm, .AppImage)
    - **Windows** -- x86_64 (.msi, .exe installer, portable)

    Tauri v2's native webview layer means DSCode uses the platform's built-in web renderer (WebKit on macOS, WebKitGTK on Linux, WebView2 on Windows) rather than bundling Chromium.

??? question "Is there a portable or standalone version?"

    Yes. Thanks to Tauri's build system, DSCode can produce a **single portable binary** that requires no installation. On Windows, a standalone `.exe` is available alongside the installer. On Linux, the `.AppImage` format provides a portable option. On macOS, the `.app` bundle can be run from any location without installation.

---

## Extensions

??? question "Can I use VS Code extensions in DSCode?"

    Yes. DSCode includes a **Node.js extension host** that provides approximately **75% compatibility** with the VS Code extension API. Extensions are sourced from the [Open VSX Registry](https://open-vsx.org/), an open marketplace for VS Code-compatible extensions.

    Most extensions that rely on standard APIs -- language servers, snippets, themes, linters, formatters -- work out of the box. Extensions that depend on VS Code-proprietary APIs or deeply integrated UI components may require adaptation.

??? question "Which extensions work and which might not?"

    **Generally work well:**

    - Language extensions (Python, Rust, Go, TypeScript, C/C++, Java, etc.)
    - Color themes and icon themes
    - Snippet packs
    - Linters and formatters (ESLint, Prettier, Black, etc.)
    - Git-related extensions (GitLens core features, Git Graph, etc.)
    - Language server-based extensions

    **May need adaptation:**

    - Extensions with custom webview-heavy UIs (e.g., some notebook renderers)
    - Extensions relying on VS Code-proprietary APIs not yet implemented
    - Extensions that directly interact with Electron internals
    - Some debugger extensions (DAP support is partial)

    Check the [VS Code Compatibility Matrix](reference/vscode-compatibility-matrix.md) for detailed API coverage.

??? question "What is the current extension API coverage, and what's the target?"

    DSCode currently implements approximately **75%** of the VS Code extension API surface. This covers the most commonly used namespaces including `vscode.languages`, `vscode.workspace`, `vscode.window`, `vscode.commands`, and `vscode.extensions`.

    The target is **95%+ coverage** by the v1.0 release, focusing on the APIs that real-world extensions actually use. Community feedback and extension compatibility reports directly influence which APIs are prioritized.

---

## Languages & Editing

??? question "What programming languages does DSCode support?"

    DSCode supports **200+ languages** through multiple layers:

    - **Monaco Editor** provides syntax highlighting, bracket matching, and basic IntelliSense for 200+ languages out of the box.
    - **Tree-sitter** is scaffolded for structural syntax analysis but is not yet functional (the parser stub returns empty results). Enhanced highlighting currently relies on LSP semantic tokens.
    - **Language Server Protocol (LSP)** integration enables intelligent features like auto-completion, go-to-definition, find references, hover documentation, and diagnostics for any language with an LSP server.

    DSCode ships with **4 built-in language server configurations**: Python (Pyright), Rust (rust-analyzer), Go (gopls), and JSON (vscode-json-language-server). The extension API supports 24 language provider registration types, and additional language servers can be added through extensions.

??? question "Does DSCode support Vim or Emacs keybindings?"

    Yes, through extensions. Vim emulation is available via the **VSCodeVim** extension (or compatible alternatives from Open VSX), and Emacs keybindings can be installed similarly. Since DSCode supports the VS Code keybinding system, any keybinding extension that works with the standard API will function in DSCode.

---

## Features

??? question "Does DSCode have a built-in terminal?"

    Yes. DSCode includes a fully featured integrated terminal powered by **xterm.js** for rendering and **portable-pty** for pseudo-terminal management. It supports:

    - Multiple terminal instances with tabbed interface
    - Shell integration (bash, zsh, fish, PowerShell, cmd)
    - Cross-platform shell detection
    - Copy/paste, selection, scrollback buffer
    - Customizable font, colors, and cursor style

    Open the terminal with ++ctrl+grave++ (or ++cmd+grave++ on macOS).

??? question "Does DSCode support remote development (SSH, WSL, containers)?"

    Remote development is **planned** but not yet available in the current alpha release. The roadmap includes:

    - **SSH remote targets** -- Edit files on remote machines with a local UI
    - **WSL integration** -- Seamless development inside Windows Subsystem for Linux
    - **Container targets** -- Develop inside Docker or Podman containers

    These features are targeted for a future release. Follow the [Roadmap](roadmap.md) for progress updates.

??? question "Can I import my VS Code settings?"

    Partial settings import is **planned**. DSCode uses the same `settings.json` format as VS Code, so many settings will work if you copy your configuration file directly. To migrate manually today:

    1. Copy `settings.json` from your VS Code config directory to `~/.config/dscode/`
    2. Copy `keybindings.json` and your `snippets/` folder
    3. Reinstall your extensions via the Open VSX marketplace or `.vsix` files

    !!! note
        Not all VS Code settings keys are recognized yet. Unrecognized settings are silently ignored and will not cause errors.

---

## Performance

??? question "How does DSCode's performance compare to VS Code?"

    DSCode is designed for measurably faster performance across all key metrics:

    | Metric | VS Code | DSCode |
    |---|---|---|
    | **Cold start** | 1--3 seconds | < 100 ms |
    | **Memory (idle)** | 300--500 MB | ~50 MB |
    | **Keystroke latency** | 5--15 ms | < 5 ms |
    | **File open (1 MB)** | 50--100 ms | < 50 ms |
    | **Install size** | ~350 MB | < 30 MB |

    These improvements come from replacing Electron with Tauri's native webview, implementing core operations in Rust, and using zero-copy serialization (rkyv) for IPC between the frontend and extension host.

---

## Development & Contributing

??? question "What is the tech stack behind DSCode?"

    DSCode is built with:

    - **Tauri v2** -- Application framework providing native windowing, IPC, and system APIs (Rust)
    - **Svelte** -- Reactive frontend framework for the UI shell
    - **Monaco Editor** -- The same editor component that powers VS Code
    - **Rust** -- Backend logic including 48 command modules for file ops, git, terminal, search, LSP, etc.
    - **Node.js** -- Extension host process running VS Code-compatible extensions
    - **NNG + rkyv** -- High-performance IPC between Rust backend and Node.js extension host
    - **git2** -- Native git operations without requiring an external git binary
    - **Tree-sitter** -- Incremental parsing for syntax analysis (scaffolded; stub pending full integration)
    - **tower-lsp** -- LSP client implementation
    - **xterm.js + portable-pty** -- Integrated terminal

??? question "How do I report bugs?"

    Bug reports are tracked on **GitHub Issues**:

    1. Go to [github.com/dipankar/dscode/issues](https://github.com/dipankar/dscode/issues)
    2. Search existing issues to avoid duplicates
    3. Click **New Issue** and select the **Bug Report** template
    4. Include your OS, DSCode version, steps to reproduce, and any relevant logs

    For log collection instructions, see the [Troubleshooting](troubleshooting.md#collecting-logs) guide.

??? question "How do I contribute to DSCode?"

    DSCode welcomes contributions of all kinds -- code, documentation, testing, and bug reports. To get started:

    1. Read the [Contributing Guide](contributing/building-from-source.md) for development setup
    2. Check the [GitHub Issues](https://github.com/dipankar/dscode/issues) for tasks labeled `good first issue`
    3. Fork the repository, create a branch, make your changes, and open a pull request
    4. Join discussions in [GitHub Discussions](https://github.com/dipankar/dscode/discussions) to propose features or ask questions

    See the [Architecture Overview](architecture/overview.md) to understand the codebase structure before diving in.

??? question "When will DSCode v1.0 be released?"

    DSCode is currently at **v0.1.0-alpha**. The path to v1.0 involves completing several milestones including full search and replace, customization UI, performance optimization, accessibility improvements, and comprehensive documentation.

    There is no fixed date for v1.0. Development is driven by community feedback and contribution velocity. See the [Roadmap](roadmap.md) for the current plan and phase status.

    !!! tip "Influence the timeline"
        The best way to accelerate the roadmap is to contribute. Even bug reports, testing on different platforms, and extension compatibility feedback help move the project forward.

---

## Still Have Questions?

If your question is not answered here:

- **Search the docs** -- Use the search bar at the top of this site
- **GitHub Discussions** -- Ask in [github.com/dipankar/dscode/discussions](https://github.com/dipankar/dscode/discussions)
- **GitHub Issues** -- Report bugs or request features at [github.com/dipankar/dscode/issues](https://github.com/dipankar/dscode/issues)
