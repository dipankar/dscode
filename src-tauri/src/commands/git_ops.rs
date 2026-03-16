use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub changes: Vec<GitChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitChange {
    pub path: String,
    pub status: String,
}

#[tauri::command]
pub async fn git_status(repo_path: String) -> Result<GitStatus, String> {
    // TODO: Implement using git2 crate
    Ok(GitStatus {
        branch: "main".to_string(),
        changes: vec![],
    })
}
