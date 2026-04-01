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

    #[test]
    fn test_permission_serde_roundtrip() {
        // Verify each Permission variant serializes and deserializes correctly
        let all_perms = vec![
            Permission::FileSystemRead,
            Permission::FileSystemWrite,
            Permission::FileSystemDelete,
            Permission::WorkspaceRead,
            Permission::WorkspaceWrite,
            Permission::WorkspaceExecute,
            Permission::NetworkHttp,
            Permission::NetworkHttps,
            Permission::NetworkWebSocket,
            Permission::ClipboardRead,
            Permission::ClipboardWrite,
            Permission::SecretsRead,
            Permission::SecretsWrite,
            Permission::ShowNotifications,
            Permission::ShowDialogs,
            Permission::ShowQuickPick,
            Permission::TextEditorRead,
            Permission::TextEditorWrite,
            Permission::TextEditorSelection,
            Permission::DebugStart,
            Permission::DebugStop,
            Permission::DebugBreakpoints,
            Permission::TerminalCreate,
            Permission::TerminalSend,
            Permission::TasksExecute,
            Permission::CommandsExecute,
            Permission::AuthenticationGetSession,
            Permission::AuthenticationCreateSession,
        ];

        for perm in &all_perms {
            let json = serde_json::to_string(perm).unwrap();
            let deserialized: Permission = serde_json::from_str(&json).unwrap();
            assert_eq!(&deserialized, perm, "Roundtrip failed for {:?}", perm);
        }
    }

    #[test]
    fn test_permission_serde_uses_camel_case() {
        // Verify that serde rename_all = "camelCase" is applied
        let json = serde_json::to_string(&Permission::FileSystemRead).unwrap();
        assert!(
            json.contains("fileSystemRead"),
            "Expected camelCase 'fileSystemRead' in JSON: {}",
            json
        );

        let json = serde_json::to_string(&Permission::NetworkWebSocket).unwrap();
        assert!(
            json.contains("networkWebSocket"),
            "Expected camelCase 'networkWebSocket' in JSON: {}",
            json
        );

        let json = serde_json::to_string(&Permission::AuthenticationGetSession).unwrap();
        assert!(
            json.contains("authenticationGetSession"),
            "Expected camelCase 'authenticationGetSession' in JSON: {}",
            json
        );
    }

    #[test]
    fn test_permission_workspace_category() {
        let perms = ExtensionPermissions::from_manifest(
            "workspace.ext".to_string(),
            Some(vec![
                "workspace.read".to_string(),
                "workspace.write".to_string(),
                "workspace.execute".to_string(),
            ]),
        )
        .unwrap();
        assert!(perms.has_permission(&Permission::WorkspaceRead));
        assert!(perms.has_permission(&Permission::WorkspaceWrite));
        assert!(perms.has_permission(&Permission::WorkspaceExecute));
        assert!(!perms.has_permission(&Permission::FileSystemRead));
    }

    #[test]
    fn test_permission_clipboard_category() {
        let perms = ExtensionPermissions::from_manifest(
            "clipboard.ext".to_string(),
            Some(vec![
                "clipboard.read".to_string(),
                "clipboard.write".to_string(),
            ]),
        )
        .unwrap();
        assert!(perms.has_permission(&Permission::ClipboardRead));
        assert!(perms.has_permission(&Permission::ClipboardWrite));
        assert!(!perms.has_permission(&Permission::FileSystemRead));
    }

    #[test]
    fn test_permission_secrets_category() {
        let perms = ExtensionPermissions::from_manifest(
            "secrets.ext".to_string(),
            Some(vec![
                "secrets.read".to_string(),
                "secrets.write".to_string(),
            ]),
        )
        .unwrap();
        assert!(perms.has_permission(&Permission::SecretsRead));
        assert!(perms.has_permission(&Permission::SecretsWrite));
        assert!(!perms.has_permission(&Permission::FileSystemRead));
    }

    #[test]
    fn test_permission_text_editor_category() {
        let perms = ExtensionPermissions::from_manifest(
            "editor.ext".to_string(),
            Some(vec![
                "textEditor.read".to_string(),
                "textEditor.write".to_string(),
                "textEditor.selection".to_string(),
            ]),
        )
        .unwrap();
        assert!(perms.has_permission(&Permission::TextEditorRead));
        assert!(perms.has_permission(&Permission::TextEditorWrite));
        assert!(perms.has_permission(&Permission::TextEditorSelection));
        assert!(!perms.has_permission(&Permission::FileSystemRead));
    }

    #[test]
    fn test_permission_terminal_category() {
        let perms = ExtensionPermissions::from_manifest(
            "terminal.ext".to_string(),
            Some(vec![
                "terminal.create".to_string(),
                "terminal.send".to_string(),
            ]),
        )
        .unwrap();
        assert!(perms.has_permission(&Permission::TerminalCreate));
        assert!(perms.has_permission(&Permission::TerminalSend));
        assert!(!perms.has_permission(&Permission::FileSystemRead));
    }

    #[test]
    fn test_permission_tasks_and_commands_category() {
        let perms = ExtensionPermissions::from_manifest(
            "cmd.ext".to_string(),
            Some(vec![
                "tasks.execute".to_string(),
                "commands.execute".to_string(),
            ]),
        )
        .unwrap();
        assert!(perms.has_permission(&Permission::TasksExecute));
        assert!(perms.has_permission(&Permission::CommandsExecute));
        assert!(!perms.has_permission(&Permission::FileSystemRead));
    }

    #[test]
    fn test_permission_authentication_category() {
        let perms = ExtensionPermissions::from_manifest(
            "auth.ext".to_string(),
            Some(vec![
                "authentication.getSession".to_string(),
                "authentication.createSession".to_string(),
            ]),
        )
        .unwrap();
        assert!(perms.has_permission(&Permission::AuthenticationGetSession));
        assert!(perms.has_permission(&Permission::AuthenticationCreateSession));
        assert!(!perms.has_permission(&Permission::FileSystemRead));
    }

    #[test]
    fn test_permission_hash_equality() {
        // Permissions with the same variant should be equal and hash the same
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Permission::FileSystemRead);
        set.insert(Permission::FileSystemRead);
        assert_eq!(set.len(), 1, "Duplicate permissions should deduplicate in HashSet");
        set.insert(Permission::FileSystemWrite);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_extension_permissions_new_constructor() {
        let perms = ExtensionPermissions::new(
            "direct.ext".to_string(),
            vec![Permission::NetworkHttp, Permission::NetworkHttps],
        );
        assert!(perms.has_permission(&Permission::NetworkHttp));
        assert!(perms.has_permission(&Permission::NetworkHttps));
        assert!(!perms.has_permission(&Permission::NetworkWebSocket));
    }

    #[test]
    fn test_check_permission_error_message() {
        let perms = ExtensionPermissions::from_manifest(
            "my.ext".to_string(),
            Some(vec!["fileSystem.read".to_string()]),
        )
        .unwrap();

        let err = perms.check_permission(&Permission::FileSystemWrite).unwrap_err();
        assert!(err.contains("my.ext"), "Error should mention extension id");
        assert!(err.contains("FileSystemWrite"), "Error should mention the permission");
    }
}