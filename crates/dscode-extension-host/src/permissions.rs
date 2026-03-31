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
        Self { extension_id, granted_permissions: permissions.into_iter().collect() }
    }

    pub fn from_manifest(
        extension_id: String, manifest_permissions: Option<Vec<String>>,
    ) -> Result<Self, String> {
        let permissions = match manifest_permissions {
            Some(perms) => {
                let mut granted = HashSet::new();
                for perm_str in perms {
                    let perm = Self::parse_permission(&perm_str)
                        .ok_or_else(|| format!("Invalid permission: {}", perm_str))?;
                    granted.insert(perm);
                }
                granted
            }
            None => HashSet::new(), // No permissions by default
        };

        Ok(Self { extension_id, granted_permissions: permissions })
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
            Some(vec!["fileSystem.read".to_string(), "fileSystem.write".to_string()]),
        )
        .unwrap();

        assert!(perms.has_permission(&Permission::FileSystemRead));
        assert!(perms.has_permission(&Permission::FileSystemWrite));
        assert!(!perms.has_permission(&Permission::FileSystemDelete));
    }

    #[test]
    fn test_permission_check() {
        let perms = ExtensionPermissions::from_manifest(
            "test.extension".to_string(),
            Some(vec!["fileSystem.read".to_string()]),
        )
        .unwrap();

        assert!(perms.check_permission(&Permission::FileSystemRead).is_ok());
        assert!(perms.check_permission(&Permission::FileSystemWrite).is_err());
    }

    #[test]
    fn test_extension_permissions_grants() {
        // Test file system category
        let fs_perms = ExtensionPermissions::from_manifest(
            "fs.ext".to_string(),
            Some(vec![
                "fileSystem.read".to_string(),
                "fileSystem.write".to_string(),
                "fileSystem.delete".to_string(),
            ]),
        )
        .unwrap();
        assert!(fs_perms.has_permission(&Permission::FileSystemRead));
        assert!(fs_perms.has_permission(&Permission::FileSystemWrite));
        assert!(fs_perms.has_permission(&Permission::FileSystemDelete));
        assert!(!fs_perms.has_permission(&Permission::NetworkHttp));

        // Test network category
        let net_perms = ExtensionPermissions::from_manifest(
            "net.ext".to_string(),
            Some(vec![
                "network.http".to_string(),
                "network.https".to_string(),
                "network.webSocket".to_string(),
            ]),
        )
        .unwrap();
        assert!(net_perms.has_permission(&Permission::NetworkHttp));
        assert!(net_perms.has_permission(&Permission::NetworkHttps));
        assert!(net_perms.has_permission(&Permission::NetworkWebSocket));
        assert!(!net_perms.has_permission(&Permission::FileSystemRead));

        // Test ui category
        let ui_perms = ExtensionPermissions::from_manifest(
            "ui.ext".to_string(),
            Some(vec![
                "ui.notifications".to_string(),
                "ui.dialogs".to_string(),
                "ui.quickPick".to_string(),
            ]),
        )
        .unwrap();
        assert!(ui_perms.has_permission(&Permission::ShowNotifications));
        assert!(ui_perms.has_permission(&Permission::ShowDialogs));
        assert!(ui_perms.has_permission(&Permission::ShowQuickPick));
        assert!(!ui_perms.has_permission(&Permission::FileSystemRead));

        // Test debug category
        let dbg_perms = ExtensionPermissions::from_manifest(
            "dbg.ext".to_string(),
            Some(vec![
                "debug.start".to_string(),
                "debug.stop".to_string(),
                "debug.breakpoints".to_string(),
            ]),
        )
        .unwrap();
        assert!(dbg_perms.has_permission(&Permission::DebugStart));
        assert!(dbg_perms.has_permission(&Permission::DebugStop));
        assert!(dbg_perms.has_permission(&Permission::DebugBreakpoints));
        assert!(!dbg_perms.has_permission(&Permission::FileSystemRead));
    }

    #[test]
    fn test_extension_permissions_default() {
        // No permissions granted when manifest has None
        let no_perms = ExtensionPermissions::from_manifest(
            "default.ext".to_string(),
            None,
        )
        .unwrap();
        assert!(!no_perms.has_permission(&Permission::FileSystemRead));
        assert!(!no_perms.has_permission(&Permission::NetworkHttp));
        assert!(!no_perms.has_permission(&Permission::ShowNotifications));
        assert!(!no_perms.has_permission(&Permission::DebugStart));

        // No permissions granted when manifest has empty list
        let empty_perms = ExtensionPermissions::from_manifest(
            "empty.ext".to_string(),
            Some(vec![]),
        )
        .unwrap();
        assert!(!empty_perms.has_permission(&Permission::FileSystemRead));

        // Invalid permission string should fail
        let bad_result = ExtensionPermissions::from_manifest(
            "bad.ext".to_string(),
            Some(vec!["invalid.permission".to_string()]),
        );
        assert!(bad_result.is_err());
    }
}