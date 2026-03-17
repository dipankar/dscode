use serde::{Deserialize, Serialize};
use git2::{Repository, StatusOptions, Status, Signature, IndexAddOption, BranchType, DiffOptions};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub changes: Vec<GitChange>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitChange {
    pub path: String,
    pub status: String,
    pub staged: bool,
}

#[tauri::command]
pub async fn git_status(repo_path: String) -> Result<GitStatus, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        // Get current branch
        let head = repo.head().map_err(|e| e.to_string())?;
        let branch = head
            .shorthand()
            .unwrap_or("HEAD")
            .to_string();

        // Get status of files
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.recurse_untracked_dirs(true);

        let statuses = repo.statuses(Some(&mut opts))
            .map_err(|e| e.to_string())?;

        let mut changes = Vec::new();
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("").to_string();

            let status_str = get_status_string(status);
            let staged = is_staged(status);

            if !status_str.is_empty() {
                changes.push(GitChange {
                    path,
                    status: status_str,
                    staged,
                });
            }
        }

        // Get ahead/behind count
        let (ahead, behind) = get_ahead_behind(&repo, &branch);

        Ok(GitStatus {
            branch,
            changes,
            ahead,
            behind,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

fn get_status_string(status: Status) -> String {
    if status.contains(Status::INDEX_NEW) || status.contains(Status::WT_NEW) {
        "added".to_string()
    } else if status.contains(Status::INDEX_MODIFIED) || status.contains(Status::WT_MODIFIED) {
        "modified".to_string()
    } else if status.contains(Status::INDEX_DELETED) || status.contains(Status::WT_DELETED) {
        "deleted".to_string()
    } else if status.contains(Status::INDEX_RENAMED) || status.contains(Status::WT_RENAMED) {
        "renamed".to_string()
    } else if status.contains(Status::CONFLICTED) {
        "conflicted".to_string()
    } else {
        String::new()
    }
}

fn is_staged(status: Status) -> bool {
    status.contains(Status::INDEX_NEW)
        || status.contains(Status::INDEX_MODIFIED)
        || status.contains(Status::INDEX_DELETED)
        || status.contains(Status::INDEX_RENAMED)
        || status.contains(Status::INDEX_TYPECHANGE)
}

fn get_ahead_behind(repo: &Repository, branch_name: &str) -> (usize, usize) {
    let local_branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(b) => b,
        Err(_) => return (0, 0),
    };

    let upstream = match local_branch.upstream() {
        Ok(u) => u,
        Err(_) => return (0, 0),
    };

    let local_oid = match local_branch.get().target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    let upstream_oid = match upstream.get().target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    match repo.graph_ahead_behind(local_oid, upstream_oid) {
        Ok((ahead, behind)) => (ahead, behind),
        Err(_) => (0, 0),
    }
}

// Commit operations
#[tauri::command]
pub async fn git_commit(repo_path: String, message: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        // Get signature from config or use default
        let sig = repo.signature()
            .or_else(|_| Signature::now("DSCode User", "user@dscode.local"))
            .map_err(|e| format!("Failed to create signature: {}", e))?;

        // Get the index and write it as a tree
        let mut index = repo.index().map_err(|e| format!("Failed to get index: {}", e))?;
        let tree_id = index.write_tree().map_err(|e| format!("Failed to write tree: {}", e))?;
        let tree = repo.find_tree(tree_id).map_err(|e| format!("Failed to find tree: {}", e))?;

        // Get the parent commit
        let parent_commit = match repo.head() {
            Ok(head) => {
                let oid = head.target().ok_or("HEAD has no target")?;
                Some(repo.find_commit(oid).map_err(|e| format!("Failed to find commit: {}", e))?)
            }
            Err(_) => None,
        };

        // Create the commit
        let commit_oid = if let Some(parent) = parent_commit {
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &message,
                &tree,
                &[&parent],
            )
        } else {
            // Initial commit
            repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &[])
        }
        .map_err(|e| format!("Failed to create commit: {}", e))?;

        Ok(commit_oid.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// Stage/Unstage operations
#[tauri::command]
pub async fn git_stage_file(repo_path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let mut index = repo.index().map_err(|e| format!("Failed to get index: {}", e))?;
        index.add_path(Path::new(&file_path))
            .map_err(|e| format!("Failed to stage file: {}", e))?;
        index.write().map_err(|e| format!("Failed to write index: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn git_unstage_file(repo_path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
        let head_commit = head.peel_to_commit().map_err(|e| format!("Failed to get commit: {}", e))?;

        repo.reset_default(Some(&head_commit.into_object()), &[Path::new(&file_path)])
            .map_err(|e| format!("Failed to unstage file: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn git_stage_all(repo_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let mut index = repo.index().map_err(|e| format!("Failed to get index: {}", e))?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .map_err(|e| format!("Failed to stage all: {}", e))?;
        index.write().map_err(|e| format!("Failed to write index: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// Diff operations
#[derive(Debug, Serialize, Deserialize)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub diff_text: String,
    pub additions: usize,
    pub deletions: usize,
}

#[tauri::command]
pub async fn git_get_diff(repo_path: String, file_path: String, staged: bool) -> Result<FileDiff, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(&file_path);

        let diff = if staged {
            // Diff between HEAD and index (staged changes)
            let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
            let head_tree = head.peel_to_tree().map_err(|e| format!("Failed to get tree: {}", e))?;
            repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))
        } else {
            // Diff between index and working directory (unstaged changes)
            repo.diff_index_to_workdir(None, Some(&mut diff_opts))
        }
        .map_err(|e| format!("Failed to get diff: {}", e))?;

        let stats = diff.stats().map_err(|e| format!("Failed to get stats: {}", e))?;

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            true
        }).map_err(|e| format!("Failed to print diff: {}", e))?;

        Ok(FileDiff {
            old_path: file_path.clone(),
            new_path: file_path,
            diff_text,
            additions: stats.insertions(),
            deletions: stats.deletions(),
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// Remote operations
#[tauri::command]
pub async fn git_push(repo_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
        let branch_name = head.shorthand().ok_or("Failed to get branch name")?.to_string();

        let mut remote = repo.find_remote("origin")
            .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;

        let refspec = format!("refs/heads/{}", branch_name);
        remote.push(&[&refspec], None)
            .map_err(|e| format!("Failed to push: {}", e))?;

        Ok(format!("Pushed to origin/{}", branch_name))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn git_pull(repo_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        // Fetch first
        let mut remote = repo.find_remote("origin")
            .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;

        remote.fetch(&[] as &[&str], None, None)
            .map_err(|e| format!("Failed to fetch: {}", e))?;

        // Then merge
        let fetch_head = repo.find_reference("FETCH_HEAD")
            .map_err(|e| format!("Failed to find FETCH_HEAD: {}", e))?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)
            .map_err(|e| format!("Failed to get annotated commit: {}", e))?;

        let analysis = repo.merge_analysis(&[&fetch_commit])
            .map_err(|e| format!("Failed to analyze merge: {}", e))?;

        if analysis.0.is_up_to_date() {
            Ok("Already up to date".to_string())
        } else if analysis.0.is_fast_forward() {
            // Fast-forward merge
            let refname = format!("refs/heads/{}", repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?.shorthand().unwrap_or("master"));
            let mut reference = repo.find_reference(&refname)
                .map_err(|e| format!("Failed to find reference: {}", e))?;
            reference.set_target(fetch_commit.id(), "Fast-forward merge")
                .map_err(|e| format!("Failed to set target: {}", e))?;
            repo.set_head(&refname)
                .map_err(|e| format!("Failed to set HEAD: {}", e))?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| format!("Failed to checkout: {}", e))?;
            Ok("Fast-forwarded".to_string())
        } else {
            Err("Merge required - not yet implemented".to_string())
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn git_fetch(repo_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let mut remote = repo.find_remote("origin")
            .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;

        remote.fetch(&[] as &[&str], None, None)
            .map_err(|e| format!("Failed to fetch: {}", e))?;

        Ok("Fetched from origin".to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// Branch operations
#[derive(Debug, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
}

#[tauri::command]
pub async fn git_list_branches(repo_path: String) -> Result<Vec<GitBranch>, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let mut branches = Vec::new();
        let current_branch = repo.head().ok().and_then(|h| h.shorthand().map(String::from));

        for branch in repo.branches(None).map_err(|e| format!("Failed to list branches: {}", e))? {
            let (branch, branch_type) = branch.map_err(|e| format!("Failed to get branch: {}", e))?;
            let name = branch.name().map_err(|e| format!("Invalid branch name: {}", e))?
                .ok_or("Branch name is not UTF-8")?;

            branches.push(GitBranch {
                name: name.to_string(),
                is_head: current_branch.as_ref().map(|c| c == name).unwrap_or(false),
                is_remote: branch_type == BranchType::Remote,
            });
        }

        Ok(branches)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn git_create_branch(repo_path: String, branch_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
        let head_commit = head.peel_to_commit().map_err(|e| format!("Failed to get commit: {}", e))?;

        repo.branch(&branch_name, &head_commit, false)
            .map_err(|e| format!("Failed to create branch: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn git_checkout_branch(repo_path: String, branch_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let (object, reference) = repo.revparse_ext(&branch_name)
            .map_err(|e| format!("Failed to find branch: {}", e))?;

        repo.checkout_tree(&object, None)
            .map_err(|e| format!("Failed to checkout tree: {}", e))?;

        match reference {
            Some(gref) => repo.set_head(gref.name().ok_or("Invalid reference name")?),
            None => repo.set_head_detached(object.id()),
        }
        .map_err(|e| format!("Failed to set HEAD: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn git_delete_branch(repo_path: String, branch_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(|e| format!("Not a git repository: {}", e))?;

        let mut branch = repo.find_branch(&branch_name, BranchType::Local)
            .map_err(|e| format!("Failed to find branch: {}", e))?;

        branch.delete()
            .map_err(|e| format!("Failed to delete branch: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
