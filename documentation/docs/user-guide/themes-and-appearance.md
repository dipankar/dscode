# Themes and Appearance

DSCode provides extensive theming and appearance customization through a theme system managed by the Rust `ThemeRegistry`. The registry stores color themes, icon themes, and product icon themes, and emits events to the frontend when the active theme changes. Extensions can contribute new themes that are loaded and registered automatically.

---

## Color Themes

Color themes control the colors used throughout the DSCode UI -- the editor background, syntax highlighting, sidebar, status bar, and more.

### Theme Types

The `ThemeRegistry` classifies color themes into four types:

```rust
pub enum ThemeType {
    Light,
    Dark,
    HighContrast,
    HighContrastLight,
}
```

| Type | Description |
|---|---|
| **Dark** | Light text on a dark background (default) |
| **Light** | Dark text on a light background |
| **High Contrast** | High contrast dark theme for accessibility |
| **High Contrast Light** | High contrast light theme for accessibility |

### Built-in Themes

DSCode ships with several built-in themes:

- **DSCode Dark** (default) -- a modern dark theme
- **DSCode Light** -- a clean light theme
- **High Contrast** -- dark theme with increased contrast
- **High Contrast Light** -- light theme with increased contrast

### Changing the Color Theme

**Method 1: Command Palette**

1. Press ++ctrl+shift+p++ (or ++cmd+shift+p++ on macOS).
2. Type `Preferences: Color Theme`.
3. Browse the list of available themes. A live preview is applied as you navigate.
4. Press ++enter++ to confirm your selection.

**Method 2: Settings**

Set the theme directly in `settings.json`:

```json
{
  "workbench.colorTheme": "One Dark Pro"
}
```

**Method 3: Keyboard Shortcut**

Press ++ctrl+k++ ++ctrl+t++ to open the theme picker directly.

!!! tip
    As you arrow through themes in the picker, DSCode applies a live preview so you can see exactly how each theme looks before committing to it.

---

## Theme Structure

Each color theme registered with the `ThemeRegistry` contains two main components:

### UI Colors

A map of color keys to hex values that define the appearance of UI elements:

```rust
pub struct ColorTheme {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub theme_type: ThemeType,
    pub colors: HashMap<String, String>,
    pub token_colors: Vec<TokenColor>,
}
```

Example color keys:

```json
{
  "editor.background": "#1e1e2e",
  "editor.foreground": "#cdd6f4",
  "sideBar.background": "#181825",
  "statusBar.background": "#11111b",
  "activityBar.background": "#11111b",
  "tab.activeBackground": "#1e1e2e",
  "tab.inactiveBackground": "#181825"
}
```

### Token Colors

Token colors define syntax highlighting rules. Each rule targets TextMate scopes:

```rust
pub struct TokenColor {
    pub name: Option<String>,
    pub scope: Vec<String>,
    pub settings: TokenColorSettings,
}

pub struct TokenColorSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<String>,
}
```

Example:

```json
{
  "name": "Keywords",
  "scope": ["keyword", "storage.type", "storage.modifier"],
  "settings": {
    "foreground": "#cba6f7",
    "fontStyle": "bold"
  }
}
```

---

## Color Customization

You can override specific colors from any theme without creating a full custom theme. Add overrides to your `settings.json`:

```json
{
  "workbench.colorCustomizations": {
    "editor.background": "#1a1a2e",
    "editor.lineHighlightBackground": "#2a2a4e",
    "editorCursor.foreground": "#e2b714",
    "statusBar.background": "#0f0f1a",
    "terminal.background": "#1a1a2e"
  }
}
```

For syntax highlighting overrides:

```json
{
  "editor.tokenColorCustomizations": {
    "comments": "#6c7086",
    "strings": "#a6e3a1",
    "keywords": "#cba6f7",
    "functions": "#89b4fa",
    "textMateRules": [
      {
        "scope": "entity.name.function",
        "settings": {
          "foreground": "#89dceb",
          "fontStyle": "italic"
        }
      }
    ]
  }
}
```

!!! note
    Color customizations override the active theme. They persist even when you switch themes. To clear customizations, remove them from `settings.json`.

---

## Icon Themes

Icon themes control the file and folder icons displayed in the Explorer, editor tabs, and breadcrumbs.

### Icon Theme Structure

```rust
pub struct IconTheme {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub icon_definitions: HashMap<String, IconDefinition>,
    pub file_associations: HashMap<String, String>,
    pub folder_associations: HashMap<String, String>,
    pub file_extensions: HashMap<String, String>,
    pub language_ids: HashMap<String, String>,
}
```

Icon themes map file extensions, file names, folder names, and language identifiers to specific icon images.

### Changing the Icon Theme

1. Press ++ctrl+shift+p++.
2. Type `Preferences: File Icon Theme`.
3. Select from the list of installed icon themes.

Or set it in `settings.json`:

```json
{
  "workbench.iconTheme": "material-icon-theme"
}
```

### Popular Icon Themes

Install icon themes from the extension marketplace. Some popular options include:

- **Material Icon Theme** -- comprehensive material design icons
- **vscode-icons** -- extensive file type coverage
- **Catppuccin Icons** -- icons matching the Catppuccin color palette

---

## Selecting Themes via the Command Palette

The Command Palette provides quick access to all theme-related settings:

| Command | Description |
|---|---|
| `Preferences: Color Theme` | Change the color theme |
| `Preferences: File Icon Theme` | Change the file icon theme |
| `Preferences: Product Icon Theme` | Change the product icon theme |
| `Preferences: Color Theme by Type` | Filter themes by light/dark/high contrast |

The Rust backend provides the API for listing and activating themes:

```rust
// Get all themes, optionally filtered by type
pub async fn get_all_color_themes(
    registry: State<'_, ThemeRegistry>,
) -> Result<Vec<ColorTheme>, String>

pub async fn get_color_themes_by_type(
    theme_type: ThemeType,
    registry: State<'_, ThemeRegistry>,
) -> Result<Vec<ColorTheme>, String>

// Set the active theme
pub async fn set_active_color_theme(
    theme_id: String,
    registry: State<'_, ThemeRegistry>,
) -> Result<(), String>
```

---

## Theme Registry

The Rust `ThemeRegistry` is the central store for all theme data. It runs on the backend and provides thread-safe access to theme state.

### How Themes Are Loaded

1. **Built-in themes** are registered at application startup.
2. **Extension-contributed themes** are loaded when the extension host activates an extension that contributes themes (defined in the extension's `package.json` under `contributes.themes`).
3. The `register_color_theme` Tauri command adds a theme to the registry.
4. When a theme is activated, the registry emits a `theme-changed` event to the frontend.
5. The frontend `theme.ts` module applies the theme colors to CSS custom properties and updates the Monaco editor's theme.

### Theme Events

| Event | Description |
|---|---|
| `theme-changed` | Active color theme changed |
| `icon-theme-changed` | Active icon theme changed |
| `product-icon-theme-changed` | Active product icon theme changed |

---

## Product Icon Themes

Product icon themes customize the icons used for UI elements such as Activity Bar icons, status bar icons, and toolbar buttons (as opposed to file/folder icons).

```rust
pub struct ProductIconTheme {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub icon_definitions: HashMap<String, ProductIconDefinition>,
}

pub struct ProductIconDefinition {
    pub font_character: String,
    pub font_color: Option<String>,
}
```

Product icons use icon fonts (such as Codicons) where each icon is defined by a font character code. To change the product icon theme:

1. Press ++ctrl+shift+p++.
2. Type `Preferences: Product Icon Theme`.
3. Select from the list of installed product icon themes.

---

## Font Customization

### Editor Font

Configure the font used in the code editor:

```json
{
  "editor.fontFamily": "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
  "editor.fontSize": 14,
  "editor.fontWeight": "400",
  "editor.fontLigatures": true,
  "editor.lineHeight": 1.6,
  "editor.letterSpacing": 0.5
}
```

!!! tip
    Enable `editor.fontLigatures` to use programming ligatures in fonts that support them (such as JetBrains Mono, Fira Code, or Cascadia Code). Ligatures render multi-character sequences like `=>`, `!=`, and `>=` as single glyphs.

### Terminal Font

Configure the font used in the integrated terminal independently from the editor:

```json
{
  "terminal.integrated.fontFamily": "'JetBrains Mono', monospace",
  "terminal.integrated.fontSize": 13,
  "terminal.integrated.fontWeight": "400",
  "terminal.integrated.lineHeight": 1.2
}
```

### UI Font

The documentation site and UI chrome use the Inter font family (configured in `mkdocs.yml`). The application UI font can be customized:

```json
{
  "workbench.fontFamily": "Inter, -apple-system, BlinkMacSystemFont, sans-serif",
  "workbench.fontSize": 13
}
```

!!! note
    Font changes take effect immediately. If a specified font is not installed on your system, DSCode falls back to the next font in the `fontFamily` list, and ultimately to the system default monospace font.

---

## UI Zoom

Adjust the overall zoom level of the DSCode UI to make everything larger or smaller.

| Action | Shortcut |
|---|---|
| Zoom In | ++ctrl+equal++ (or ++cmd+equal++ on macOS) |
| Zoom Out | ++ctrl+minus++ (or ++cmd+minus++ on macOS) |
| Reset Zoom | ++ctrl+0++ (or ++cmd+0++ on macOS) |

You can also set the zoom level in settings:

```json
{
  "window.zoomLevel": 0
}
```

The zoom level is a numeric value where `0` is the default (100%), `1` is approximately 120%, and `-1` is approximately 80%. Fractional values are supported.

!!! warning
    Extreme zoom levels (below `-3` or above `5`) may cause layout issues. If the UI becomes unusable due to an extreme zoom level, reset it by editing `settings.json` directly or by pressing ++ctrl+0++.

---

## Appearance Shortcuts Reference

| Action | Shortcut |
|---|---|
| Open color theme picker | ++ctrl+k++ ++ctrl+t++ |
| Color theme via Command Palette | ++ctrl+shift+p++ then `Color Theme` |
| File icon theme | ++ctrl+shift+p++ then `File Icon Theme` |
| Product icon theme | ++ctrl+shift+p++ then `Product Icon Theme` |
| Zoom in | ++ctrl+equal++ |
| Zoom out | ++ctrl+minus++ |
| Reset zoom | ++ctrl+0++ |
| Toggle sidebar | ++ctrl+b++ |
| Toggle panel | ++ctrl+j++ |
| Toggle full screen | ++f11++ |
| Toggle Zen mode | ++ctrl+k++ ++z++ |
