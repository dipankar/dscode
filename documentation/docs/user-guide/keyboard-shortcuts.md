# Keyboard Shortcuts

DSCode features a comprehensive keyboard shortcut system managed by the Rust `KeybindingRegistry`. Every command in DSCode -- from built-in editor actions to extension-contributed commands -- can be bound to a keyboard shortcut. The system supports conditional activation through "when clauses," multi-key chord sequences, and platform-specific bindings.

---

## Customizing Shortcuts

### Keyboard Shortcuts Editor

Open the graphical Keyboard Shortcuts editor:

- Press ++ctrl+k++ ++ctrl+s++
- Or open the Command Palette (++ctrl+shift+p++) and type `Preferences: Open Keyboard Shortcuts`

The editor displays a searchable table of all keybindings with columns for:

- **Command** -- the command identifier and description
- **Keybinding** -- the current shortcut
- **When** -- the condition under which the shortcut is active
- **Source** -- whether the binding is a default, user override, or extension contribution

### Modifying a Shortcut

1. Find the command in the Keyboard Shortcuts editor.
2. Double-click the keybinding cell (or click the pencil icon).
3. Press the desired key combination.
4. Press ++enter++ to confirm.

### Removing a Shortcut

Right-click a keybinding and select **Remove Keybinding**, or click the minus icon.

---

## keybindings.json

For full control, edit the `keybindings.json` file directly.

### Opening keybindings.json

- Press ++ctrl+k++ ++ctrl+s++ to open the Keyboard Shortcuts editor, then click the file icon in the top-right corner to open the JSON file.
- Or use the Command Palette: `Preferences: Open Keyboard Shortcuts (JSON)`.

### File Format

The file contains a JSON array of keybinding objects:

```json
[
  {
    "key": "ctrl+shift+n",
    "command": "workbench.action.newWindow"
  },
  {
    "key": "ctrl+k ctrl+c",
    "command": "editor.action.addCommentLine",
    "when": "editorTextFocus"
  },
  {
    "key": "ctrl+shift+f",
    "command": "-workbench.action.findInFiles"
  }
]
```

### Keybinding Structure

Each keybinding corresponds to the Rust `Keybinding` struct:

```rust
pub struct Keybinding {
    pub command: String,
    pub key: String,
    pub when: Option<String>,
    pub platform: Option<String>,
    pub owner: String,
    pub args: Option<serde_json::Value>,
}
```

| Field | Required | Description |
|---|---|---|
| `key` | Yes | The key combination (e.g., `"ctrl+shift+p"`) |
| `command` | Yes | The command to execute |
| `when` | No | Condition for when this binding is active |
| `args` | No | Arguments to pass to the command |

!!! tip
    Prefix a command with `-` to remove a default keybinding. For example, `"-workbench.action.findInFiles"` unbinds the default ++ctrl+shift+f++ shortcut.

---

## Default Shortcuts Overview

Below is a selection of frequently used default shortcuts. For the full reference, see the [Keyboard Shortcuts Reference](../reference/keyboard-shortcuts-reference.md).

### General

| Action | Shortcut |
|---|---|
| Command Palette | ++ctrl+shift+p++ |
| Quick Open | ++ctrl+p++ |
| New Window | ++ctrl+shift+n++ |
| Close Window | ++ctrl+shift+w++ |
| User Settings | ++ctrl+comma++ |
| Keyboard Shortcuts | ++ctrl+k++ ++ctrl+s++ |

### Editing

| Action | Shortcut |
|---|---|
| Cut line | ++ctrl+x++ |
| Copy line | ++ctrl+c++ |
| Paste | ++ctrl+v++ |
| Undo | ++ctrl+z++ |
| Redo | ++ctrl+shift+z++ |
| Move line up | ++alt+up++ |
| Move line down | ++alt+down++ |
| Duplicate line | ++shift+alt+down++ |
| Delete line | ++ctrl+shift+k++ |
| Toggle comment | ++ctrl+slash++ |
| Toggle block comment | ++ctrl+shift+a++ |
| Indent line | ++ctrl+bracket-right++ |
| Outdent line | ++ctrl+bracket-left++ |

### Multi-Cursor

| Action | Shortcut |
|---|---|
| Add cursor | ++alt+click++ |
| Add cursor above | ++ctrl+alt+up++ |
| Add cursor below | ++ctrl+alt+down++ |
| Select all occurrences | ++ctrl+shift+l++ |
| Select next occurrence | ++ctrl+d++ |

### Navigation

| Action | Shortcut |
|---|---|
| Go to line | ++ctrl+g++ |
| Go to symbol | ++ctrl+shift+o++ |
| Go to definition | ++f12++ |
| Peek definition | ++alt+f12++ |
| Go back | ++alt+left++ |
| Go forward | ++alt+right++ |

### Debug

| Action | Shortcut |
|---|---|
| Start debugging | ++f5++ |
| Stop debugging | ++shift+f5++ |
| Step over | ++f10++ |
| Step into | ++f11++ |
| Step out | ++shift+f11++ |
| Toggle breakpoint | ++f9++ |

---

## When Clauses

When clauses control whether a keybinding is active based on the current context. They are evaluated by the `when-clause.ts` module on the frontend.

### Context Variables

| Variable | Description |
|---|---|
| `editorTextFocus` | The text editor has focus |
| `editorHasSelection` | Text is selected in the editor |
| `editorReadonly` | The editor is in read-only mode |
| `terminalFocus` | The terminal has focus |
| `sideBarFocus` | The sidebar has focus |
| `inDebugMode` | A debug session is active |
| `debugState` | Current debug state (`running`, `paused`, `stopped`) |
| `inputFocus` | An input field has focus |
| `panelFocus` | The panel area has focus |
| `searchViewletFocus` | The search view has focus |
| `explorerViewletFocus` | The explorer view has focus |

### Operators

| Operator | Example | Description |
|---|---|---|
| `==` | `debugState == 'paused'` | Equality |
| `!=` | `resourceScheme != 'untitled'` | Inequality |
| `&&` | `editorTextFocus && !editorReadonly` | Logical AND |
| `\|\|` | `terminalFocus \|\| editorFocus` | Logical OR |
| `!` | `!inDebugMode` | Logical NOT |
| `=~` | `resourceFilename =~ /.*\.md/` | Regex match |

### Examples

```json
[
  {
    "key": "ctrl+enter",
    "command": "git.commit",
    "when": "scmInputFocus"
  },
  {
    "key": "escape",
    "command": "workbench.action.closeQuickOpen",
    "when": "inQuickOpen"
  },
  {
    "key": "f5",
    "command": "workbench.action.debug.continue",
    "when": "inDebugMode && debugState == 'paused'"
  }
]
```

---

## Key Chord Sequences

DSCode supports multi-key sequences (chords) where you press one key combination followed by another.

### Format

Chords are written with a space separating the two key combinations:

```json
{
  "key": "ctrl+k ctrl+s",
  "command": "workbench.action.openGlobalKeybindings"
}
```

This means: press ++ctrl+k++, release, then press ++ctrl+s++.

### Common Chords

| Chord | Action |
|---|---|
| ++ctrl+k++ ++ctrl+s++ | Open Keyboard Shortcuts |
| ++ctrl+k++ ++ctrl+f++ | Format selection |
| ++ctrl+k++ ++ctrl+c++ | Add line comment |
| ++ctrl+k++ ++ctrl+u++ | Remove line comment |
| ++ctrl+k++ ++ctrl+w++ | Close all editors |
| ++ctrl+k++ ++ctrl+0++ | Fold all regions |
| ++ctrl+k++ ++ctrl+j++ | Unfold all regions |
| ++ctrl+k++ ++ctrl+backslash++ | Split editor down |

!!! note
    After pressing the first chord key (e.g., ++ctrl+k++), DSCode waits briefly for the second key. The Status Bar shows `(Ctrl+K) was pressed. Waiting for second key of chord...` to indicate that a chord is in progress. Press ++escape++ to cancel.

---

## Platform Differences

DSCode adapts keyboard shortcuts to the current operating system. The `KeybindingRegistry` detects the platform and converts key names accordingly:

```rust
pub fn platform_key(&self, target_platform: &str) -> String {
    match target_platform {
        "macos" => key.replace("Ctrl", "Cmd"),
        "windows" | "linux" => key.replace("Cmd", "Ctrl"),
        _ => key.to_string(),
    }
}
```

### macOS Modifier Mapping

| Generic | macOS |
|---|---|
| ++ctrl++ | ++cmd++ |
| ++alt++ | ++option++ |
| ++ctrl+shift++ | ++cmd+shift++ |

### Platform-Specific Bindings in keybindings.json

You can define bindings that apply only to a specific platform:

```json
[
  {
    "key": "cmd+shift+p",
    "command": "workbench.action.showCommands",
    "platform": "macos"
  },
  {
    "key": "ctrl+shift+p",
    "command": "workbench.action.showCommands",
    "platform": "windows"
  },
  {
    "key": "ctrl+shift+p",
    "command": "workbench.action.showCommands",
    "platform": "linux"
  }
]
```

!!! tip
    In most cases, you do not need to specify platform-specific bindings. DSCode automatically translates ++ctrl++ to ++cmd++ on macOS. Only use the `platform` field when you need genuinely different shortcuts per platform.

---

## Resolving Conflicts

When multiple keybindings use the same key combination, DSCode resolves conflicts using these rules:

1. **Specificity** -- a binding with a `when` clause wins over one without.
2. **User bindings** override extension bindings, which override defaults.
3. **Later entries** in `keybindings.json` override earlier entries with the same key.

### Finding Conflicts

1. Open the Keyboard Shortcuts editor (++ctrl+k++ ++ctrl+s++).
2. Search for the key combination (e.g., type `ctrl+shift+p` in the search bar).
3. All bindings using that combination are listed, ordered by precedence.
4. Bindings that are overridden by a higher-precedence binding show a warning icon.

### Resolving a Conflict

- **Change one of the bindings** to use a different key combination.
- **Add a when clause** to make the bindings context-dependent so they do not overlap.
- **Remove a binding** by prefixing the command with `-` in `keybindings.json`.

!!! warning
    Be cautious when overriding default bindings for common actions like ++ctrl+s++ (Save) or ++ctrl+z++ (Undo). If you accidentally break a critical shortcut, you can reset all keybindings by deleting your `keybindings.json` file and restarting DSCode.
