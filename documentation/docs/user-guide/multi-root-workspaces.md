# Multi-Root Workspaces

DSCode supports **multi-root workspaces**, allowing you to work with multiple project folders in a single editor window. This is useful when you have related projects (e.g., a frontend and backend repository) that you want to navigate and search across simultaneously.

---

## Adding Workspace Folders

### From the Menu

1. Go to **File > Add Folder to Workspace...**
2. Select the folder you want to add.
3. The folder appears as a new root in the Explorer sidebar.

### From the Command Palette

1. Press ++ctrl+shift+p++ (++cmd+shift+p++ on macOS).
2. Type `Workspaces: Add Folder to Workspace`.
3. Select the folder to add.

### Programmatically (Extensions)

Extensions can add workspace folders using the `vscode.workspace.updateWorkspaceFolders()` API, which calls the `workspace_add_folder` Tauri command in the Rust backend.

```typescript
vscode.workspace.updateWorkspaceFolders(
    vscode.workspace.workspaceFolders?.length ?? 0,
    null,
    { uri: vscode.Uri.file('/path/to/new/folder') }
);
```

---

## Removing Workspace Folders

### From the Explorer

1. Right-click a root folder in the Explorer sidebar.
2. Select **Remove Folder from Workspace**.

### From the Command Palette

1. Press ++ctrl+shift+p++ (++cmd+shift+p++ on macOS).
2. Type `Workspaces: Remove Folder from Workspace`.
3. Select the folder to remove from the list.

!!! warning
    Removing a folder from the workspace does not delete any files. It only removes the folder from the current editor session.

---

## How It Works

### Backend Implementation

The Rust backend manages workspace folders through the `SessionManager`:

```rust
// session/workspace.rs
pub async fn add_workspace_folder(&self, path: PathBuf) -> Result<(), String>
pub async fn remove_workspace_folder(&self, path: &PathBuf) -> Result<(), String>
pub async fn get_workspace_folders_detailed(&self) -> Vec<WorkspaceFolder>
```

When a folder is added:

1. The backend checks for duplicates (folders already in the workspace are rejected).
2. The folder is added to the internal workspace state.
3. The folder is registered with the `PathValidator` for security validation.
4. A `WorkspaceFolderAdded` event is emitted, followed by a `StateChanged` event.
5. The frontend updates the Explorer tree to show the new root.

When a folder is removed:

1. The folder is removed from the internal workspace state.
2. A `WorkspaceFolderRemoved` event is emitted.
3. The frontend removes the folder from the Explorer tree.

---

## Settings Scoping

In a multi-root workspace, settings can be scoped to individual folders or applied globally across the workspace.

### Scope Hierarchy

| Scope | Location | Applies To |
|-------|----------|-----------|
| **User** | `~/.config/dscode/settings.json` | All workspaces |
| **Workspace** | `*.code-workspace` file | Current workspace session |
| **Folder** | `.vscode/settings.json` in each folder | Files within that folder |

Folder-level settings override workspace settings, which override user settings. This allows each project in a multi-root workspace to maintain its own editor configuration.

### Example

```json title="frontend/.vscode/settings.json"
{
    "editor.tabSize": 2,
    "editor.defaultFormatter": "esbenp.prettier-vscode"
}
```

```json title="backend/.vscode/settings.json"
{
    "editor.tabSize": 4,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
}
```

With this configuration, TypeScript files in the `frontend/` folder use 2-space indentation with Prettier, while Rust files in the `backend/` folder use 4-space indentation with rust-analyzer formatting.

---

## Extension Workspace Folder Events

Extensions are notified when workspace folders change through the `onDidChangeWorkspaceFolders` event:

```typescript
vscode.workspace.onDidChangeWorkspaceFolders(event => {
    for (const added of event.added) {
        console.log(`Folder added: ${added.uri.fsPath}`);
    }
    for (const removed of event.removed) {
        console.log(`Folder removed: ${removed.uri.fsPath}`);
    }
});
```

This event fires when:

- A folder is added to the workspace
- A folder is removed from the workspace
- The workspace file is saved with folder changes

---

## Search Across Folders

When you use **Find in Files** (++ctrl+shift+f++), DSCode searches across all workspace folders by default. You can restrict the search to specific folders using include patterns:

```
./frontend/**    # Search only in the frontend folder
./backend/src/** # Search only in backend/src
```

---

## File Decorations

File decorations (Git status, diagnostics) work independently per workspace folder. Each folder tracks its own Git repository status, so you can see the modification state of files across multiple repositories simultaneously.

---

## Workspace File

Multi-root workspaces can be saved as a `.code-workspace` file for easy reopening:

```json title="myproject.code-workspace"
{
    "folders": [
        { "path": "./frontend" },
        { "path": "./backend" },
        { "path": "./shared-libs" }
    ],
    "settings": {
        "editor.fontSize": 14
    }
}
```

Open a workspace file with:

```bash
dscode myproject.code-workspace
```

---

## See Also

- [Settings and Configuration](settings-and-configuration.md) -- settings scopes and precedence
- [Git Integration](git-integration.md) -- source control across workspace folders
- [Search](search.md) -- searching across multiple folders
