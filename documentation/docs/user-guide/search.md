# Search

DSCode provides powerful search capabilities at two levels: searching within the current file using Monaco's built-in find widget, and searching across all files in the workspace using the Rust `search_ops` backend powered by the `grep-regex` and `ignore` crates.

---

## Find in File

Open the in-file find widget with ++ctrl+f++ (or ++cmd+f++ on macOS). A search bar appears at the top-right of the editor.

### Basic Find

1. Press ++ctrl+f++ to open the find widget.
2. Type your search term.
3. Press ++enter++ to jump to the next match, or ++shift+enter++ for the previous match.
4. Press ++escape++ to close the find widget.

The match count is displayed on the right side of the search bar (e.g., "3 of 12").

### Find Options

Toggle these options using the buttons in the find widget or their keyboard shortcuts:

| Option | Button | Shortcut |
|---|---|---|
| Case Sensitive | `Aa` | ++alt+c++ |
| Whole Word | `Ab\|` | ++alt+w++ |
| Regular Expression | `.*` | ++alt+r++ |

!!! tip
    When you open the find widget with text selected, the selection is automatically used as the search term. If the selection spans multiple lines, "Find in Selection" is enabled automatically.

---

## Find and Replace

Open find-and-replace with ++ctrl+h++ (or ++cmd+h++ on macOS). This extends the find widget with a second input field for the replacement string.

### Replace Operations

| Action | Shortcut |
|---|---|
| Replace current match | ++ctrl+shift+1++ or click the replace button |
| Replace all matches | ++ctrl+alt+enter++ or click the replace-all button |

### Replacement Patterns with Regex

When regular expression mode is enabled, you can use capture groups in the replacement string:

| Pattern | Meaning |
|---|---|
| `$0` | Entire match |
| `$1`, `$2`, ... | Capture group 1, 2, etc. |
| `$&` | Entire match (alternative syntax) |
| `\n` | Newline |
| `\t` | Tab |

**Example**: Find `(\w+)\.log` and replace with `$1.debug` to rename all `.log` references to `.debug`.

---

## Regular Expression Search

DSCode's regex engine supports full Rust-flavored regular expressions (via the `grep-regex` crate for workspace search, and JavaScript RegExp for in-editor search).

### Common Regex Patterns

```
\d+          Match one or more digits
\bword\b     Match "word" as a whole word
foo|bar      Match "foo" or "bar"
^start       Match "start" at the beginning of a line
end$         Match "end" at the end of a line
colou?r      Match "color" or "colour"
\w+@\w+      Match email-like patterns
```

!!! note
    The regex flavor differs slightly between in-file search (JavaScript RegExp) and workspace search (Rust regex). Most common patterns work identically in both. Lookaheads and lookbehinds are supported by both engines.

---

## Find in Files (Workspace Search)

Search across all files in your workspace with ++ctrl+shift+f++ (or ++cmd+shift+f++ on macOS). This opens the **Search** view in the Sidebar, powered by `SearchView.svelte`.

### How It Works

The workspace search pipeline:

1. The `SearchView.svelte` component collects the query and options.
2. A Tauri IPC call invokes the `search_in_files` command in `search_ops.rs`.
3. The Rust backend uses `grep-regex` for pattern matching and the `ignore` crate for respecting `.gitignore` rules.
4. Results are streamed back and displayed in the search results panel.

### Search Options

The search input provides the same toggles as in-file search:

- **Case Sensitive** (++alt+c++)
- **Whole Word** (++alt+w++)
- **Regular Expression** (++alt+r++)

The `SearchOptions` struct in the Rust backend supports these fields:

```rust
pub struct SearchOptions {
    pub query: String,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub whole_word: bool,
    pub include_pattern: Option<String>,
    pub exclude_pattern: Option<String>,
    pub max_results: Option<usize>,
}
```

!!! warning
    By default, workspace search returns a maximum of **1,000 results** to maintain responsiveness. If your search term is very common, refine your query or use include/exclude patterns to narrow the scope.

---

## Include and Exclude Patterns

Below the search input in the Search view, you can specify glob patterns to include or exclude files.

### Include Patterns

Restrict the search to specific file types or directories:

```
*.ts                   Only TypeScript files
src/**/*.rs            Only Rust files under src/
*.{js,jsx,ts,tsx}      JavaScript and TypeScript files
docs/**                Only files under the docs directory
```

### Exclude Patterns

Skip specific files or directories:

```
node_modules/**        Skip node_modules
*.min.js               Skip minified files
**/dist/**             Skip build output
**/*.test.ts           Skip test files
```

!!! tip
    DSCode automatically respects `.gitignore` patterns during workspace search thanks to the `ignore` crate. Files listed in `.gitignore` are excluded from results without needing to specify them manually.

Toggle the include/exclude fields by clicking the ellipsis button (`...`) below the search input, or press ++ctrl+shift+j++ to toggle the detail fields.

---

## Search Results Panel

Search results are displayed in a tree structure grouped by file.

### Result Entry Format

Each result shows:

- **File path** -- relative to the workspace root
- **Line number** and **column** -- the exact position of the match
- **Matching line** -- with the matched text highlighted

The `SearchResult` structure returned by the backend:

```rust
pub struct SearchResult {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub r#match: String,
}
```

### Navigating Results

- Click a result to open the file at the matching line.
- Use ++f4++ to go to the next result and ++shift+f4++ for the previous result.
- Collapse or expand file groups by clicking the file header.

### Actions on Results

- **Dismiss a result**: Click the `x` icon next to a result to remove it from the list.
- **Dismiss a file group**: Click the `x` icon on the file header to remove all results for that file.
- **Clear all results**: Click the clear button at the top of the search results panel.

---

## Replace in Files

To replace across files, click the toggle arrow to the left of the search input (or press ++ctrl+shift+h++ / ++cmd+shift+h++ on macOS) to expand the replace field.

1. Enter your search term in the first input.
2. Enter your replacement text in the second input.
3. Review the results in the search results panel.
4. Use the inline replace buttons on individual results, or click **Replace All** to apply all replacements at once.

!!! warning
    **Replace All** across files modifies multiple files simultaneously. This operation cannot be undone with a single ++ctrl+z++. Consider committing your changes to Git before performing a large-scale replacement so you can revert if needed.

!!! tip
    Use the preview diff that appears when hovering over a result to verify the replacement before applying it. This helps you catch unintended changes, especially when using regular expressions.

---

## Search Shortcuts Reference

| Action | Shortcut |
|---|---|
| Find in file | ++ctrl+f++ |
| Find and replace in file | ++ctrl+h++ |
| Find in files (workspace) | ++ctrl+shift+f++ |
| Replace in files | ++ctrl+shift+h++ |
| Next match | ++enter++ or ++f4++ |
| Previous match | ++shift+enter++ or ++shift+f4++ |
| Toggle case sensitive | ++alt+c++ |
| Toggle whole word | ++alt+w++ |
| Toggle regex | ++alt+r++ |
| Toggle search details | ++ctrl+shift+j++ |
| Close find widget | ++escape++ |
