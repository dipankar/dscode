/**
 * Extension Permission System
 *
 * Implements capability-based security for extensions.
 * Each extension declares its required permissions in manifest.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Permission {
    // File System
    FileSystemRead,
    FileSystemWrite,
    FileSystemDelete,

    // Workspace
    WorkspaceRead,
    WorkspaceWrite,
    WorkspaceExecute,

    // Network
    NetworkHttp,
    NetworkHttps,
    NetworkWebSocket,

    // System
    ClipboardRead,
    ClipboardWrite,

    // Secrets
    SecretsRead,
    SecretsWrite,

    // UI
    ShowNotifications,
    ShowDialogs,
    ShowQuickPick,

    // Editor
    TextEditorRead,
    TextEditorWrite,
    TextEditorSelection,

    // Debug
    DebugStart,
    DebugStop,
    DebugBreakpoints,

    // Terminal
    TerminalCreate,
    TerminalSend,

    // Tasks
    TasksExecute,

    // Commands
    CommandsExecute,

    // Authentication
    AuthenticationGetSession,
    AuthenticationCreateSession,
}

#[derive(Debug, Clone)]
pub struct ExtensionPermissions {
    extension_id: String,
    granted_permissions: HashSet<Permission>,
}

impl ExtensionPermissions {
    pub fn new(extension_id: String, permissions: Vec<Permission>) -> Self {
        Self {
            extension_id,
            granted_permissions: permissions.into_iter().collect(),
        }
    }

    pub fn from_manifest(extension_id: String, manifest_permissions: Option<Vec<String>>) -> Result<Self, String> {
        let permissions = match manifest_permissions {
            Some(perms) => {
                let mut granted = HashSet::new();
                for perm_str in perms {
                    let perm = Self::parse_permission(&perm_str)
                        .ok_or_else(|| format!("Invalid permission: {}", perm_str))?;
                    granted.insert(perm);
                }
                granted
            },
            None => HashSet::new(), // No permissions by default
        };

        Ok(Self {
            extension_id,
            granted_permissions: permissions,
        })
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.granted_permissions.contains(permission)
    }

    pub fn check_permission(&self, permission: &Permission) -> Result<(), String> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(format!(
                "Extension '{}' does not have permission: {:?}",
                self.extension_id, permission
            ))
        }
    }

    fn parse_permission(s: &str) -> Option<Permission> {
        match s {
            "fileSystem.read" => Some(Permission::FileSystemRead),
            "fileSystem.write" => Some(Permission::FileSystemWrite),
            "fileSystem.delete" => Some(Permission::FileSystemDelete),
            "workspace.read" => Some(Permission::WorkspaceRead),
            "workspace.write" => Some(Permission::WorkspaceWrite),
            "workspace.execute" => Some(Permission::WorkspaceExecute),
            "network.http" => Some(Permission::NetworkHttp),
            "network.https" => Some(Permission::NetworkHttps),
            "network.webSocket" => Some(Permission::NetworkWebSocket),
            "clipboard.read" => Some(Permission::ClipboardRead),
            "clipboard.write" => Some(Permission::ClipboardWrite),
            "secrets.read" => Some(Permission::SecretsRead),
            "secrets.write" => Some(Permission::SecretsWrite),
            "ui.notifications" => Some(Permission::ShowNotifications),
            "ui.dialogs" => Some(Permission::ShowDialogs),
            "ui.quickPick" => Some(Permission::ShowQuickPick),
            "textEditor.read" => Some(Permission::TextEditorRead),
            "textEditor.write" => Some(Permission::TextEditorWrite),
            "textEditor.selection" => Some(Permission::TextEditorSelection),
            "debug.start" => Some(Permission::DebugStart),
            "debug.stop" => Some(Permission::DebugStop),
            "debug.breakpoints" => Some(Permission::DebugBreakpoints),
            "terminal.create" => Some(Permission::TerminalCreate),
            "terminal.send" => Some(Permission::TerminalSend),
            "tasks.execute" => Some(Permission::TasksExecute),
            "commands.execute" => Some(Permission::CommandsExecute),
            "authentication.getSession" => Some(Permission::AuthenticationGetSession),
            "authentication.createSession" => Some(Permission::AuthenticationCreateSession),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_parsing() {
        let perms = ExtensionPermissions::from_manifest(
            "test.extension".to_string(),
            Some(vec![
                "fileSystem.read".to_string(),
                "fileSystem.write".to_string(),
            ])
        ).unwrap();

        assert!(perms.has_permission(&Permission::FileSystemRead));
        assert!(perms.has_permission(&Permission::FileSystemWrite));
        assert!(!perms.has_permission(&Permission::FileSystemDelete));
    }

    #[test]
    fn test_permission_check() {
        let perms = ExtensionPermissions::from_manifest(
            "test.extension".to_string(),
            Some(vec!["fileSystem.read".to_string()])
        ).unwrap();

        assert!(perms.check_permission(&Permission::FileSystemRead).is_ok());
        assert!(perms.check_permission(&Permission::FileSystemWrite).is_err());
    }
}
