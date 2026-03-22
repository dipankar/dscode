use serde::{Deserialize, Serialize};
use git2::{Repository, StatusOptions, Status, Signature, IndexAddOption, BranchType, DiffOptions};
use std::path::{Path, Component};
use thiserror::Error;

/// Git operation errors with proper context
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Not a git repository: {0}")]
    NotARepository(#[from] git2::Error),

    #[error("Invalid file path: {reason}")]
    InvalidPath { reason: String },

    #[error("Git identity not configured. Please run: git config --global user.name \"Your Name\" && git config --global user.email \"your@email.com\"")]
    IdentityNotConfigured,

    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

impl From<GitError> for String {
    fn from(err: GitError) -> Self {
        err.to_string()
    }
}

/// Validates that a file path is safe (relative, no traversal)
fn validate_file_path(file_path: &str) -> Result<(), GitError> {
    let path = Path::new(file_path);

    // Reject absolute paths
    if path.is_absolute() {
        return Err(GitError::InvalidPath {
            reason: "Absolute paths are not allowed".to_string(),
        });
    }

    // Reject paths with parent directory components (..)
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(GitError::InvalidPath {
                    reason: "Path traversal (..) is not allowed".to_string(),
                });
            }
            Component::Prefix(_) => {
                return Err(GitError::InvalidPath {
                    reason: "Windows path prefixes are not allowed".to_string(),
                });
            }
            _ => {}
        }
    }

    // Reject empty paths
    if file_path.is_empty() {
        return Err(GitError::InvalidPath {
            reason: "Empty path is not allowed".to_string(),
        });
    }

    Ok(())
}

/// Validates branch name to prevent injection
fn validate_branch_name(name: &str) -> Result<(), GitError> {
    // Git branch names cannot contain: space, ~, ^, :, ?, *, [, \, control chars
    // They also cannot start with - or end with .lock
    let invalid_chars = [' ', '~', '^', ':', '?', '*', '[', '\\', '\x7f'];

    if name.is_empty() {
        return Err(GitError::InvalidPath {
            reason: "Branch name cannot be empty".to_string(),
        });
    }

    if name.starts_with('-') {
        return Err(GitError::InvalidPath {
            reason: "Branch name cannot start with '-'".to_string(),
        });
    }

    if name.ends_with(".lock") {
        return Err(GitError::InvalidPath {
            reason: "Branch name cannot end with '.lock'".to_string(),
        });
    }

    if name.contains("..") {
        return Err(GitError::InvalidPath {
            reason: "Branch name cannot contain '..'".to_string(),
        });
    }

    for ch in invalid_chars.iter() {
        if name.contains(*ch) {
            return Err(GitError::InvalidPath {
                reason: format!("Branch name cannot contain '{}'", ch),
            });
        }
    }

    // Check for control characters
    if name.chars().any(|c| c.is_control()) {
        return Err(GitError::InvalidPath {
            reason: "Branch name cannot contain control characters".to_string(),
        });
    }

    Ok(())
}

/// Gets the git signature, returning an error if not configured
fn get_signature(repo: &Repository) -> Result<Signature<'static>, GitError> {
    repo.signature().map_err(|_| GitError::IdentityNotConfigured)
}

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
        let repo = Repository::discover(path).map_err(GitError::from)?;

        // Get current branch
        let head = repo.head().map_err(GitError::from)?;
        let branch = head
            .shorthand()
            .unwrap_or("HEAD")
            .to_string();

        // Get status of files
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.recurse_untracked_dirs(true);

        let statuses = repo.statuses(Some(&mut opts))
            .map_err(GitError::from)?;

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
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
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
        let repo = Repository::discover(path).map_err(GitError::from)?;

        // Get signature - fail if not configured (no silent fallback)
        let sig = get_signature(&repo)?;

        // Get the index and write it as a tree
        let mut index = repo.index().map_err(GitError::from)?;
        let tree_id = index.write_tree().map_err(GitError::from)?;
        let tree = repo.find_tree(tree_id).map_err(GitError::from)?;

        // Get the parent commit
        let parent_commit = match repo.head() {
            Ok(head) => {
                let oid = head.target().ok_or_else(|| GitError::OperationFailed("HEAD has no target".to_string()))?;
                Some(repo.find_commit(oid).map_err(GitError::from)?)
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
        .map_err(GitError::from)?;

        Ok(commit_oid.to_string())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

// Stage/Unstage operations
#[tauri::command]
pub async fn git_stage_file(repo_path: String, file_path: String) -> Result<(), String> {
    // Validate file path before any git operations
    validate_file_path(&file_path)?;

    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let mut index = repo.index().map_err(GitError::from)?;
        index.add_path(Path::new(&file_path))
            .map_err(GitError::from)?;
        index.write().map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_unstage_file(repo_path: String, file_path: String) -> Result<(), String> {
    // Validate file path before any git operations
    validate_file_path(&file_path)?;

    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let head = repo.head().map_err(GitError::from)?;
        let head_commit = head.peel_to_commit().map_err(GitError::from)?;

        repo.reset_default(Some(&head_commit.into_object()), &[Path::new(&file_path)])
            .map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_stage_all(repo_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let mut index = repo.index().map_err(GitError::from)?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .map_err(GitError::from)?;
        index.write().map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
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
    // Validate file path before any git operations
    validate_file_path(&file_path)?;

    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(&file_path);

        let diff = if staged {
            // Diff between HEAD and index (staged changes)
            let head = repo.head().map_err(GitError::from)?;
            let head_tree = head.peel_to_tree().map_err(GitError::from)?;
            repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))
        } else {
            // Diff between index and working directory (unstaged changes)
            repo.diff_index_to_workdir(None, Some(&mut diff_opts))
        }
        .map_err(GitError::from)?;

        let stats = diff.stats().map_err(GitError::from)?;

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            true
        }).map_err(GitError::from)?;

        Ok(FileDiff {
            old_path: file_path.clone(),
            new_path: file_path,
            diff_text,
            additions: stats.insertions(),
            deletions: stats.deletions(),
        })
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

// Remote operations
#[tauri::command]
pub async fn git_push(repo_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let head = repo.head().map_err(GitError::from)?;
        let branch_name = head.shorthand()
            .ok_or_else(|| GitError::OperationFailed("Failed to get branch name".to_string()))?
            .to_string();

        let mut remote = repo.find_remote("origin")
            .map_err(GitError::from)?;

        let refspec = format!("refs/heads/{}", branch_name);
        remote.push(&[&refspec], None)
            .map_err(GitError::from)?;

        Ok(format!("Pushed to origin/{}", branch_name))
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_pull(repo_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        // Fetch first
        let mut remote = repo.find_remote("origin")
            .map_err(GitError::from)?;

        remote.fetch(&[] as &[&str], None, None)
            .map_err(GitError::from)?;

        // Then merge
        let fetch_head = repo.find_reference("FETCH_HEAD")
            .map_err(GitError::from)?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)
            .map_err(GitError::from)?;

        let analysis = repo.merge_analysis(&[&fetch_commit])
            .map_err(GitError::from)?;

        if analysis.0.is_up_to_date() {
            Ok("Already up to date".to_string())
        } else if analysis.0.is_fast_forward() {
            // Fast-forward merge
            let refname = format!("refs/heads/{}", repo.head().map_err(GitError::from)?.shorthand().unwrap_or("master"));
            let mut reference = repo.find_reference(&refname)
                .map_err(GitError::from)?;
            reference.set_target(fetch_commit.id(), "Fast-forward merge")
                .map_err(GitError::from)?;
            repo.set_head(&refname)
                .map_err(GitError::from)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(GitError::from)?;
            Ok("Fast-forwarded".to_string())
        } else {
            Err(GitError::OperationFailed("Merge required - not yet implemented".to_string()).into())
        }
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_fetch(repo_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let mut remote = repo.find_remote("origin")
            .map_err(GitError::from)?;

        remote.fetch(&[] as &[&str], None, None)
            .map_err(GitError::from)?;

        Ok("Fetched from origin".to_string())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
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
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let mut branches = Vec::new();
        let current_branch = repo.head().ok().and_then(|h| h.shorthand().map(String::from));

        for branch in repo.branches(None).map_err(GitError::from)? {
            let (branch, branch_type) = branch.map_err(GitError::from)?;
            let name = branch.name().map_err(GitError::from)?
                .ok_or_else(|| GitError::OperationFailed("Branch name is not UTF-8".to_string()))?;

            branches.push(GitBranch {
                name: name.to_string(),
                is_head: current_branch.as_ref().map(|c| c == name).unwrap_or(false),
                is_remote: branch_type == BranchType::Remote,
            });
        }

        Ok(branches)
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_create_branch(repo_path: String, branch_name: String) -> Result<(), String> {
    // Validate branch name before any git operations
    validate_branch_name(&branch_name)?;

    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let head = repo.head().map_err(GitError::from)?;
        let head_commit = head.peel_to_commit().map_err(GitError::from)?;

        repo.branch(&branch_name, &head_commit, false)
            .map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_checkout_branch(repo_path: String, branch_name: String) -> Result<(), String> {
    // Validate branch name before any git operations
    validate_branch_name(&branch_name)?;

    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let (object, reference) = repo.revparse_ext(&branch_name)
            .map_err(GitError::from)?;

        repo.checkout_tree(&object, None)
            .map_err(GitError::from)?;

        match reference {
            Some(gref) => repo.set_head(gref.name().ok_or_else(|| GitError::OperationFailed("Invalid reference name".to_string()))?),
            None => repo.set_head_detached(object.id()),
        }
        .map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_delete_branch(repo_path: String, branch_name: String) -> Result<(), String> {
    // Validate branch name before any git operations
    validate_branch_name(&branch_name)?;

    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let mut branch = repo.find_branch(&branch_name, BranchType::Local)
            .map_err(GitError::from)?;

        branch.delete()
            .map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

// Stash operations
#[tauri::command]
pub async fn git_stash_save(repo_path: String, message: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let mut repo = Repository::discover(path).map_err(GitError::from)?;

        // Get signature - fail if not configured (no silent fallback)
        let sig = get_signature(&repo)?;

        let stash_id = repo.stash_save(
            &sig,
            message.as_deref().unwrap_or("WIP on stash"),
            Some(git2::StashFlags::DEFAULT)
        ).map_err(GitError::from)?;

        Ok(stash_id.to_string())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitStash {
    pub index: usize,
    pub message: String,
    pub oid: String,
}

#[tauri::command]
pub async fn git_stash_list(repo_path: String) -> Result<Vec<GitStash>, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let mut repo = Repository::discover(path).map_err(GitError::from)?;

        let mut stashes = Vec::new();
        repo.stash_foreach(|index, message, oid| {
            stashes.push(GitStash {
                index,
                message: message.to_string(),
                oid: oid.to_string(),
            });
            true
        }).map_err(GitError::from)?;

        Ok(stashes)
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_stash_pop(repo_path: String, index: usize) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let mut repo = Repository::discover(path).map_err(GitError::from)?;

        repo.stash_pop(index, None)
            .map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_stash_drop(repo_path: String, index: usize) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let mut repo = Repository::discover(path).map_err(GitError::from)?;

        repo.stash_drop(index)
            .map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

// Commit history operations
#[derive(Debug, Serialize, Deserialize)]
pub struct GitCommit {
    pub id: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: i64,
    pub parent_ids: Vec<String>,
}

#[tauri::command]
pub async fn git_log(repo_path: String, limit: Option<usize>) -> Result<Vec<GitCommit>, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let mut revwalk = repo.revwalk()
            .map_err(GitError::from)?;
        revwalk.push_head()
            .map_err(GitError::from)?;
        revwalk.set_sorting(git2::Sort::TIME)
            .map_err(GitError::from)?;

        let mut commits = Vec::new();
        let limit = limit.unwrap_or(100);

        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }

            let oid = oid.map_err(GitError::from)?;
            let commit = repo.find_commit(oid)
                .map_err(GitError::from)?;

            let author = commit.author();
            let parent_ids: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();

            commits.push(GitCommit {
                id: commit.id().to_string(),
                author: author.name().unwrap_or("Unknown").to_string(),
                email: author.email().unwrap_or("").to_string(),
                message: commit.message().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
                parent_ids,
            });
        }

        Ok(commits)
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

// Discard changes
#[tauri::command]
pub async fn git_discard_file(repo_path: String, file_path: String) -> Result<(), String> {
    // Validate file path before any git operations
    validate_file_path(&file_path)?;

    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let head = repo.head().map_err(GitError::from)?;
        let commit = head.peel_to_commit().map_err(GitError::from)?;
        let tree = commit.tree().map_err(GitError::from)?;

        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.path(&file_path);
        checkout_builder.force();

        repo.checkout_tree(tree.as_object(), Some(&mut checkout_builder))
            .map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[tauri::command]
pub async fn git_discard_all(repo_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);
        let repo = Repository::discover(path).map_err(GitError::from)?;

        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.force();
        checkout_builder.remove_untracked(true);

        repo.checkout_head(Some(&mut checkout_builder))
            .map_err(GitError::from)?;

        Ok(())
    })
    .await
    .map_err(|e| GitError::TaskFailed(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_path_valid() {
        assert!(validate_file_path("src/main.rs").is_ok());
        assert!(validate_file_path("file.txt").is_ok());
        assert!(validate_file_path("foo/bar/baz.rs").is_ok());
    }

    #[test]
    fn test_validate_file_path_absolute() {
        assert!(validate_file_path("/etc/passwd").is_err());
        assert!(validate_file_path("/home/user/file.txt").is_err());
    }

    #[test]
    fn test_validate_file_path_traversal() {
        assert!(validate_file_path("../secret.txt").is_err());
        assert!(validate_file_path("foo/../../../etc/passwd").is_err());
        assert!(validate_file_path("foo/bar/../../baz").is_err());
    }

    #[test]
    fn test_validate_file_path_empty() {
        assert!(validate_file_path("").is_err());
    }

    #[test]
    fn test_validate_branch_name_valid() {
        assert!(validate_branch_name("main").is_ok());
        assert!(validate_branch_name("feature/new-thing").is_ok());
        assert!(validate_branch_name("release-1.0").is_ok());
    }

    #[test]
    fn test_validate_branch_name_invalid() {
        assert!(validate_branch_name("").is_err());
        assert!(validate_branch_name("-starts-with-dash").is_err());
        assert!(validate_branch_name("branch.lock").is_err());
        assert!(validate_branch_name("branch..name").is_err());
        assert!(validate_branch_name("branch name").is_err());
        assert!(validate_branch_name("branch~name").is_err());
    }
}
