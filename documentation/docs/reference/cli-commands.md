---
title: CLI Reference
description: Complete reference for the DSCode command-line interface -- all flags, subcommands, and environment variables.
---

# CLI Reference

DSCode provides a full-featured CLI for opening files, managing extensions, and controlling editor behavior from the terminal. After installation, the `dscode` binary is available on your `PATH`.

---

## Synopsis

```
dscode [options] [path...]
```

When invoked without arguments, DSCode opens the most recent workspace. When given one or more paths, it opens each file or folder in an editor window.

---

## Commands

### Opening Files and Folders

| Command | Description |
|---|---|
| `dscode` | Open DSCode (restores the last session) |
| `dscode <file>` | Open a specific file in the editor |
| `dscode <folder>` | Open a folder as the workspace root |
| `dscode <file1> <file2> ...` | Open multiple files in tabs |
| `dscode <folder1> <folder2> ...` | Open a multi-root workspace |

```bash
# Open the current directory
dscode .

# Open a specific file
dscode src/main.rs

# Open multiple files
dscode index.html style.css app.js

# Open a multi-root workspace
dscode ~/projects/frontend ~/projects/backend
```

---

### Window Behavior

| Flag | Short | Description |
|---|---|---|
| `--new-window` | `-n` | Force a new window instead of reusing an existing one |
| `--reuse-window` | `-r` | Reuse the most recently active window |
| `--wait` | `-w` | Wait for the file to be closed before returning (useful for `$EDITOR`) |
| `--locale <locale>` | | Set the display language (e.g., `en`, `de`, `ja`) |

```bash
# Always open in a new window
dscode --new-window ~/projects/my-app

# Reuse the existing window
dscode --reuse-window README.md

# Use DSCode as your git commit editor
git config --global core.editor "dscode --wait"
```

---

### File Navigation

| Flag | Description |
|---|---|
| `--goto <file>:<line>:<column>` | Open a file and place the cursor at the specified line and column |
| `--diff <file1> <file2>` | Open the diff editor comparing two files side by side |

```bash
# Open a file at line 42, column 10
dscode --goto src/main.rs:42:10

# Compare two files
dscode --diff old_config.json new_config.json
```

!!! tip "Shorthand goto syntax"
    You can also use the colon syntax directly without the `--goto` flag:

    ```bash
    dscode src/main.rs:42:10
    ```

    DSCode detects the `file:line:col` pattern automatically when a file path is given.

---

### Extension Management

| Flag | Description |
|---|---|
| `--install-extension <id-or-path>` | Install an extension by marketplace ID or `.vsix` file path |
| `--uninstall-extension <id>` | Uninstall an extension by its identifier |
| `--list-extensions` | List all installed extensions |
| `--show-versions` | Show extension versions when listing (use with `--list-extensions`) |
| `--force` | Skip confirmation prompts during install/uninstall |

```bash
# Install an extension from the marketplace
dscode --install-extension rust-lang.rust-analyzer

# Install from a local .vsix file
dscode --install-extension ~/downloads/my-extension-0.1.0.vsix

# Uninstall an extension
dscode --uninstall-extension rust-lang.rust-analyzer

# List all installed extensions with versions
dscode --list-extensions --show-versions
```

**Example output from `--list-extensions --show-versions`:**

```
esbenp.prettier-vscode@10.1.0
rust-lang.rust-analyzer@0.4.1894
svelte.svelte-vscode@108.3.1
```

---

### Informational Flags

| Flag | Short | Description |
|---|---|---|
| `--version` | `-v` | Print the DSCode version and exit |
| `--help` | `-h` | Show the help message with all available flags |
| `--status` | `-s` | Print process status and diagnostics |
| `--verbose` | | Enable verbose logging to the terminal |

```bash
# Print version information
dscode --version
# Output: DSCode 0.1.0 (Tauri 2.x, commit abc1234)

# Show detailed process status
dscode --status
```

**Example output from `--status`:**

```
DSCode 0.1.0

Process Information:
  PID:            12345
  Uptime:         2h 15m
  Memory (RSS):   48 MB
  CPU:            0.3%

Extension Host:
  Status:         Running
  Extensions:     12 loaded
  PID:            12346
  Memory (RSS):   85 MB

Workspaces:
  /home/user/projects/my-app (active)
```

---

### Advanced Options

| Flag | Description |
|---|---|
| `--extensionDevelopmentPath <path>` | Load an extension in development mode from the given path |
| `--inspect-extensions <port>` | Attach a Node.js debugger to the extension host on the specified port |
| `--disable-extensions` | Start DSCode without loading any extensions |
| `--disable-gpu` | Disable GPU hardware acceleration |
| `--log <level>` | Set the log level: `trace`, `debug`, `info`, `warn`, `error`, `off` |
| `--user-data-dir <path>` | Override the user data directory |
| `--extensions-dir <path>` | Override the extensions installation directory |
| `--prof-startup` | Profile startup performance and write results to the console |

```bash
# Develop and test an extension
dscode --extensionDevelopmentPath ~/projects/my-extension

# Debug the extension host
dscode --inspect-extensions 9229

# Run without extensions (for troubleshooting)
dscode --disable-extensions

# Use a custom data directory (useful for isolated testing)
dscode --user-data-dir /tmp/dscode-test
```

---

## Environment Variables

DSCode reads the following environment variables to configure behavior at startup. All variables are optional; DSCode uses sensible defaults when they are not set.

| Variable | Default | Description |
|---|---|---|
| `DSCODE_USER_DATA_DIR` | `~/.config/dscode` (Linux), `~/Library/Application Support/dscode` (macOS), `%APPDATA%\dscode` (Windows) | Override the user data directory for settings, keybindings, and state |
| `DSCODE_EXTENSIONS_DIR` | `<user-data-dir>/extensions` | Override the directory where extensions are installed |
| `DSCODE_LOG_LEVEL` | `info` | Set the default log level (`trace`, `debug`, `info`, `warn`, `error`, `off`) |
| `DSCODE_DEV` | (unset) | When set to `1`, enables development mode with additional debug output |
| `DSCODE_DISABLE_GPU` | (unset) | When set to `1`, disables GPU hardware acceleration |
| `DSCODE_NNG_TIMEOUT` | `5000` | Timeout in milliseconds for NNG IPC messages to the extension host |
| `DSCODE_FORCE_COLOR` | (unset) | When set to `1`, forces colored output in the terminal even without a TTY |
| `DSCODE_LOCALE` | System locale | Override the display language (e.g., `en`, `de`, `ja`, `zh-cn`) |
| `EDITOR` | (unset) | If set to `dscode --wait`, DSCode is used as the default editor for git and other CLI tools |
| `VISUAL` | (unset) | Same as `EDITOR`, preferred by some Unix tools |
| `HTTP_PROXY` | (unset) | HTTP proxy URL for marketplace and update requests |
| `HTTPS_PROXY` | (unset) | HTTPS proxy URL for marketplace and update requests |
| `NO_PROXY` | (unset) | Comma-separated list of hosts that bypass the proxy |
| `RUSTC_WRAPPER` | (unset) | Used during builds from source (e.g., `sccache` for faster recompilation) |

!!! tip "Setting environment variables permanently"

    === "macOS / Linux (bash/zsh)"

        Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

        ```bash
        export DSCODE_LOG_LEVEL="debug"
        export DSCODE_EXTENSIONS_DIR="$HOME/.dscode-extensions"
        ```

    === "Windows (PowerShell)"

        Set persistent environment variables:

        ```powershell
        [Environment]::SetEnvironmentVariable("DSCODE_LOG_LEVEL", "debug", "User")
        [Environment]::SetEnvironmentVariable("DSCODE_EXTENSIONS_DIR", "$env:USERPROFILE\.dscode-extensions", "User")
        ```

---

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | General error |
| `2` | Invalid command-line argument |
| `3` | Extension install/uninstall failed |
| `4` | File or folder not found |
| `5` | Permission denied |

---

## Shell Completion

DSCode can generate shell completion scripts for tab-completion support.

=== "Bash"

    ```bash
    dscode --generate-completion bash > ~/.local/share/bash-completion/completions/dscode
    ```

=== "Zsh"

    ```zsh
    dscode --generate-completion zsh > ~/.zfunc/_dscode
    # Add to .zshrc: fpath=(~/.zfunc $fpath)
    ```

=== "Fish"

    ```fish
    dscode --generate-completion fish > ~/.config/fish/completions/dscode.fish
    ```

=== "PowerShell"

    ```powershell
    dscode --generate-completion powershell >> $PROFILE
    ```

---

## Usage Examples

### Use DSCode as your default `$EDITOR`

```bash
export EDITOR="dscode --wait"
export VISUAL="dscode --wait"
```

This lets tools like `git commit`, `crontab -e`, and `kubectl edit` open DSCode and wait for you to close the file before continuing.

### Pipe content into DSCode

```bash
# Open stdin in a new editor tab
echo "Hello, DSCode" | dscode -

# Pipe command output into the editor
kubectl logs my-pod | dscode -
```

### Diff two branches of a file

```bash
dscode --diff <(git show main:config.json) <(git show feature:config.json)
```

### Profile startup performance

```bash
dscode --prof-startup
# Writes timing data to the console:
# [startup] Window ready: 68ms
# [startup] Extensions loaded: 142ms
# [startup] Workspace indexed: 210ms
```

---

## See Also

- [Installation](../getting-started/installation.md) -- installing DSCode and adding it to your `PATH`
- [Configuration Options](configuration-options.md) -- all `settings.json` options
- [Keyboard Shortcuts Reference](keyboard-shortcuts-reference.md) -- complete shortcut tables
