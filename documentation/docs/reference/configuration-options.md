---
title: Configuration Options
description: Complete reference for all DSCode settings -- editor, workbench, terminal, files, search, git, extensions, and telemetry.
---

# Configuration Options

DSCode stores its configuration in JSON files. Settings can be scoped to the user level or to individual workspaces.

| Scope | File Location |
|---|---|
| **User settings** | `~/.config/dscode/settings.json` (Linux), `~/Library/Application Support/dscode/settings.json` (macOS), `%APPDATA%\dscode\settings.json` (Windows) |
| **Workspace settings** | `<workspace>/.dscode/settings.json` |

Workspace settings override user settings. Open the settings editor with ++ctrl+comma++ (++cmd+comma++ on macOS) or edit the JSON file directly.

---

## Editor Settings

Core settings for the Monaco Editor component.

| Setting | Type | Default | Description |
|---|---|---|---|
| `editor.fontSize` | `number` | `14` | Font size in pixels for the editor |
| `editor.fontFamily` | `string` | `"JetBrains Mono, Menlo, Monaco, Consolas, monospace"` | Font family for the editor. The first available font is used |
| `editor.fontLigatures` | `boolean` | `false` | Enable font ligatures (e.g., `=>` rendered as an arrow in supported fonts) |
| `editor.fontWeight` | `string` | `"normal"` | Font weight: `"normal"`, `"bold"`, or a numeric value like `"300"` |
| `editor.tabSize` | `number` | `4` | Number of spaces a tab character occupies |
| `editor.insertSpaces` | `boolean` | `true` | Insert spaces when pressing ++tab++ instead of a tab character |
| `editor.detectIndentation` | `boolean` | `true` | Automatically detect indentation settings from file content |
| `editor.wordWrap` | `string` | `"off"` | Controls line wrapping. Values: `"off"`, `"on"`, `"wordWrapColumn"`, `"bounded"` |
| `editor.wordWrapColumn` | `number` | `80` | Column at which the editor wraps when `wordWrap` is `"wordWrapColumn"` or `"bounded"` |
| `editor.lineNumbers` | `string` | `"on"` | Controls line number display. Values: `"on"`, `"off"`, `"relative"`, `"interval"` |
| `editor.minimap.enabled` | `boolean` | `true` | Show the minimap (code overview) in the right gutter |
| `editor.minimap.maxColumn` | `number` | `120` | Maximum number of columns rendered in the minimap |
| `editor.minimap.renderCharacters` | `boolean` | `true` | Render actual characters in the minimap instead of color blocks |
| `editor.minimap.side` | `string` | `"right"` | Minimap position: `"left"` or `"right"` |
| `editor.cursorBlinking` | `string` | `"blink"` | Cursor animation style. Values: `"blink"`, `"smooth"`, `"phase"`, `"expand"`, `"solid"` |
| `editor.cursorStyle` | `string` | `"line"` | Cursor shape. Values: `"line"`, `"block"`, `"underline"`, `"line-thin"`, `"block-outline"`, `"underline-thin"` |
| `editor.cursorSmoothCaretAnimation` | `string` | `"off"` | Enable smooth caret animation. Values: `"off"`, `"explicit"`, `"on"` |
| `editor.renderWhitespace` | `string` | `"selection"` | When to render whitespace characters. Values: `"none"`, `"boundary"`, `"selection"`, `"trailing"`, `"all"` |
| `editor.renderControlCharacters` | `boolean` | `true` | Render control characters (e.g., null bytes) as visible symbols |
| `editor.renderLineHighlight` | `string` | `"line"` | Highlight the current line. Values: `"none"`, `"gutter"`, `"line"`, `"all"` |
| `editor.formatOnSave` | `boolean` | `false` | Automatically format the document when saving |
| `editor.formatOnPaste` | `boolean` | `false` | Automatically format pasted content |
| `editor.formatOnType` | `boolean` | `false` | Automatically format a line after typing a trigger character (e.g., `;`) |
| `editor.autoClosingBrackets` | `string` | `"languageDefined"` | Auto-close brackets. Values: `"always"`, `"languageDefined"`, `"beforeWhitespace"`, `"never"` |
| `editor.autoClosingQuotes` | `string` | `"languageDefined"` | Auto-close quotes. Same values as `autoClosingBrackets` |
| `editor.autoSurround` | `string` | `"languageDefined"` | Auto-surround selected text with brackets/quotes. Values: `"languageDefined"`, `"quotes"`, `"brackets"`, `"never"` |
| `editor.autoIndent` | `string` | `"full"` | Auto-indentation strategy. Values: `"none"`, `"keep"`, `"brackets"`, `"advanced"`, `"full"` |
| `editor.bracketPairColorization.enabled` | `boolean` | `true` | Colorize matching bracket pairs |
| `editor.guides.bracketPairs` | `boolean \| string` | `"active"` | Render bracket pair guides. Values: `true`, `false`, `"active"` |
| `editor.guides.indentation` | `boolean` | `true` | Render indentation guides |
| `editor.linkedEditing` | `boolean` | `false` | Enable linked editing (rename matching HTML tags simultaneously) |
| `editor.suggestSelection` | `string` | `"recentlyUsed"` | IntelliSense suggestion pre-selection. Values: `"first"`, `"recentlyUsed"`, `"recentlyUsedByPrefix"` |
| `editor.acceptSuggestionOnCommitCharacter` | `boolean` | `true` | Accept suggestions with commit characters (e.g., `.` or `;`) |
| `editor.acceptSuggestionOnEnter` | `string` | `"on"` | Accept suggestions with ++enter++. Values: `"on"`, `"smart"`, `"off"` |
| `editor.snippetSuggestions` | `string` | `"inline"` | Controls snippet suggestions. Values: `"top"`, `"bottom"`, `"inline"`, `"none"` |
| `editor.quickSuggestions` | `object` | `{"other": "on", "comments": "off", "strings": "off"}` | Enable quick suggestions (auto-completion without trigger character) per context |
| `editor.smoothScrolling` | `boolean` | `false` | Enable smooth scrolling in the editor |
| `editor.scrollBeyondLastLine` | `boolean` | `true` | Allow scrolling past the last line of the document |
| `editor.stickyScroll.enabled` | `boolean` | `true` | Show sticky scroll headers (nested scopes pinned to the top) |
| `editor.folding` | `boolean` | `true` | Enable code folding |
| `editor.foldingStrategy` | `string` | `"auto"` | Folding strategy. Values: `"auto"` (language-based or indentation), `"indentation"` |
| `editor.showFoldingControls` | `string` | `"mouseover"` | When to show folding controls. Values: `"always"`, `"never"`, `"mouseover"` |
| `editor.dragAndDrop` | `boolean` | `true` | Enable drag-and-drop to move text selections |
| `editor.columnSelection` | `boolean` | `false` | Enable column (box) selection mode by default |

---

## Workbench Settings

Settings that control the overall UI layout and appearance.

| Setting | Type | Default | Description |
|---|---|---|---|
| `workbench.colorTheme` | `string` | `"Default Dark Modern"` | The active color theme identifier |
| `workbench.iconTheme` | `string` | `"seti"` | The active file icon theme |
| `workbench.startupEditor` | `string` | `"welcomePage"` | What to show on startup. Values: `"none"`, `"welcomePage"`, `"readme"`, `"newUntitledFile"`, `"welcomePageInEmptyWorkbench"` |
| `workbench.sideBar.location` | `string` | `"left"` | Side bar position. Values: `"left"`, `"right"` |
| `workbench.activityBar.visible` | `boolean` | `true` | Show the activity bar (icon strip on the far left/right) |
| `workbench.statusBar.visible` | `boolean` | `true` | Show the status bar at the bottom of the window |
| `workbench.editor.showTabs` | `string` | `"multiple"` | Tab display mode. Values: `"multiple"`, `"single"`, `"none"` |
| `workbench.editor.tabCloseButton` | `string` | `"right"` | Position of the close button on tabs. Values: `"left"`, `"right"`, `"off"` |
| `workbench.editor.labelFormat` | `string` | `"default"` | Tab label format. Values: `"default"`, `"short"`, `"medium"`, `"long"` |
| `workbench.editor.enablePreview` | `boolean` | `true` | Single-click opens files in preview mode (italic tab title) |
| `workbench.editor.enablePreviewFromQuickOpen` | `boolean` | `false` | Open files from Quick Open in preview mode |
| `workbench.tree.indent` | `number` | `8` | Indentation in pixels for tree views (explorer, outline) |
| `workbench.tree.renderIndentGuides` | `string` | `"onHover"` | When to render tree indent guides. Values: `"none"`, `"onHover"`, `"always"` |
| `workbench.colorCustomizations` | `object` | `{}` | Override theme colors. Keys are color identifiers; values are hex colors |
| `workbench.panel.defaultLocation` | `string` | `"bottom"` | Default panel location. Values: `"bottom"`, `"right"`, `"left"` |

---

## Terminal Settings

Settings for the integrated terminal (xterm.js + portable-pty).

| Setting | Type | Default | Description |
|---|---|---|---|
| `terminal.integrated.fontSize` | `number` | `14` | Font size in pixels for the terminal |
| `terminal.integrated.fontFamily` | `string` | `"JetBrains Mono, Menlo, Monaco, Consolas, monospace"` | Font family for the terminal |
| `terminal.integrated.fontWeight` | `string` | `"normal"` | Font weight for the terminal |
| `terminal.integrated.lineHeight` | `number` | `1.0` | Line height multiplier for the terminal |
| `terminal.integrated.letterSpacing` | `number` | `0` | Letter spacing in pixels |
| `terminal.integrated.cursorBlinking` | `boolean` | `false` | Enable cursor blinking in the terminal |
| `terminal.integrated.cursorStyle` | `string` | `"block"` | Terminal cursor style. Values: `"block"`, `"underline"`, `"bar"` |
| `terminal.integrated.scrollback` | `number` | `1000` | Maximum number of lines retained in the terminal scrollback buffer |
| `terminal.integrated.shell.osx` | `string` | `"/bin/zsh"` | Default shell on macOS |
| `terminal.integrated.shell.linux` | `string` | `"/bin/bash"` | Default shell on Linux |
| `terminal.integrated.shell.windows` | `string` | `"C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"` | Default shell on Windows |
| `terminal.integrated.env.osx` | `object` | `{}` | Additional environment variables for macOS terminals |
| `terminal.integrated.env.linux` | `object` | `{}` | Additional environment variables for Linux terminals |
| `terminal.integrated.env.windows` | `object` | `{}` | Additional environment variables for Windows terminals |
| `terminal.integrated.cwd` | `string` | `""` | Working directory for new terminal instances (empty = workspace root) |
| `terminal.integrated.confirmOnExit` | `string` | `"never"` | Confirm before closing a terminal with running processes. Values: `"never"`, `"always"`, `"hasChildProcesses"` |
| `terminal.integrated.copyOnSelection` | `boolean` | `false` | Copy selected text to the clipboard automatically |
| `terminal.integrated.rightClickBehavior` | `string` | `"default"` | Right-click behavior. Values: `"default"`, `"copyPaste"`, `"paste"`, `"selectWord"` |

---

## Files Settings

Settings that control how DSCode handles files, encoding, and auto-save.

| Setting | Type | Default | Description |
|---|---|---|---|
| `files.autoSave` | `string` | `"off"` | Auto-save strategy. Values: `"off"`, `"afterDelay"`, `"onFocusChange"`, `"onWindowChange"` |
| `files.autoSaveDelay` | `number` | `1000` | Delay in milliseconds before auto-saving (when `autoSave` is `"afterDelay"`) |
| `files.exclude` | `object` | `{"**/.git": true, "**/.DS_Store": true, "**/node_modules": true, "**/target": true}` | Glob patterns for files and folders to exclude from the explorer and search |
| `files.associations` | `object` | `{}` | Map file extensions to language identifiers (e.g., `{"*.mdx": "markdown"}`) |
| `files.encoding` | `string` | `"utf8"` | Default file encoding. Values: `"utf8"`, `"utf8bom"`, `"utf16le"`, `"utf16be"`, `"ascii"`, `"iso88591"`, etc. |
| `files.eol` | `string` | `"auto"` | Default end-of-line character. Values: `"auto"`, `"\n"` (LF), `"\r\n"` (CRLF) |
| `files.trimTrailingWhitespace` | `boolean` | `false` | Trim trailing whitespace on save |
| `files.insertFinalNewline` | `boolean` | `false` | Insert a final newline at the end of the file on save |
| `files.trimFinalNewlines` | `boolean` | `false` | Trim all final newlines after the last line on save |
| `files.watcherExclude` | `object` | `{"**/.git/objects/**": true, "**/node_modules/**": true, "**/target/**": true}` | Glob patterns for files to exclude from the file watcher (reduces CPU usage) |
| `files.hotExit` | `string` | `"onExit"` | Restore unsaved files after exit. Values: `"off"`, `"onExit"`, `"onExitAndWindowClose"` |
| `files.defaultLanguage` | `string` | `""` | Default language mode for new untitled files (e.g., `"typescript"`) |
| `files.maxMemoryForLargeFilesMB` | `number` | `4096` | Maximum memory in MB allowed for large file operations |

---

## Search Settings

Settings for the file search and find-in-files features (powered by ripgrep).

| Setting | Type | Default | Description |
|---|---|---|---|
| `search.exclude` | `object` | `{"**/node_modules": true, "**/bower_components": true, "**/*.code-search": true, "**/target": true}` | Glob patterns to exclude from search results |
| `search.useIgnoreFiles` | `boolean` | `true` | Respect `.gitignore` and `.ignore` files when searching |
| `search.useGlobalIgnoreFiles` | `boolean` | `false` | Respect global `.gitignore` files |
| `search.followSymlinks` | `boolean` | `true` | Follow symbolic links when searching |
| `search.smartCase` | `boolean` | `false` | Case-sensitive search only when the pattern contains uppercase characters |
| `search.showLineNumbers` | `boolean` | `false` | Show line numbers in search results |
| `search.defaultViewMode` | `string` | `"list"` | Default search results view. Values: `"list"`, `"tree"` |
| `search.seedOnFocus` | `boolean` | `false` | Populate the search field with the selected text when opening find-in-files |
| `search.searchOnType` | `boolean` | `true` | Search as you type instead of waiting for ++enter++ |
| `search.searchOnTypeDebouncePeriod` | `number` | `300` | Debounce delay in milliseconds for search-on-type |
| `search.collapseResults` | `string` | `"alwaysExpand"` | How to display search result file groups. Values: `"auto"`, `"alwaysCollapse"`, `"alwaysExpand"` |

---

## Git Settings

Settings for the built-in Git integration (powered by the `git2` crate).

| Setting | Type | Default | Description |
|---|---|---|---|
| `git.enabled` | `boolean` | `true` | Enable Git features |
| `git.path` | `string \| null` | `null` | Path to the Git executable (DSCode uses libgit2 internally, but some operations can fall back to the CLI) |
| `git.autofetch` | `boolean \| string` | `false` | Auto-fetch from remotes. Values: `false`, `true`, `"all"` |
| `git.autofetchPeriod` | `number` | `180` | Auto-fetch interval in seconds |
| `git.confirmSync` | `boolean` | `true` | Ask for confirmation before synchronizing (push/pull) |
| `git.enableSmartCommit` | `boolean` | `false` | Commit all changes when there are no staged changes (skip the staging step) |
| `git.enableCommitSigning` | `boolean` | `false` | Sign commits with GPG |
| `git.showInlineOpenFileAction` | `boolean` | `true` | Show the "Open File" action inline in the SCM diff view |
| `git.inputValidationLength` | `number` | `72` | Character limit for the first line of commit messages (shows a warning when exceeded) |
| `git.inputValidationSubjectLength` | `number \| null` | `50` | Soft limit for the commit subject line length |
| `git.decorations.enabled` | `boolean` | `true` | Show git status decorations (colors and badges) in the explorer |
| `git.defaultCloneDirectory` | `string \| null` | `null` | Default parent directory for `git clone` operations |
| `git.branchSortOrder` | `string` | `"committerdate"` | How to sort branches. Values: `"committerdate"`, `"alphabetically"` |
| `git.pruneOnFetch` | `boolean` | `false` | Prune remote-tracking branches on fetch |

---

## Extension Settings

Settings for the extension system and marketplace.

| Setting | Type | Default | Description |
|---|---|---|---|
| `extensions.autoUpdate` | `boolean \| string` | `true` | Automatically update extensions. Values: `true`, `false`, `"onlyEnabledExtensions"` |
| `extensions.autoCheckUpdates` | `boolean` | `true` | Check for extension updates periodically |
| `extensions.ignoreRecommendations` | `boolean` | `false` | Suppress extension recommendations |
| `extensions.closeExtensionDetailsOnViewChange` | `boolean` | `false` | Close the extension details panel when switching views |
| `extensions.confirmedUriHandlerExtensionIds` | `string[]` | `[]` | Extension IDs allowed to handle URIs without prompting |
| `extensions.supportUntrustedWorkspaces` | `object` | `{}` | Per-extension trust settings for restricted mode |

---

## Telemetry Settings

| Setting | Type | Default | Description |
|---|---|---|---|
| `telemetry.telemetryLevel` | `string` | `"all"` | Controls what telemetry data is collected. Values: `"off"`, `"crash"`, `"error"`, `"all"` |

!!! info "Privacy by default"
    DSCode collects minimal, anonymous usage telemetry to improve the editor. No code, file contents, or file names are ever transmitted. Set `telemetry.telemetryLevel` to `"off"` to disable all data collection.

---

## Example Settings File

A complete example `settings.json` that demonstrates many common customizations:

```json title="settings.json"
{
    // Editor
    "editor.fontSize": 15,
    "editor.fontFamily": "Fira Code, JetBrains Mono, monospace",
    "editor.fontLigatures": true,
    "editor.tabSize": 2,
    "editor.insertSpaces": true,
    "editor.wordWrap": "on",
    "editor.lineNumbers": "relative",
    "editor.minimap.enabled": false,
    "editor.cursorBlinking": "smooth",
    "editor.renderWhitespace": "trailing",
    "editor.formatOnSave": true,
    "editor.formatOnPaste": true,
    "editor.bracketPairColorization.enabled": true,
    "editor.stickyScroll.enabled": true,
    "editor.guides.bracketPairs": "active",

    // Workbench
    "workbench.colorTheme": "One Dark Pro",
    "workbench.iconTheme": "material-icon-theme",
    "workbench.startupEditor": "none",
    "workbench.sideBar.location": "left",
    "workbench.activityBar.visible": true,

    // Terminal
    "terminal.integrated.fontSize": 13,
    "terminal.integrated.fontFamily": "JetBrains Mono",
    "terminal.integrated.cursorBlinking": true,
    "terminal.integrated.scrollback": 5000,

    // Files
    "files.autoSave": "afterDelay",
    "files.autoSaveDelay": 500,
    "files.trimTrailingWhitespace": true,
    "files.insertFinalNewline": true,
    "files.exclude": {
        "**/.git": true,
        "**/.DS_Store": true,
        "**/node_modules": true,
        "**/target": true,
        "**/__pycache__": true
    },

    // Search
    "search.exclude": {
        "**/node_modules": true,
        "**/dist": true,
        "**/target": true
    },
    "search.useIgnoreFiles": true,
    "search.smartCase": true,

    // Git
    "git.enabled": true,
    "git.autofetch": true,
    "git.confirmSync": false,
    "git.enableSmartCommit": true,

    // Extensions
    "extensions.autoUpdate": true,

    // Telemetry
    "telemetry.telemetryLevel": "error"
}
```

---

## Language-Specific Settings

You can override any editor setting for a specific language by wrapping it in a language identifier block:

```json title="settings.json"
{
    "editor.tabSize": 4,

    "[python]": {
        "editor.tabSize": 4,
        "editor.insertSpaces": true,
        "editor.formatOnSave": true
    },

    "[javascript]": {
        "editor.tabSize": 2,
        "editor.formatOnSave": true
    },

    "[rust]": {
        "editor.tabSize": 4,
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    },

    "[markdown]": {
        "editor.wordWrap": "on",
        "editor.quickSuggestions": {
            "other": "off",
            "comments": "off",
            "strings": "off"
        }
    }
}
```

---

## See Also

- [Settings and Configuration](../user-guide/settings-and-configuration.md) -- user guide for working with settings
- [Themes and Appearance](../user-guide/themes-and-appearance.md) -- customizing the visual appearance
- [CLI Reference](cli-commands.md) -- command-line flags that override settings
