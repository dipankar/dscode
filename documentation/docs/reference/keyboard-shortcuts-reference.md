---
title: Keyboard Shortcuts Reference
description: Complete reference of all default keyboard shortcuts in DSCode, organized by category with macOS and Linux/Windows bindings.
---

# Keyboard Shortcuts Reference

This page lists all default keyboard shortcuts in DSCode. Shortcuts follow the same defaults as VS Code, so experienced users will feel at home immediately.

!!! tip "Customizing shortcuts"
    Edit your keybindings file at `~/.config/dscode/keybindings.json` (Linux), `~/Library/Application Support/dscode/keybindings.json` (macOS), or `%APPDATA%\dscode\keybindings.json` (Windows). You can also open it from the Command Palette with **Preferences: Open Keyboard Shortcuts (JSON)**.

---

## General

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Command Palette | ++cmd+shift+p++ | ++ctrl+shift+p++ |
| Quick Open (Go to File) | ++cmd+p++ | ++ctrl+p++ |
| New Window | ++cmd+shift+n++ | ++ctrl+shift+n++ |
| Close Window | ++cmd+shift+w++ | ++alt+f4++ |
| User Settings | ++cmd+comma++ | ++ctrl+comma++ |
| Keyboard Shortcuts | ++cmd+k++ ++cmd+s++ | ++ctrl+k++ ++ctrl+s++ |
| Toggle Full Screen | ++ctrl+cmd+f++ | ++f11++ |
| Toggle Zen Mode | ++cmd+k++ ++z++ | ++ctrl+k++ ++z++ |

</div>

---

## Basic Editing

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Cut line (empty selection) | ++cmd+x++ | ++ctrl+x++ |
| Copy line (empty selection) | ++cmd+c++ | ++ctrl+c++ |
| Paste | ++cmd+v++ | ++ctrl+v++ |
| Undo | ++cmd+z++ | ++ctrl+z++ |
| Redo | ++cmd+shift+z++ | ++ctrl+shift+z++ |
| Find | ++cmd+f++ | ++ctrl+f++ |
| Replace | ++cmd+option+f++ | ++ctrl+h++ |
| Find Next | ++cmd+g++ | ++f3++ |
| Find Previous | ++cmd+shift+g++ | ++shift+f3++ |
| Select All Occurrences of Find Match | ++cmd+option+enter++ | ++ctrl+alt+enter++ |
| Indent Line | ++cmd+bracket-right++ | ++ctrl+bracket-right++ |
| Outdent Line | ++cmd+bracket-left++ | ++ctrl+bracket-left++ |
| Toggle Line Comment | ++cmd+slash++ | ++ctrl+slash++ |
| Toggle Block Comment | ++cmd+shift+a++ | ++ctrl+shift+a++ |
| Move Line Up | ++option+up++ | ++alt+up++ |
| Move Line Down | ++option+down++ | ++alt+down++ |
| Copy Line Up | ++option+shift+up++ | ++alt+shift+up++ |
| Copy Line Down | ++option+shift+down++ | ++alt+shift+down++ |
| Delete Line | ++cmd+shift+k++ | ++ctrl+shift+k++ |
| Insert Line Below | ++cmd+enter++ | ++ctrl+enter++ |
| Insert Line Above | ++cmd+shift+enter++ | ++ctrl+shift+enter++ |
| Jump to Matching Bracket | ++cmd+shift+backslash++ | ++ctrl+shift+backslash++ |
| Add Line Indent | ++cmd+bracket-right++ | ++ctrl+bracket-right++ |
| Remove Line Indent | ++cmd+bracket-left++ | ++ctrl+bracket-left++ |
| Select Current Line | ++cmd+l++ | ++ctrl+l++ |
| Smart Select Expand | ++ctrl+shift+cmd+right++ | ++shift+alt+right++ |
| Smart Select Shrink | ++ctrl+shift+cmd+left++ | ++shift+alt+left++ |
| Fold Region | ++cmd+option+bracket-left++ | ++ctrl+shift+bracket-left++ |
| Unfold Region | ++cmd+option+bracket-right++ | ++ctrl+shift+bracket-right++ |
| Fold All | ++cmd+k++ ++cmd+0++ | ++ctrl+k++ ++ctrl+0++ |
| Unfold All | ++cmd+k++ ++cmd+j++ | ++ctrl+k++ ++ctrl+j++ |
| Trim Trailing Whitespace | ++cmd+k++ ++cmd+x++ | ++ctrl+k++ ++ctrl+x++ |
| Transform to Uppercase | ++cmd+k++ ++cmd+u++ | ++ctrl+k++ ++ctrl+u++ |
| Transform to Lowercase | ++cmd+k++ ++cmd+l++ | ++ctrl+k++ ++ctrl+l++ |

</div>

---

## Multi-Cursor and Selection

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Insert Cursor Above | ++cmd+option+up++ | ++ctrl+alt+up++ |
| Insert Cursor Below | ++cmd+option+down++ | ++ctrl+alt+down++ |
| Insert Cursor at End of Each Selected Line | ++option+shift+i++ | ++alt+shift+i++ |
| Add Cursor at Mouse Click | ++option+"click"++ | ++alt+"click"++ |
| Select Current Word | ++cmd+d++ | ++ctrl+d++ |
| Select All Occurrences of Current Word | ++cmd+shift+l++ | ++ctrl+shift+l++ |
| Select All Occurrences of Current Selection | ++cmd+option+enter++ | ++ctrl+alt+enter++ |
| Column (Box) Selection | ++option+shift+"drag"++ | ++alt+shift+"drag"++ |
| Column Selection Up | ++cmd+option+shift+up++ | ++ctrl+alt+shift+up++ |
| Column Selection Down | ++cmd+option+shift+down++ | ++ctrl+alt+shift+down++ |
| Column Selection Left | ++cmd+option+shift+left++ | ++ctrl+alt+shift+left++ |
| Column Selection Right | ++cmd+option+shift+right++ | ++ctrl+alt+shift+right++ |
| Undo Last Cursor Operation | ++cmd+u++ | ++ctrl+u++ |

</div>

---

## Navigation

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Go to File (Quick Open) | ++cmd+p++ | ++ctrl+p++ |
| Go to Symbol in Workspace | ++cmd+t++ | ++ctrl+t++ |
| Go to Symbol in File | ++cmd+shift+o++ | ++ctrl+shift+o++ |
| Go to Line | ++ctrl+g++ | ++ctrl+g++ |
| Go to Definition | ++f12++ | ++f12++ |
| Peek Definition | ++option+f12++ | ++alt+f12++ |
| Go to Type Definition | ++cmd+f12++ | ++ctrl+f12++ |
| Go to Implementation | ++cmd+f12++ | ++ctrl+f12++ |
| Go to References | ++shift+f12++ | ++shift+f12++ |
| Show Hover | ++cmd+k++ ++cmd+i++ | ++ctrl+k++ ++ctrl+i++ |
| Go Back | ++ctrl+minus++ | ++alt+left++ |
| Go Forward | ++ctrl+shift+minus++ | ++alt+right++ |
| Go to Beginning of File | ++cmd+home++ | ++ctrl+home++ |
| Go to End of File | ++cmd+end++ | ++ctrl+end++ |
| Go to Beginning of Line | ++home++ | ++home++ |
| Go to End of Line | ++end++ | ++end++ |
| Trigger Suggest (IntelliSense) | ++ctrl+space++ | ++ctrl+space++ |
| Trigger Parameter Hints | ++cmd+shift+space++ | ++ctrl+shift+space++ |
| Open Next Error/Warning | ++f8++ | ++f8++ |
| Open Previous Error/Warning | ++shift+f8++ | ++shift+f8++ |

</div>

---

## Search and Replace

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Find | ++cmd+f++ | ++ctrl+f++ |
| Replace | ++cmd+option+f++ | ++ctrl+h++ |
| Find in Files | ++cmd+shift+f++ | ++ctrl+shift+f++ |
| Replace in Files | ++cmd+shift+h++ | ++ctrl+shift+h++ |
| Find Next | ++cmd+g++ | ++f3++ |
| Find Previous | ++cmd+shift+g++ | ++shift+f3++ |
| Toggle Match Case | ++option+c++ (in find widget) | ++alt+c++ (in find widget) |
| Toggle Match Whole Word | ++option+w++ (in find widget) | ++alt+w++ (in find widget) |
| Toggle Use Regular Expression | ++option+r++ (in find widget) | ++alt+r++ (in find widget) |

</div>

---

## Editor Management

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Split Editor Right | ++cmd+backslash++ | ++ctrl+backslash++ |
| Split Editor Down | ++cmd+k++ ++cmd+backslash++ | ++ctrl+k++ ++ctrl+backslash++ |
| Close Editor | ++cmd+w++ | ++ctrl+w++ |
| Close All Editors | ++cmd+k++ ++cmd+w++ | ++ctrl+k++ ++ctrl+w++ |
| Close All Editors in Group | ++cmd+k++ ++w++ | ++ctrl+k++ ++w++ |
| Switch to Next Editor | ++cmd+option+right++ | ++ctrl+pagedown++ |
| Switch to Previous Editor | ++cmd+option+left++ | ++ctrl+pageup++ |
| Focus 1st Editor Group | ++cmd+1++ | ++ctrl+1++ |
| Focus 2nd Editor Group | ++cmd+2++ | ++ctrl+2++ |
| Focus 3rd Editor Group | ++cmd+3++ | ++ctrl+3++ |
| Move Editor to Next Group | ++cmd+ctrl+right++ | ++ctrl+alt+right++ |
| Move Editor to Previous Group | ++cmd+ctrl+left++ | ++ctrl+alt+left++ |
| Move Editor Tab Left | ++cmd+shift+pageup++ | ++ctrl+shift+pageup++ |
| Move Editor Tab Right | ++cmd+shift+pagedown++ | ++ctrl+shift+pagedown++ |
| Pin/Unpin Editor | ++cmd+k++ ++shift+enter++ | ++ctrl+k++ ++shift+enter++ |
| Reopen Closed Editor | ++cmd+shift+t++ | ++ctrl+shift+t++ |

</div>

---

## File Management

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| New File | ++cmd+n++ | ++ctrl+n++ |
| New Window | ++cmd+shift+n++ | ++ctrl+shift+n++ |
| Open File | ++cmd+o++ | ++ctrl+o++ |
| Open Folder | ++cmd+o++ | ++ctrl+k++ ++ctrl+o++ |
| Save | ++cmd+s++ | ++ctrl+s++ |
| Save As | ++cmd+shift+s++ | ++ctrl+shift+s++ |
| Save All | ++cmd+option+s++ | ++ctrl+k++ ++s++ |
| Close File | ++cmd+w++ | ++ctrl+w++ |
| Close All | ++cmd+k++ ++cmd+w++ | ++ctrl+k++ ++ctrl+w++ |
| Revert File | (none) | (none) |
| Copy Path of Active File | ++cmd+k++ ++p++ | ++ctrl+k++ ++p++ |
| Reveal in Explorer / Finder | ++cmd+k++ ++r++ | ++ctrl+k++ ++r++ |

</div>

---

## Display

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Toggle Full Screen | ++ctrl+cmd+f++ | ++f11++ |
| Toggle Sidebar Visibility | ++cmd+b++ | ++ctrl+b++ |
| Show Explorer | ++cmd+shift+e++ | ++ctrl+shift+e++ |
| Show Search | ++cmd+shift+f++ | ++ctrl+shift+f++ |
| Show Source Control | ++ctrl+shift+g++ | ++ctrl+shift+g++ |
| Show Run and Debug | ++cmd+shift+d++ | ++ctrl+shift+d++ |
| Show Extensions | ++cmd+shift+x++ | ++ctrl+shift+x++ |
| Toggle Terminal | ++ctrl+grave++ | ++ctrl+grave++ |
| Toggle Problems Panel | ++cmd+shift+m++ | ++ctrl+shift+m++ |
| Toggle Output Panel | ++cmd+shift+u++ | ++ctrl+shift+u++ |
| Toggle Debug Console | ++cmd+shift+y++ | ++ctrl+shift+y++ |
| Zoom In | ++cmd+equal++ | ++ctrl+equal++ |
| Zoom Out | ++cmd+minus++ | ++ctrl+minus++ |
| Reset Zoom | ++cmd+0++ | ++ctrl+0++ |
| Toggle Word Wrap | ++option+z++ | ++alt+z++ |

</div>

---

## Debug

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Start / Continue Debugging | ++f5++ | ++f5++ |
| Stop Debugging | ++shift+f5++ | ++shift+f5++ |
| Restart Debugging | ++cmd+shift+f5++ | ++ctrl+shift+f5++ |
| Toggle Breakpoint | ++f9++ | ++f9++ |
| Step Over | ++f10++ | ++f10++ |
| Step Into | ++f11++ | ++f11++ |
| Step Out | ++shift+f11++ | ++shift+f11++ |
| Start Without Debugging | ++ctrl+f5++ | ++ctrl+f5++ |
| Show Debug Hover | ++cmd+k++ ++cmd+i++ | ++ctrl+k++ ++ctrl+i++ |
| Inline Breakpoint | ++shift+f9++ | ++shift+f9++ |

</div>

---

## Integrated Terminal

<div class="shortcut-table" markdown>

| Action | macOS | Linux / Windows |
|---|---|---|
| Toggle Terminal | ++ctrl+grave++ | ++ctrl+grave++ |
| Create New Terminal | ++ctrl+shift+grave++ | ++ctrl+shift+grave++ |
| Split Terminal | ++cmd+backslash++ (in terminal) | ++ctrl+shift+5++ |
| Focus Terminal | ++ctrl+grave++ | ++ctrl+grave++ |
| Focus Next Terminal | ++cmd+shift+bracket-right++ (in terminal) | ++ctrl+pagedown++ (in terminal) |
| Focus Previous Terminal | ++cmd+shift+bracket-left++ (in terminal) | ++ctrl+pageup++ (in terminal) |
| Clear Terminal | ++cmd+k++ (in terminal) | ++ctrl+k++ (in terminal) |
| Kill Terminal | (none) | (none) |
| Scroll Up | ++cmd+up++ (in terminal) | ++ctrl+shift+up++ (in terminal) |
| Scroll Down | ++cmd+down++ (in terminal) | ++ctrl+shift+down++ (in terminal) |
| Scroll to Top | ++cmd+home++ (in terminal) | ++ctrl+home++ (in terminal) |
| Scroll to Bottom | ++cmd+end++ (in terminal) | ++ctrl+end++ (in terminal) |
| Copy Selection | ++cmd+c++ (in terminal) | ++ctrl+shift+c++ (in terminal) |
| Paste | ++cmd+v++ (in terminal) | ++ctrl+shift+v++ (in terminal) |
| Select All | ++cmd+a++ (in terminal) | (none) |
| Find in Terminal | ++cmd+f++ (in terminal) | ++ctrl+shift+f++ (in terminal) |

</div>

---

## Custom Keybindings

You can add or override keybindings by editing `keybindings.json`. Each entry specifies a key combination, a command identifier, and an optional `when` clause for context.

```json title="keybindings.json"
[
    {
        "key": "ctrl+shift+t",
        "command": "workbench.action.terminal.new",
        "when": "!terminalFocus"
    },
    {
        "key": "cmd+shift+d",
        "command": "editor.action.duplicateSelection",
        "when": "editorTextFocus"
    },
    {
        "key": "ctrl+shift+p",
        "command": "-workbench.action.showCommands"
    }
]
```

!!! info "Key rules"
    - Use a minus sign prefix (`-`) on the command to **remove** a default keybinding.
    - The `when` clause uses DSCode's context key expressions (e.g., `editorTextFocus`, `terminalFocus`, `inDebugMode`).
    - On macOS, use `cmd` for the Command key and `option` for the Alt/Option key. On Linux/Windows, use `ctrl` and `alt`.

### Common `when` Clause Contexts

| Context Key | Description |
|---|---|
| `editorTextFocus` | An editor has keyboard focus |
| `editorHasSelection` | Text is selected in the editor |
| `editorReadonly` | The editor is read-only |
| `terminalFocus` | The terminal has keyboard focus |
| `inDebugMode` | A debug session is active |
| `sideBarVisible` | The sidebar is open |
| `panelFocus` | A panel (terminal, output, problems) has focus |
| `searchViewletVisible` | The search sidebar is open |
| `explorerViewletVisible` | The file explorer sidebar is open |
| `suggestWidgetVisible` | The IntelliSense suggestion widget is open |
| `findWidgetVisible` | The find widget is open |
| `inQuickOpen` | The Quick Open dialog is open |
| `inputFocus` | An input field has focus |

---

## See Also

- [Keyboard Shortcuts](../user-guide/keyboard-shortcuts.md) -- user guide with tips and workflows
- [Configuration Options](configuration-options.md) -- all `settings.json` options
- [CLI Reference](cli-commands.md) -- command-line flags
