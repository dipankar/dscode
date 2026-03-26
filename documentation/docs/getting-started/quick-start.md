# Quick Start

Get productive with DSCode in under five minutes. This guide walks you through
the core workflows you will use every day.

---

## Launching DSCode

After [installing DSCode](installation.md), launch it from your terminal:

```bash
dscode .
```

Or open the application directly from your system launcher (Applications on
macOS, application menu on Linux, Start Menu on Windows). If you are building
from source, use `cargo tauri dev` to launch the development build.

When DSCode opens for the first time, you will see the **Welcome** tab with
links to documentation, recent projects, and configuration options.

![Welcome Screen](../assets/screenshots/welcome-screen.png){ .screenshot }

---

## Opening a Folder

DSCode is project-oriented -- most features work best when you have a folder
open.

=== "Menu"

    Go to **File > Open Folder...** and select your project directory.

=== "Keyboard"

    Press ++ctrl+o++ (or ++cmd+o++ on macOS) to open the folder picker.

=== "Terminal"

    Launch DSCode with a folder argument:

    ```bash
    dscode /path/to/your/project
    ```

Once a folder is open, the **Explorer** sidebar displays the file tree on the
left.

![Explorer Sidebar](../assets/screenshots/explorer-sidebar.png){ .screenshot }

---

## Creating and Editing Files

### Create a New File

1. Hover over the Explorer sidebar and click the **New File** icon, or
2. Press ++ctrl+n++ (++cmd+n++ on macOS) to create an untitled file.

### Basic Editing

DSCode uses the **Monaco Editor** -- the same editing engine behind VS Code.
Everything you already know works here:

- **Undo / Redo** -- ++ctrl+z++ / ++ctrl+shift+z++ (++cmd+z++ / ++cmd+shift+z++
  on macOS)
- **Save** -- ++ctrl+s++ (++cmd+s++ on macOS)
- **Find** -- ++ctrl+f++ (++cmd+f++ on macOS)
- **Find and Replace** -- ++ctrl+h++ (++cmd+h++ on macOS)

!!! tip "For VS Code users"
    If you are coming from VS Code, your muscle memory transfers directly.
    DSCode mirrors the default VS Code keybindings, so shortcuts like
    ++ctrl+d++ (select next occurrence) and ++ctrl+shift+l++ (select all
    occurrences) work the same way.

---

## The Command Palette

The **Command Palette** is the fastest way to access any command in DSCode.

Open it with ++ctrl+shift+p++ (or ++cmd+shift+p++ on macOS), then start
typing to filter the list:

![Command Palette](../assets/screenshots/command-palette.png){ .screenshot }

Try these commands to get started:

| Type this...             | What it does                      |
|--------------------------|-----------------------------------|
| `theme`                  | Change the color theme            |
| `settings`               | Open user settings                |
| `terminal`               | Toggle the integrated terminal    |
| `format document`        | Auto-format the current file      |
| `toggle word wrap`       | Turn line wrapping on or off      |

!!! tip "Quick Open vs. Command Palette"
    Press ++ctrl+p++ (++cmd+p++ on macOS) to open **Quick Open**, which
    searches for files by name instead of commands. Use this when you know
    the filename you want to jump to.

---

## The Integrated Terminal

Open the built-in terminal with ++ctrl+grave++ (++cmd+grave++ on macOS), or
through the Command Palette by typing `Toggle Terminal`.

![Integrated Terminal](../assets/screenshots/integrated-terminal.png){ .screenshot }

The terminal opens at the root of your current workspace and supports:

- Multiple terminal instances (click the **+** icon)
- Split terminals (click the split icon or press ++ctrl+shift+5++)
- Shell selection (bash, zsh, PowerShell, etc.)

!!! tip "Running build commands"
    The integrated terminal is perfect for running build tools, test suites,
    and dev servers without leaving the editor. Try `npm run dev`, `cargo
    build`, or `python app.py` right inside DSCode.

---

## Installing an Extension

Extensions add language support, themes, debuggers, and more.

1. Open the **Extensions** sidebar by clicking the extensions icon in the
   activity bar, or press ++ctrl+shift+x++ (++cmd+shift+x++ on macOS).
2. Search for an extension (for example, `Python` or `Rust Analyzer`).
3. Click **Install**.

![Extensions Sidebar](../assets/screenshots/extensions-sidebar.png){ .screenshot }

DSCode supports a wide range of VS Code-compatible extensions. See
[Installing Extensions](../extensions/installing-extensions.md) for more
details.

---

## Essential Keyboard Shortcuts

These shortcuts will cover most of your daily editing needs:

### General

| Action                  | Windows / Linux           | macOS                     |
|-------------------------|---------------------------|---------------------------|
| Command Palette         | ++ctrl+shift+p++          | ++cmd+shift+p++           |
| Quick Open (file)       | ++ctrl+p++                | ++cmd+p++                 |
| User Settings           | ++ctrl+comma++            | ++cmd+comma++             |
| Toggle Sidebar          | ++ctrl+b++                | ++cmd+b++                 |
| Toggle Terminal          | ++ctrl+grave++            | ++cmd+grave++             |
| New Window              | ++ctrl+shift+n++          | ++cmd+shift+n++           |

### Editing

| Action                  | Windows / Linux           | macOS                     |
|-------------------------|---------------------------|---------------------------|
| Cut line                | ++ctrl+x++                | ++cmd+x++                 |
| Copy line               | ++ctrl+c++                | ++cmd+c++                 |
| Move line up/down       | ++alt+up++ / ++alt+down++ | ++option+up++ / ++option+down++ |
| Duplicate line          | ++shift+alt+down++        | ++shift+option+down++     |
| Delete line             | ++ctrl+shift+k++          | ++cmd+shift+k++           |
| Toggle comment          | ++ctrl+slash++            | ++cmd+slash++             |
| Multi-cursor            | ++alt+click++             | ++option+click++          |
| Select next occurrence  | ++ctrl+d++                | ++cmd+d++                 |

### Navigation

| Action                  | Windows / Linux           | macOS                     |
|-------------------------|---------------------------|---------------------------|
| Go to line              | ++ctrl+g++                | ++cmd+g++                 |
| Go to symbol            | ++ctrl+shift+o++          | ++cmd+shift+o++           |
| Go to definition        | ++f12++                   | ++f12++                   |
| Peek definition         | ++alt+f12++               | ++option+f12++            |
| Navigate back           | ++alt+left++              | ++ctrl+minus++            |
| Navigate forward        | ++alt+right++             | ++ctrl+shift+minus++      |

!!! tip "Printable shortcut reference"
    For a comprehensive list, see the
    [Keyboard Shortcuts Reference](../reference/keyboard-shortcuts-reference.md)
    or press ++ctrl+k++ then ++ctrl+s++ (++cmd+k++ then ++cmd+s++ on macOS)
    to open the interactive shortcut editor.

---

---

## Source Control Quick Start

DSCode has built-in Git support. Here is how to stage, commit, and diff in
three steps:

1. **Open the Source Control view** -- Press ++ctrl+shift+g++ (++cmd+shift+g++
   on macOS) or click the branch icon in the Activity Bar.
2. **Stage changes** -- Click the `+` icon next to changed files, or click
   `+` on the section header to stage all changes.
3. **Commit** -- Type a commit message in the input box at the top of the
   SCM view and press ++ctrl+enter++ (++cmd+enter++ on macOS).

Click any changed file in the SCM view to open a side-by-side diff showing
exactly what has changed.

!!! tip
    DSCode's Git is powered by the `git2` Rust crate -- no external `git`
    binary is required. See the full
    [Git Integration](../user-guide/git-integration.md) guide for details.

---

## Language Intelligence Quick Start

When you open a file in a supported language, DSCode automatically starts
the appropriate language server and provides intelligent features:

- **Hover** -- Hold your cursor over a symbol to see its type, documentation,
  and function signature.
- **Auto-completion** -- Start typing to see context-aware suggestions.
  Press ++ctrl+space++ (++cmd+space++ on macOS) to trigger completions
  manually.
- **Go to Definition** -- Press ++f12++ on a symbol to jump to its
  definition. Press ++alt+f12++ (++option+f12++ on macOS) to peek at the
  definition inline.

DSCode ships with 4 built-in language servers (Python, Rust, Go, JSON).
Install additional language extensions from the marketplace for other
languages.

---

## Customization Quick Start

Make DSCode yours in under a minute:

- **Change theme** -- Open the Command Palette (++ctrl+shift+p++) and type
  `Color Theme`, then select from the list. Dark and light themes are
  available out of the box, and you can install more from the marketplace.
- **Adjust font size** -- Open Settings (++ctrl+comma++) and search for
  `editor.fontSize`. Set it to your preferred size.
- **Customize keybindings** -- Open the Command Palette and type
  `Keyboard Shortcuts` to browse and modify keybindings, or edit
  `keybindings.json` directly.

!!! note "For VS Code users"
    DSCode reads the same `settings.json` and `keybindings.json` formats as
    VS Code. You can copy your existing configuration files to
    `~/.config/dscode/` (Linux), `~/Library/Application Support/dscode/`
    (macOS), or `%APPDATA%\dscode\` (Windows) to get started immediately.

    Key differences from VS Code:

    - DSCode uses the **Open VSX** marketplace instead of the Microsoft
      marketplace. Most popular extensions are available on both.
    - Remote development (SSH, WSL, containers) is **not yet available**.
    - Some VS Code-proprietary APIs are not implemented (~75% API coverage).
    - DSCode uses significantly less memory (~50 MB vs. 300-500 MB) and
      starts faster (<100ms vs. 1-3 seconds).

    See [Migrating from VS Code](migrating-from-vscode.md) for a complete
    migration walkthrough.

---

## Next Steps

Now that you know the basics, try building something! Head to
[Your First Project](first-project.md) for a hands-on tutorial, or explore
the [User Guide](../user-guide/editor-features.md) for a deeper look at
DSCode's features.
