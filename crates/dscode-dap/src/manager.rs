use crate::types::{Breakpoint, DebugSession, DebugState, SourceBreakpoint};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;
use uuid::Uuid;

pub struct DebugManager {
    sessions: Arc<Mutex<HashMap<String, DebugSession>>>,
    breakpoints: Arc<Mutex<HashMap<String, Vec<Breakpoint>>>>,
}

impl Default for DebugManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugManager {
    pub fn new() -> Self {
        DebugManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_session(&self, name: String, adapter_type: String) -> Result<String, String> {
        let session_id = Uuid::new_v4().to_string();
        let session = DebugSession {
            id: session_id.clone(),
            name,
            state: DebugState::Stopped,
            adapter_type,
        };

        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(session_id.clone(), session);

        info!(id = %session_id, "Created debug session");
        Ok(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Result<DebugSession, String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| format!("Session {} not found", session_id))
    }

    pub fn list_sessions(&self) -> Result<Vec<DebugSession>, String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        Ok(sessions.values().cloned().collect())
    }

    pub fn update_session_state(&self, session_id: &str, state: DebugState) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = state;
            info!(id = %session_id, "Updated session state");
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    pub fn terminate_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.remove(session_id);
        info!(id = %session_id, "Terminated debug session");
        Ok(())
    }

    pub fn set_breakpoints(
        &self,
        file_path: String,
        breakpoints: Vec<SourceBreakpoint>,
    ) -> Result<(), String> {
        // Convert SourceBreakpoint to Breakpoint
        let converted_bps: Vec<Breakpoint> = breakpoints
            .into_iter()
            .map(|sbp| Breakpoint {
                id: None,
                verified: false,
                message: None,
                source: None,
                line: Some(sbp.line),
                column: sbp.column,
            })
            .collect();

        let mut bps = self.breakpoints.lock().map_err(|e| e.to_string())?;
        bps.insert(file_path.clone(), converted_bps);
        info!(path = %file_path, "Set breakpoints");
        Ok(())
    }

    pub fn get_breakpoints(&self, file_path: &str) -> Result<Vec<Breakpoint>, String> {
        let bps = self.breakpoints.lock().map_err(|e| e.to_string())?;
        Ok(bps.get(file_path).cloned().unwrap_or_default())
    }

    pub fn clear_breakpoints(&self, file_path: &str) -> Result<(), String> {
        let mut bps = self.breakpoints.lock().map_err(|e| e.to_string())?;
        bps.remove(file_path);
        info!(path = %file_path, "Cleared breakpoints");
        Ok(())
    }

    pub fn get_all_breakpoints(&self) -> Result<HashMap<String, Vec<Breakpoint>>, String> {
        let bps = self.breakpoints.lock().map_err(|e| e.to_string())?;
        Ok(bps.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DebugState;

    #[test]
    fn test_debug_manager_new() {
        let manager = DebugManager::new();
        // A new manager should have no sessions
        let sessions = manager.list_sessions().unwrap();
        assert!(sessions.is_empty(), "New manager should have no sessions");
    }

    #[test]
    fn test_debug_manager_create_session() {
        let manager = DebugManager::new();
        let session_id = manager
            .create_session("Test Session".to_string(), "cppdbg".to_string())
            .unwrap();
        assert!(!session_id.is_empty(), "Session ID should not be empty");
    }

    #[test]
    fn test_debug_manager_get_session() {
        let manager = DebugManager::new();
        let session_id = manager
            .create_session("My Debug".to_string(), "python".to_string())
            .unwrap();

        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.id, session_id);
        assert_eq!(session.name, "My Debug");
        assert_eq!(session.adapter_type, "python");
        assert_eq!(session.state, DebugState::Stopped);
    }

    #[test]
    fn test_debug_manager_get_nonexistent_session() {
        let manager = DebugManager::new();
        let result = manager.get_session("nonexistent-id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_debug_manager_list_sessions() {
        let manager = DebugManager::new();

        // Initially empty
        assert!(manager.list_sessions().unwrap().is_empty());

        // Create multiple sessions
        let id1 = manager
            .create_session("S1".to_string(), "cppdbg".to_string())
            .unwrap();
        let id2 = manager
            .create_session("S2".to_string(), "python".to_string())
            .unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);

        let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&id1.as_str()));
        assert!(ids.contains(&id2.as_str()));
    }

    #[test]
    fn test_debug_manager_update_session_state() {
        let manager = DebugManager::new();
        let session_id = manager
            .create_session("State Test".to_string(), "go".to_string())
            .unwrap();

        // Initially Stopped
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.state, DebugState::Stopped);

        // Update to Running
        manager
            .update_session_state(&session_id, DebugState::Running)
            .unwrap();
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.state, DebugState::Running);

        // Update to Paused
        manager
            .update_session_state(&session_id, DebugState::Paused)
            .unwrap();
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.state, DebugState::Paused);

        // Update to Terminated
        manager
            .update_session_state(&session_id, DebugState::Terminated)
            .unwrap();
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.state, DebugState::Terminated);
    }

    #[test]
    fn test_debug_manager_update_nonexistent_session_state() {
        let manager = DebugManager::new();
        let result = manager.update_session_state("no-such-id", DebugState::Running);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_debug_manager_terminate_session() {
        let manager = DebugManager::new();
        let session_id = manager
            .create_session("To Terminate".to_string(), "rust".to_string())
            .unwrap();

        // Verify session exists
        assert!(manager.get_session(&session_id).is_ok());

        // Terminate it
        manager.terminate_session(&session_id).unwrap();

        // Verify it's gone
        assert!(manager.get_session(&session_id).is_err());
    }

    #[test]
    fn test_debug_manager_terminate_nonexistent_session() {
        let manager = DebugManager::new();
        // terminate_session on a nonexistent ID is a no-op (HashMap::remove)
        let result = manager.terminate_session("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_debug_manager_set_and_get_breakpoints() {
        let manager = DebugManager::new();

        let breakpoints = vec![SourceBreakpoint {
            line: 10,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: None,
        }];

        manager
            .set_breakpoints("/src/main.rs".to_string(), breakpoints)
            .unwrap();

        let result = manager.get_breakpoints("/src/main.rs").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(10));
        assert!(!result[0].verified);
    }

    #[test]
    fn test_debug_manager_get_breakpoints_missing_file() {
        let manager = DebugManager::new();
        let result = manager.get_breakpoints("/nonexistent/file.rs").unwrap();
        assert!(
            result.is_empty(),
            "Missing file should return empty breakpoints"
        );
    }

    #[test]
    fn test_debug_manager_clear_breakpoints() {
        let manager = DebugManager::new();

        let breakpoints = vec![SourceBreakpoint {
            line: 5,
            column: Some(2),
            condition: Some("x > 0".to_string()),
            hit_condition: None,
            log_message: None,
        }];

        manager
            .set_breakpoints("/app.py".to_string(), breakpoints)
            .unwrap();
        assert_eq!(manager.get_breakpoints("/app.py").unwrap().len(), 1);

        manager.clear_breakpoints("/app.py").unwrap();
        assert!(manager.get_breakpoints("/app.py").unwrap().is_empty());
    }

    #[test]
    fn test_debug_manager_get_all_breakpoints() {
        let manager = DebugManager::new();

        assert!(manager.get_all_breakpoints().unwrap().is_empty());

        manager
            .set_breakpoints(
                "/a.rs".to_string(),
                vec![SourceBreakpoint {
                    line: 1,
                    column: None,
                    condition: None,
                    hit_condition: None,
                    log_message: None,
                }],
            )
            .unwrap();

        manager
            .set_breakpoints(
                "/b.rs".to_string(),
                vec![SourceBreakpoint {
                    line: 2,
                    column: None,
                    condition: None,
                    hit_condition: None,
                    log_message: None,
                }],
            )
            .unwrap();

        let all = manager.get_all_breakpoints().unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.contains_key("/a.rs"));
        assert!(all.contains_key("/b.rs"));
    }

    #[test]
    fn test_debug_manager_set_breakpoints_overwrites() {
        let manager = DebugManager::new();

        let bp1 = vec![SourceBreakpoint {
            line: 1,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: None,
        }];
        let bp2 = vec![
            SourceBreakpoint {
                line: 10,
                column: None,
                condition: None,
                hit_condition: None,
                log_message: None,
            },
            SourceBreakpoint {
                line: 20,
                column: Some(4),
                condition: None,
                hit_condition: None,
                log_message: None,
            },
        ];

        manager
            .set_breakpoints("/file.rs".to_string(), bp1)
            .unwrap();
        manager
            .set_breakpoints("/file.rs".to_string(), bp2)
            .unwrap();

        let result = manager.get_breakpoints("/file.rs").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_debug_manager_multiple_sessions_independent() {
        let manager = DebugManager::new();

        let id1 = manager
            .create_session("S1".to_string(), "cppdbg".to_string())
            .unwrap();
        let id2 = manager
            .create_session("S2".to_string(), "python".to_string())
            .unwrap();

        manager
            .update_session_state(&id1, DebugState::Running)
            .unwrap();
        manager
            .update_session_state(&id2, DebugState::Paused)
            .unwrap();

        let s1 = manager.get_session(&id1).unwrap();
        let s2 = manager.get_session(&id2).unwrap();
        assert_eq!(s1.state, DebugState::Running);
        assert_eq!(s2.state, DebugState::Paused);

        // Terminating one should not affect the other
        manager.terminate_session(&id1).unwrap();
        assert!(manager.get_session(&id1).is_err());
        assert!(manager.get_session(&id2).is_ok());
    }
}
