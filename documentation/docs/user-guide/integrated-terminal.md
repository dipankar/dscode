# Integrated Terminal

DSCode includes a fully featured integrated terminal so you can run shell commands, build scripts, and interactive programs without leaving the editor. The terminal is rendered by [xterm.js](https://xtermjs.org/) on the frontend (`Terminal.svelte`) and backed by a native pseudo-terminal (PTY) managed by the Rust `TerminalManager` through `terminal_ops` commands.

---

## Terminal Overview

The integrated terminal stack consists of two layers:

- **Frontend** -- `Terminal.svelte` wraps an xterm.js instance with add-ons for fit, search, and Unicode support. User keystrokes are sent to the backend via Tauri IPC (`write_to_terminal`), and output from the PTY is streamed back as events.
- **Backend** -- The Rust `TerminalManager` creates and manages PTY processes using the `portable-pty` crate. Each terminal instance runs a real shell process (bash, zsh, PowerShell, etc.) with full TTY semantics, including signal handling and job control.

---

## Opening the Terminal

| Action | Shortcut |
|---|---|
| Toggle terminal panel | ++ctrl+grave++ |
| Create new terminal | ++ctrl+shift+grave++ |

You can also open a terminal from:

- The **Command Palette** (++ctrl+shift+p++) -- type `Terminal: Create New Terminal`
- The **Terminal** menu in the menu bar
- The **Panel Area** tab bar -- click the `+` button

!!! note
    The ++ctrl+grave++ shortcut uses the backtick key, also known as the grave accent key, located below ++escape++ on most keyboards.

---

## Multiple Terminals

DSCode supports multiple concurrent terminal instances. Each terminal runs in its own PTY process and can use a different shell.

### Managing Terminals

- **Create a new terminal**: ++ctrl+shift+grave++ or click the `+` icon in the terminal tab bar.
- **Switch between terminals**: Click a terminal tab, or use the dropdown list in the terminal panel header.
- **Rename a terminal**: Right-click the terminal tab and select **Rename**.
- **Close a terminal**: Click the trash icon on the terminal tab, or type `exit` in the shell.

The backend `list_terminals` command returns all active terminal sessions:

```rust
pub fn list_terminals(
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<Vec<TerminalInfo>, String>
```

!!! tip
    Use descriptive names for your terminals (e.g., "dev server", "build", "tests") to quickly identify them when you have several open.

---

## Split Terminals

You can split a terminal horizontally to view two terminals side by side within the same panel.

- **Split terminal**: Click the split icon in the terminal panel header, or use the Command Palette and type `Terminal: Split Terminal`.
- **Navigate between split panes**: ++alt+left++ and ++alt+right++.
- **Resize panes**: Drag the divider between the split terminals.

---

## Terminal Profiles

Terminal profiles define which shell to use and how to launch it. DSCode auto-detects available shells on your system.

### Default Profiles

=== "macOS"

    - `/bin/zsh` (default)
    - `/bin/bash`
    - `/bin/sh`

=== "Linux"

    - `/bin/bash` (default)
    - `/bin/zsh`
    - `/bin/sh`

=== "Windows"

    - `PowerShell` (default)
    - `Command Prompt` (cmd.exe)
    - `Git Bash` (if installed)
    - `WSL` (if installed)

### Configuring Profiles

You can configure terminal profiles in your settings:

```json
{
  "terminal.integrated.profiles.linux": {
    "bash": {
      "path": "/bin/bash",
      "args": ["-l"]
    },
    "zsh": {
      "path": "/bin/zsh"
    },
    "fish": {
      "path": "/usr/bin/fish"
    }
  },
  "terminal.integrated.defaultProfile.linux": "zsh"
}
```

---

## Shell Selection

When creating a new terminal, you can choose which shell to use:

1. Open the Command Palette with ++ctrl+shift+p++.
2. Type `Terminal: Select Default Profile`.
3. Choose from the list of detected shells.

Alternatively, use the dropdown arrow next to the `+` button in the terminal panel to create a terminal with a specific shell without changing the default.

The Rust backend accepts a shell parameter when creating terminals:

```rust
pub fn create_terminal(
    name: Option<String>,
    shell: Option<String>,
    cwd: Option<String>,
    app_handle: AppHandle,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<String, String>
```

---

## Copy and Paste

Terminal copy and paste uses platform-standard shortcuts:

| Action | macOS | Linux / Windows |
|---|---|---|
| Copy | ++cmd+c++ | ++ctrl+shift+c++ |
| Paste | ++cmd+v++ | ++ctrl+shift+v++ |

!!! note
    On Linux and Windows, the terminal uses ++ctrl+shift+c++ and ++ctrl+shift+v++ (rather than ++ctrl+c++ and ++ctrl+v++) because ++ctrl+c++ sends the `SIGINT` signal to the running process.

Text selection in the terminal works as follows:

- **Select text**: Click and drag, or double-click to select a word.
- **Select a line**: Triple-click.
- **Select all**: No default shortcut; use the right-click context menu.

---

## Terminal Customization

### Font and Appearance

Configure the terminal's visual appearance in settings:

```json
{
  "terminal.integrated.fontFamily": "JetBrains Mono",
  "terminal.integrated.fontSize": 14,
  "terminal.integrated.lineHeight": 1.2,
  "terminal.integrated.cursorStyle": "block",
  "terminal.integrated.cursorBlinking": true
}
```

### Cursor Styles

Available cursor styles:

- `"block"` -- solid rectangle (default)
- `"underline"` -- horizontal line below the character
- `"bar"` -- vertical line to the left of the character

### Scrollback

Control how many lines the terminal keeps in its scroll buffer:

```json
{
  "terminal.integrated.scrollback": 10000
}
```

### Terminal Colors

Terminal colors follow your active color theme. You can override individual ANSI colors in your theme or settings:

```json
{
  "workbench.colorCustomizations": {
    "terminal.background": "#1e1e2e",
    "terminal.foreground": "#cdd6f4",
    "terminalCursor.foreground": "#f5e0dc"
  }
}
```

---

## Working Directory

By default, new terminals open with the working directory set to your workspace root. You can customize this behavior:

```json
{
  "terminal.integrated.cwd": "/path/to/custom/directory"
}
```

### Per-Terminal Working Directory

When creating a terminal programmatically (or via the `create_terminal` Tauri command), you can specify a custom working directory through the `cwd` parameter.

!!! tip
    Right-click a folder in the Explorer and select **Open in Integrated Terminal** to launch a terminal pre-set to that folder's path.

---

## Terminal Resize

The terminal automatically resizes to fit the panel dimensions. When you resize the panel or the window, the backend is notified via the `resize_terminal` command:

```rust
pub fn resize_terminal(
    terminal_id: String,
    cols: u16,
    rows: u16,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String>
```

The xterm.js `fit` add-on calculates the correct number of columns and rows based on the current panel size and font metrics.

---

## Terminal Shortcuts Reference

| Action | Shortcut |
|---|---|
| Toggle terminal | ++ctrl+grave++ |
| New terminal | ++ctrl+shift+grave++ |
| Copy (Linux/Windows) | ++ctrl+shift+c++ |
| Paste (Linux/Windows) | ++ctrl+shift+v++ |
| Copy (macOS) | ++cmd+c++ |
| Paste (macOS) | ++cmd+v++ |
| Clear terminal | ++ctrl+k++ (in terminal) |
| Kill terminal | Click the trash icon |
| Switch terminal | Terminal dropdown |
| Split terminal | Command Palette or split icon |
| Navigate split panes | ++alt+left++ / ++alt+right++ |

!!! warning
    If the terminal does not respond to input, the shell process may have crashed. Check the terminal tab for an error indicator and create a new terminal if needed. You can also check the DSCode developer console (++ctrl+shift+i++) for error details.
