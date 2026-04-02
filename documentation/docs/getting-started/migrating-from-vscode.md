# Migrating from VS Code

This guide walks you through migrating from Visual Studio Code to DSCode. DSCode is designed to feel familiar -- your keybindings, themes, snippets, and most extensions carry over directly.

---

## Step 1: Install DSCode

Download and install DSCode for your platform from the [installation guide](installation.md), or build from source:

=== "macOS"

    Download the `.dmg` from GitHub Releases and drag DSCode to Applications.

=== "Linux"

    Install via `.deb`, `.rpm`, or `.AppImage` from GitHub Releases.

=== "Windows"

    Run the `.msi` or `.exe` installer from GitHub Releases.

---

## Step 2: Import Settings

DSCode uses the same `settings.json` format as VS Code. Copy your configuration files:

=== "macOS"

    ```bash
    # VS Code settings location
    # ~/Library/Application Support/Code/User/

    # Copy to DSCode
    cp ~/Library/Application\ Support/Code/User/settings.json \
       ~/Library/Application\ Support/dscode/settings.json

    cp ~/Library/Application\ Support/Code/User/keybindings.json \
       ~/Library/Application\ Support/dscode/keybindings.json

    cp -r ~/Library/Application\ Support/Code/User/snippets/ \
       ~/Library/Application\ Support/dscode/snippets/
    ```

=== "Linux"

    ```bash
    # VS Code settings location
    # ~/.config/Code/User/

    # Copy to DSCode
    cp ~/.config/Code/User/settings.json ~/.config/dscode/settings.json
    cp ~/.config/Code/User/keybindings.json ~/.config/dscode/keybindings.json
    cp -r ~/.config/Code/User/snippets/ ~/.config/dscode/snippets/
    ```

=== "Windows"

    ```powershell
    # VS Code settings location
    # %APPDATA%\Code\User\

    # Copy to DSCode
    Copy-Item "$env:APPDATA\Code\User\settings.json" "$env:APPDATA\dscode\settings.json"
    Copy-Item "$env:APPDATA\Code\User\keybindings.json" "$env:APPDATA\dscode\keybindings.json"
    Copy-Item -Recurse "$env:APPDATA\Code\User\snippets" "$env:APPDATA\dscode\snippets"
    ```

!!! note "Unrecognized settings"
    Not all VS Code settings keys are recognized by DSCode yet. Unrecognized settings are silently ignored and will not cause errors. Core `editor.*`, `workbench.*`, `terminal.*`, and `files.*` settings have high compatibility.

---

## Step 3: Install Extensions

DSCode uses the **Open VSX Registry** instead of the Microsoft Visual Studio Marketplace. Most popular extensions are available on both, but some Microsoft-exclusive extensions may not be present.

### Check Extension Availability

List your VS Code extensions and check availability:

```bash
# Export your VS Code extension list
code --list-extensions > my-extensions.txt

# Review the list and search for each on Open VSX
cat my-extensions.txt
```

### Install Extensions

Install extensions one by one or use the CLI:

```bash
# Install from the marketplace
dscode --install-extension rust-lang.rust-analyzer
dscode --install-extension esbenp.prettier-vscode
dscode --install-extension dbaeumer.vscode-eslint
```

### Common Extension Mapping

| VS Code Extension | Open VSX Availability | Notes |
|---|---|---|
| Python (ms-python) | Available | Full functionality |
| Rust Analyzer | Available | Full functionality |
| Prettier | Available | Full functionality |
| ESLint | Available | Full functionality |
| GitLens | Available | Core features work |
| C/C++ (ms-vscode) | **Not available** | Use `clangd` instead |
| Live Share | **Not available** | VS Code-exclusive |
| GitHub Copilot | **Not compatible** | VS Code-specific API |
| Remote SSH | **Not available** | DSCode remote development is planned |
| Jupyter | Available | Extension host API works; frontend rendering incomplete |

---

## Step 4: Verify Your Workspace

Open your project in DSCode and verify key workflows:

### Tasks

If you have a `.vscode/tasks.json`, test that your tasks run correctly:

1. Press ++ctrl+shift+p++ and type `Tasks: Run Task`.
2. Select a task and verify it executes as expected.

### Debug Configurations

Test your `.vscode/launch.json` debug configurations:

1. Open the Debug view (++ctrl+shift+d++).
2. Select a configuration from the dropdown.
3. Press ++f5++ to start debugging.

!!! warning
    Debug stepping (step over, step into, step out) is currently stubbed in DSCode. Breakpoints, continue, pause, and stop work correctly.

### Keybindings

Test your custom keyboard shortcuts:

1. Press ++ctrl+shift+p++ and type `Keyboard Shortcuts`.
2. Verify your custom bindings appear and work as expected.

---

## Key Architectural Differences

Understanding the differences helps set expectations:

| Area | VS Code | DSCode |
|------|---------|--------|
| **Runtime** | Electron (bundled Chromium + Node.js) | Tauri v2 (native webview + Rust) |
| **Memory usage** | 300-500 MB | ~50 MB |
| **Cold start** | 1-3 seconds | <100 ms |
| **Install size** | ~350 MB | <30 MB |
| **Extension marketplace** | Microsoft Marketplace | Open VSX Registry |
| **Extension API coverage** | 100% | ~75% (targeting 95%+ for v1.0) |
| **Git backend** | Shells out to `git` CLI | Native `git2` Rust crate (no external binary) |
| **Terminal** | Built-in Node.js PTY | `portable-pty` (Rust) + xterm.js |
| **Remote development** | Full (SSH, WSL, Containers, Tunnels) | Not yet available (planned) |
| **Notebook support** | Full (Jupyter) | Extension host API only; frontend incomplete |
| **Language servers** | Bundled via extensions | 4 built-in (Python, Rust, Go, JSON) + extensions |

---

## Common Issues and Workarounds

??? failure "Extension fails to activate"
    - Check the Extension Host log in the Output panel for API compatibility errors.
    - The extension may depend on VS Code APIs not yet implemented in DSCode. See the [Compatibility Matrix](../reference/vscode-compatibility-matrix.md).
    - Try an older version of the extension that may use fewer APIs.

??? failure "Settings key not recognized"
    - DSCode silently ignores unrecognized settings. If a feature is not working as expected, check whether the corresponding setting is supported.
    - Core editor, terminal, and file settings have high compatibility. Some workbench layout and advanced settings may differ.

??? failure "Theme looks different"
    - DSCode supports VS Code TextMate color themes. If colors appear slightly different, it may be due to differences in the webview rendering engine (WebKit/WebView2 vs. Chromium).
    - Workbench colors are fully supported. Token colors use the same TextMate grammar system.

??? failure "Git pull fails with 'Merge required'"
    - DSCode's built-in `git pull` only supports fast-forward merges. If your branch has diverged, use the integrated terminal to run `git pull --rebase` or `git merge` manually.

??? failure "Extension requires Electron APIs"
    - Extensions using `require('electron')` will not work. DSCode provides equivalent functionality through the `vscode.env` namespace.
    - Check if the extension has an Open VSX version that may have been adapted for non-Electron editors.

---

## Getting Help

If you encounter issues during migration:

- **Search the docs** -- Use the search bar at the top of this site.
- **Check the FAQ** -- Common questions are answered in the [FAQ](../faq.md).
- **Report issues** -- File a bug report at [GitHub Issues](https://github.com/dipankar/dscode/issues) with the `vscode-compat` label.
- **Community** -- Ask questions in [GitHub Discussions](https://github.com/dipankar/dscode/discussions).

---

## See Also

- [VS Code Compatibility Matrix](../reference/vscode-compatibility-matrix.md) -- detailed API coverage
- [Installing Extensions](../extensions/installing-extensions.md) -- extension management guide
- [Quick Start](quick-start.md) -- get productive with DSCode in five minutes
