# Git Integration

DSCode includes built-in Git support powered by the [`git2`](https://docs.rs/git2/) Rust crate. All Git operations run natively in the Rust backend through the `git_ops` module -- no external `git` binary is required. The frontend presents Git state through the `GitView.svelte`, `DiffViewer.svelte`, `GitHistoryPanel.svelte`, and `BranchSwitcher.svelte` components.

---

## Source Control View

Open the Source Control (SCM) view by clicking the branch icon in the Activity Bar, or press ++ctrl+shift+g++.

The SCM view displays:

- **Current branch** name at the top
- **Ahead/behind indicators** showing how many commits you are ahead of or behind the remote tracking branch
- **Changed files** grouped into sections:
    - **Staged Changes** -- files added to the Git index, ready to commit
    - **Changes** -- modified, added, or deleted files not yet staged

Each file entry shows a status icon and the file path. Click a file to open a diff view.

---

## Initializing a Repository

If you open a folder that is not a Git repository, the SCM view offers an **Initialize Repository** button.

1. Open the Source Control view (++ctrl+shift+g++).
2. Click **Initialize Repository**.
3. DSCode calls `git_init` in the Rust backend, which runs `Repository::init()` from `git2`.

You can also initialize a repository from the Command Palette:

1. Press ++ctrl+shift+p++.
2. Type `Git: Initialize Repository`.
3. Press ++enter++.

!!! note
    Initializing creates a `.git` directory in the workspace root. If your workspace contains multiple folders, you will be prompted to select which folder to initialize.

---

## Staging Changes

Staging prepares files for the next commit. DSCode provides several ways to stage files:

| Action | How |
|---|---|
| Stage a single file | Click the `+` icon next to the file in the Changes list |
| Stage all changes | Click the `+` icon in the Changes section header |
| Stage selected lines | Open a diff, select lines, then right-click and choose **Stage Selected Ranges** |
| Unstage a file | Click the `-` icon next to the file in the Staged Changes list |

The backend `git_stage_file` command adds files to the index:

```rust
#[tauri::command]
pub async fn git_stage_file(repo_path: String, file_path: String) -> Result<(), String>
```

!!! tip
    You can stage and unstage individual hunks or lines for partial commits. Open the diff of a file, select the specific changes you want, and use the context menu to stage just those ranges.

---

## Committing

Once you have staged your changes, enter a commit message and commit:

1. Type your commit message in the text input at the top of the SCM view.
2. Press ++ctrl+enter++ (or ++cmd+enter++ on macOS) to commit.

Alternatively, use the Command Palette:

1. Press ++ctrl+shift+p++.
2. Type `Git: Commit`.
3. Press ++enter++.

The backend uses `git2`'s `Signature` and commit API:

```rust
fn get_signature(repo: &Repository) -> Result<Signature<'static>, GitError> {
    repo.signature().map_err(|_ | GitError::IdentityNotConfigured)
}
```

!!! warning
    If you have not configured your Git identity, the commit will fail with the error: *"Git identity not configured."* Run the following commands in the terminal to set your identity:

    ```bash
    git config --global user.name "Your Name"
    git config --global user.email "your@email.com"
    ```

---

## Viewing Diffs

Click any changed file in the SCM view to open a diff showing what has changed.

### Side-by-Side Diff

The default diff view (`DiffViewer.svelte`) shows the old version on the left and the new version on the right, with additions highlighted in green and deletions in red.

### Inline Diff

Switch to inline (unified) diff mode by clicking the inline diff toggle button in the diff editor toolbar. In this mode, additions and deletions are shown interleaved in a single editor pane.

### Diff Navigation

| Action | Shortcut |
|---|---|
| Next change | ++alt+f5++ |
| Previous change | ++alt+shift+f5++ |

!!! tip
    You can open a diff for any file, not just those with pending changes. Right-click a file in the Explorer and select **Compare with...** to diff against a branch, commit, or another file.

---

## Branch Management

The current branch is displayed in the Status Bar at the bottom of the window (`StatusBar.svelte`). Click it to open the `BranchSwitcher.svelte` component.

### Creating a Branch

1. Click the branch name in the Status Bar.
2. Select **Create new branch...** from the dropdown.
3. Type the branch name and press ++enter++.

Or via the Command Palette: ++ctrl+shift+p++ then type `Git: Create Branch`.

### Switching Branches

1. Click the branch name in the Status Bar.
2. Select the target branch from the list.
3. DSCode calls `git_checkout_branch` in the backend.

### Deleting a Branch

Use the Command Palette: ++ctrl+shift+p++ then type `Git: Delete Branch`.

!!! warning
    Branch names are validated by the Rust backend to prevent invalid characters. Branch names cannot contain spaces, `~`, `^`, `:`, `?`, `*`, `[`, or `\`, and cannot start with `-` or end with `.lock`.

---

## Merge Conflicts

!!! warning "Pull is fast-forward only"
    The built-in `git_pull` command only supports **fast-forward** merges. If the remote branch has diverged from your local branch, pull will return an error: *"Merge required -- not yet implemented."* In this case, use the integrated terminal to run `git pull --rebase` or `git merge` manually.

When a merge or rebase produces conflicts, DSCode highlights the conflicting files in the SCM view with a `C` (conflicted) status indicator. Note that merge conflict UI applies to conflicts produced by **external tools, rebases, or manual merges** run in the terminal -- the built-in pull command does not produce merge conflicts because it only performs fast-forward merges.

### Resolving Conflicts

1. Open the conflicted file. DSCode highlights the conflict markers:

    ```
    <<<<<<< HEAD
    your changes
    =======
    incoming changes
    >>>>>>> branch-name
    ```

2. Click the inline actions above each conflict block:
    - **Accept Current Change** -- keep your version
    - **Accept Incoming Change** -- use the incoming version
    - **Accept Both Changes** -- include both versions
    - **Compare Changes** -- open a three-way diff

3. After resolving all conflicts in a file, stage it to mark it as resolved.

4. Once all conflicts are resolved, commit the merge.

!!! tip
    Use ++ctrl+shift+m++ to open the Problems panel and see all files with unresolved conflicts at a glance.

---

## Git Status in the Gutter

The editor gutter (the narrow column to the left of line numbers) shows change indicators for the current file:

| Color | Meaning |
|---|---|
| **Green bar** | Added lines |
| **Blue bar** | Modified lines |
| **Red triangle** | Deleted lines (shown at the position of deletion) |

Click a gutter indicator to see an inline diff of that specific change, with options to revert the change.

---

## File Decorations

DSCode applies visual decorations to files in the Explorer and editor tabs to indicate their Git status:

| Decoration | Meaning |
|---|---|
| **M** (orange) | Modified |
| **U** (green) | Untracked (new file) |
| **D** (red) | Deleted |
| **R** (purple) | Renamed |
| **C** (red, bold) | Conflicted |

The `git-decorations.ts` module listens for Git status changes from the backend and applies the appropriate CSS classes and labels.

These decorations propagate up through the directory tree -- if a file deep inside a folder is modified, the parent folders also show a modification indicator.

```json
{
  "git.decorations.enabled": true
}
```

---

## Git Status Data

The Rust backend returns comprehensive status information through the `git_status` command:

```rust
pub struct GitStatus {
    pub branch: String,
    pub changes: Vec<GitChange>,
    pub ahead: usize,
    pub behind: usize,
}

pub struct GitChange {
    pub path: String,
    pub status: String,   // "added", "modified", "deleted", "renamed", "conflicted"
    pub staged: bool,
}
```

The frontend polls this command to keep the UI in sync. Change events are also emitted proactively when file operations occur.

---

## Git Shortcuts Reference

| Action | Shortcut |
|---|---|
| Open Source Control view | ++ctrl+shift+g++ |
| Commit staged changes | ++ctrl+enter++ (in SCM input) |
| Open changes (diff) | Click file in SCM view |
| Next change in diff | ++alt+f5++ |
| Previous change in diff | ++alt+shift+f5++ |
| Stage file | `+` icon in SCM view |
| Unstage file | `-` icon in SCM view |

!!! note
    DSCode's Git integration does not require a system `git` installation because it uses the `git2` Rust library directly. However, some advanced operations (such as interactive rebase or GPG signing) may still require the `git` CLI to be installed.
