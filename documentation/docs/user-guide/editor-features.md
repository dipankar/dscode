# Editor Features

DSCode uses the [Monaco Editor](https://microsoft.github.io/monaco-editor/) -- the same editor engine that powers VS Code -- to provide a rich, high-performance code editing experience. The Monaco instance is embedded within a Svelte `EditorArea` component and communicates with the Rust backend through Tauri IPC for file operations, language features, and more.

---

## Syntax Highlighting

Monaco provides syntax highlighting for **200+ languages** out of the box, including JavaScript, TypeScript, Python, Rust, Go, C/C++, Java, HTML, CSS, JSON, YAML, Markdown, and many more.

DSCode extends Monaco's built-in tokenization with a WASM-based TextMate grammar engine (`wasm-tokenizer.ts`) that loads `.tmLanguage` grammars from installed extensions, giving you the same accurate, scope-based highlighting you expect from VS Code.

!!! tip
    If a language does not appear to be highlighted correctly, check whether an extension providing a TextMate grammar for that language is installed. Open the Extensions view from the Activity Bar to browse and install language packs.

---

## IntelliSense and Autocomplete

IntelliSense provides smart completions based on variable types, function definitions, and imported modules. DSCode's language features pipeline works as follows:

1. The frontend `language-features.ts` module registers Monaco completion providers.
2. Completion requests are forwarded through the Rust `language_features_ops` commands to the appropriate Language Server Protocol (LSP) server.
3. Results are returned to Monaco and displayed in the autocomplete widget.

To trigger IntelliSense manually, press ++ctrl+space++.

!!! note
    IntelliSense quality depends on the language server available for your language. Install the appropriate language extension (e.g., `rust-analyzer` for Rust, `Pylance` for Python) for the best experience.

---

## Multi-Cursor Editing

Multi-cursor editing lets you place multiple cursors in the document and type in all locations simultaneously.

| Action | Shortcut |
|---|---|
| Add cursor at click position | ++alt+click++ |
| Add cursor above | ++ctrl+alt+up++ |
| Add cursor below | ++ctrl+alt+down++ |
| Select all occurrences of current word | ++ctrl+shift+l++ |
| Select next occurrence of current selection | ++ctrl+d++ |
| Undo last cursor operation | ++ctrl+u++ |

!!! tip
    Use ++ctrl+d++ to incrementally select the next occurrence of the current word. This is a quick way to rename a variable in a local scope without a full find-and-replace.

---

## Find and Replace

Open the in-editor find widget with ++ctrl+f++ and the find-and-replace widget with ++ctrl+h++.

The find widget supports:

- **Case-sensitive search** -- toggle with the `Aa` button or ++alt+c++
- **Whole-word matching** -- toggle with the `Ab|` button or ++alt+w++
- **Regular expressions** -- toggle with the `.*` button or ++alt+r++
- **Find in selection** -- restrict search to highlighted text

Navigate between matches with ++enter++ (next) and ++shift+enter++ (previous).

---

## Code Folding

Code folding allows you to collapse and expand regions of code to focus on the parts you care about.

| Action | Shortcut |
|---|---|
| Fold (collapse) region | ++ctrl+shift+bracket-left++ |
| Unfold (expand) region | ++ctrl+shift+bracket-right++ |
| Fold all regions | ++ctrl+k++ ++ctrl+0++ |
| Unfold all regions | ++ctrl+k++ ++ctrl+j++ |
| Fold level N (1-7) | ++ctrl+k++ ++ctrl+1++ through ++ctrl+k++ ++ctrl+7++ |

!!! note
    Monaco detects foldable regions using indentation by default. Languages with LSP support may provide more accurate folding ranges based on syntax analysis.

---

## Minimap

The minimap is a scaled-down overview of your file displayed on the right side of the editor. It provides a high-level view of the file structure and lets you quickly navigate to any section by clicking on it.

You can configure the minimap through settings:

```json
{
  "editor.minimap.enabled": true,
  "editor.minimap.maxColumn": 120,
  "editor.minimap.renderCharacters": true,
  "editor.minimap.side": "right"
}
```

---

## Bracket Matching

Monaco automatically highlights matching brackets when your cursor is next to one. This works for parentheses `()`, square brackets `[]`, and curly braces `{}`.

- **Jump to matching bracket**: ++ctrl+shift+backslash++
- **Bracket pair colorization** can be enabled in settings:

```json
{
  "editor.bracketPairColorization.enabled": true,
  "editor.guides.bracketPairs": "active"
}
```

---

## Auto-Indentation

The editor automatically indents new lines based on the language grammar and context. When you press ++enter++ after an opening brace or colon, the next line is indented appropriately.

You can control indentation behavior through settings:

```json
{
  "editor.autoIndent": "full",
  "editor.tabSize": 4,
  "editor.insertSpaces": true,
  "editor.detectIndentation": true
}
```

The `detectIndentation` option allows DSCode to automatically detect whether a file uses tabs or spaces and adjust accordingly.

---

## Word Wrap

Word wrap controls whether long lines are wrapped to fit within the viewport or extend beyond it with a horizontal scrollbar.

| Setting Value | Behavior |
|---|---|
| `"off"` | Lines are never wrapped (default) |
| `"on"` | Lines wrap at the viewport width |
| `"wordWrapColumn"` | Lines wrap at the column specified by `editor.wordWrapColumn` |
| `"bounded"` | Lines wrap at the minimum of the viewport width and `editor.wordWrapColumn` |

Toggle word wrap quickly with ++alt+z++.

---

## Code Lens

Code Lens displays actionable, contextual information inline with your code -- such as reference counts, test status, or recent changes. These appear as small text labels above functions and classes.

```json
{
  "editor.codeLens": true
}
```

!!! note
    Code Lens items are provided by language extensions through the LSP `textDocument/codeLens` protocol. Not all language servers support Code Lens.

---

## Peek Definition

Peek Definition lets you view the definition of a symbol inline, without leaving your current file.

- **Peek Definition**: ++alt+f12++
- **Peek References**: ++shift+f12++

A small embedded editor opens below the current line showing the definition. You can edit directly inside the peek window and close it with ++escape++.

---

## Go to Definition

Jump directly to the source definition of any symbol.

- **Go to Definition**: ++f12++ or ++ctrl+click++
- **Go to Type Definition**: available from the context menu
- **Go to Implementation**: ++ctrl+f12++

!!! tip
    Hold ++ctrl++ and hover over a symbol to see a preview of its definition. Click while holding ++ctrl++ to jump to it.

---

## Rename Symbol

Rename a symbol across your entire project with ++f2++. DSCode uses the language server to find all references and applies the rename consistently.

1. Place your cursor on the symbol you want to rename.
2. Press ++f2++.
3. Type the new name and press ++enter++.
4. All occurrences across files are updated.

!!! warning
    Rename Symbol accuracy depends on the language server. For languages without full LSP support, consider using find-and-replace with whole-word matching as a fallback.

---

## Format Document

Automatically format your entire document or a selected region according to the language's formatting rules.

| Action | Shortcut |
|---|---|
| Format Document | ++ctrl+shift+i++ |
| Format Selection | ++ctrl+k++ ++ctrl+f++ |

Formatting is provided by the language server or a dedicated formatter extension. You can also enable format-on-save:

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode"
}
```

---

## Snippets

Snippets are templates that insert commonly used code patterns. Type a snippet prefix and select it from the autocomplete list, then use ++tab++ to jump between placeholder fields.

### Built-in Snippets

Monaco includes built-in snippets for many languages. For example, in JavaScript:

- `log` inserts `console.log()`
- `for` inserts a `for` loop
- `if` inserts an `if` statement

### User Snippets

You can define custom snippets in your user or workspace configuration. Snippet files use the following format:

```json
{
  "Print to console": {
    "prefix": "log",
    "body": [
      "console.log('$1');",
      "$0"
    ],
    "description": "Log output to console"
  }
}
```

- `$1`, `$2`, etc. are tab stops.
- `$0` is the final cursor position.
- `${1:placeholder}` inserts a tab stop with a default value.

!!! tip
    Open the Command Palette with ++ctrl+shift+p++ and type `Snippets` to configure user snippets for a specific language.
