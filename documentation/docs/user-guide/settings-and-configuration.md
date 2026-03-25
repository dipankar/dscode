# Settings and Configuration

DSCode uses a layered configuration system managed by the Rust `ConfigurationRegistry` with support for user-level, workspace-level, and workspace-folder-level settings. The `SettingsModal.svelte` component provides the graphical settings UI, while settings are persisted as JSON files on disk.

---

## Settings UI

Open the Settings UI with ++ctrl+comma++ (or ++cmd+comma++ on macOS), or through the Command Palette:

1. Press ++ctrl+shift+p++.
2. Type `Preferences: Open Settings (UI)`.
3. Press ++enter++.

The Settings UI (`SettingsModal.svelte`, backed by `settings_ui_ops` and `settings_ui_registry` in the Rust backend) displays settings in a searchable, categorized list. Each setting shows:

- **Name** and description
- **Current value** (editable inline)
- **Default value**
- **Scope** (User, Workspace, or Workspace Folder)
- **Source** -- whether the value comes from the default, user settings, workspace settings, or an extension

!!! tip
    Click the gear icon next to any setting to reset it to its default value, copy its JSON key, or copy it as a JSON setting.

---

## User vs. Workspace Settings

DSCode supports three configuration scopes, defined in the Rust `ConfigurationRegistry`:

```rust
pub enum ConfigurationScope {
    User,
    Workspace,
    WorkspaceFolder,
}
```

### User Settings

- Apply globally to all projects you open in DSCode.
- Stored in your platform-specific configuration directory:

=== "macOS"

    ```
    ~/Library/Application Support/com.dscode.app/settings.json
    ```

=== "Linux"

    ```
    ~/.config/com.dscode.app/settings.json
    ```

=== "Windows"

    ```
    %APPDATA%\com.dscode.app\settings.json
    ```

### Workspace Settings

- Apply only to the current workspace (project).
- Stored in `.vscode/settings.json` within the workspace root.
- Override user settings when both define the same key.

### Workspace Folder Settings

- Apply to a specific folder in a multi-root workspace.
- Stored in `.vscode/settings.json` within each folder.
- Override both user and workspace settings.

!!! note
    The precedence order is: **Default** < **User** < **Workspace** < **Workspace Folder**. A setting defined at a more specific scope always wins.

---

## settings.json

For full control, you can edit the raw JSON settings file directly.

### Opening settings.json

| Action | Method |
|---|---|
| Open User settings.json | Command Palette: `Preferences: Open User Settings (JSON)` |
| Open Workspace settings.json | Command Palette: `Preferences: Open Workspace Settings (JSON)` |

### File Format

The `settings.json` file is a standard JSON object where keys are dot-separated setting identifiers:

```json
{
  "editor.fontSize": 14,
  "editor.tabSize": 2,
  "editor.wordWrap": "on",
  "editor.minimap.enabled": false,
  "workbench.colorTheme": "One Dark Pro",
  "terminal.integrated.fontSize": 13,
  "files.autoSave": "afterDelay",
  "files.autoSaveDelay": 1000
}
```

!!! tip
    The JSON editor provides IntelliSense for setting keys and values. Press ++ctrl+space++ to see available settings and their allowed values.

---

## Common Settings

### Editor Settings

| Setting | Type | Default | Description |
|---|---|---|---|
| `editor.fontSize` | number | `14` | Font size in pixels |
| `editor.fontFamily` | string | `"JetBrains Mono"` | Editor font family |
| `editor.tabSize` | number | `4` | Number of spaces per tab |
| `editor.insertSpaces` | boolean | `true` | Use spaces instead of tabs |
| `editor.wordWrap` | string | `"off"` | Word wrap mode (`off`, `on`, `wordWrapColumn`, `bounded`) |
| `editor.lineNumbers` | string | `"on"` | Line number display (`on`, `off`, `relative`, `interval`) |
| `editor.minimap.enabled` | boolean | `true` | Show the minimap |
| `editor.renderWhitespace` | string | `"selection"` | Render whitespace characters |
| `editor.cursorStyle` | string | `"line"` | Cursor style (`line`, `block`, `underline`) |
| `editor.formatOnSave` | boolean | `false` | Auto-format files when saving |
| `editor.bracketPairColorization.enabled` | boolean | `true` | Colorize matching bracket pairs |

### Theme Settings

| Setting | Type | Default | Description |
|---|---|---|---|
| `workbench.colorTheme` | string | `"DSCode Dark"` | Active color theme |
| `workbench.iconTheme` | string | `"dscode-icons"` | Active file icon theme |
| `workbench.productIconTheme` | string | `"Default"` | Active product icon theme |

### File Settings

| Setting | Type | Default | Description |
|---|---|---|---|
| `files.autoSave` | string | `"off"` | Auto-save mode (`off`, `afterDelay`, `onFocusChange`, `onWindowChange`) |
| `files.autoSaveDelay` | number | `1000` | Delay in ms before auto-save (when mode is `afterDelay`) |
| `files.encoding` | string | `"utf8"` | Default file encoding |
| `files.eol` | string | `"auto"` | Default end-of-line character (`\n`, `\r\n`, `auto`) |
| `files.trimTrailingWhitespace` | boolean | `false` | Trim trailing whitespace on save |
| `files.insertFinalNewline` | boolean | `false` | Insert final newline on save |

### Terminal Settings

| Setting | Type | Default | Description |
|---|---|---|---|
| `terminal.integrated.fontSize` | number | `14` | Terminal font size |
| `terminal.integrated.fontFamily` | string | `"JetBrains Mono"` | Terminal font family |
| `terminal.integrated.cursorStyle` | string | `"block"` | Terminal cursor style |
| `terminal.integrated.scrollback` | number | `1000` | Scrollback buffer size |

---

## Settings Search

The Settings UI includes a search bar at the top. Type any keyword to filter settings:

- Search by **setting name**: `fontSize`, `tabSize`, `wordWrap`
- Search by **description**: `"trailing whitespace"`, `"auto save"`
- Search by **category**: `editor`, `terminal`, `workbench`, `files`
- Search by **extension name**: settings contributed by extensions are searchable by the extension identifier

The search uses the configuration schemas registered by extensions through the `register_configuration_schema` Tauri command:

```rust
pub struct ConfigurationSchema {
    pub key: String,
    pub scope: ConfigurationScope,
    pub value_type: String,
    pub default: Value,
    pub description: String,
    pub enum_values: Option<Vec<Value>>,
}
```

!!! tip
    Use the `@modified` filter in the search bar to show only settings that have been changed from their defaults.

---

## Configuration Scopes

The configuration system uses a fallback chain to resolve values. When you request a setting value, the `ConfigurationRegistry` checks scopes in order:

```
Workspace Folder -> Workspace -> User -> Default
```

The Rust backend provides a dedicated command for this:

```rust
pub async fn get_configuration_with_fallback(
    key: String,
    scope: ConfigurationScope,
    registry: State<'_, ConfigurationRegistry>,
) -> Result<Option<Value>, String>
```

### Scope Indicators in the UI

In the Settings UI, each setting shows a scope dropdown that lets you choose where to apply the change:

- **User** -- applies everywhere
- **Workspace** -- applies to the current project only

A small indicator shows where the current effective value comes from. If a workspace setting overrides a user setting, both values are visible with the workspace value taking precedence.

### Language-Specific Settings

You can define settings that apply only to specific languages by wrapping them in a language identifier block:

```json
{
  "[python]": {
    "editor.tabSize": 4,
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "ms-python.python"
  },
  "[javascript]": {
    "editor.tabSize": 2,
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[rust]": {
    "editor.tabSize": 4,
    "editor.formatOnSave": true
  }
}
```

---

## Configuration Change Events

When a setting value changes, the Rust backend emits a `ConfigurationChangeEvent` to the frontend:

```rust
pub struct ConfigurationChangeEvent {
    pub affected_keys: Vec<String>,
    pub scope: ConfigurationScope,
}
```

The frontend `configuration.ts` module listens for these events and updates the relevant stores and components reactively. This ensures the editor, terminal, and other components immediately reflect configuration changes without requiring a restart.

!!! note
    Some settings (such as `editor.fontFamily` or `terminal.integrated.fontFamily`) take effect immediately. Others (such as extension-specific settings) may require the extension to reload. DSCode will prompt you when a reload is needed.

---

## Settings Shortcuts Reference

| Action | Shortcut |
|---|---|
| Open Settings UI | ++ctrl+comma++ |
| Open User settings.json | Command Palette: `Open User Settings (JSON)` |
| Open Workspace settings.json | Command Palette: `Open Workspace Settings (JSON)` |
| Search settings | Type in the Settings UI search bar |
| Reset a setting | Click the gear icon, then **Reset Setting** |
