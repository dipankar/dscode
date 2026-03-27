use super::types::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct DebugManager {
    sessions: Arc<Mutex<HashMap<String, DebugSession>>>,
    breakpoints: Arc<Mutex<HashMap<String, Vec<Breakpoint>>>>,
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
        let session =
            DebugSession { id: session_id.clone(), name, state: DebugState::Stopped, adapter_type };

        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(session_id.clone(), session);

        println!("[Debug] Created session: {}", session_id);
        Ok(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Result<DebugSession, String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.get(session_id).cloned().ok_or_else(|| format!("Session {} not found", session_id))
    }

    pub fn list_sessions(&self) -> Result<Vec<DebugSession>, String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        Ok(sessions.values().cloned().collect())
    }

    pub fn update_session_state(&self, session_id: &str, state: DebugState) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = state;
            println!("[Debug] Updated session {} state", session_id);
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    pub fn terminate_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.remove(session_id);
        println!("[Debug] Terminated session: {}", session_id);
        Ok(())
    }

    pub fn set_breakpoints(
        &self, file_path: String, breakpoints: Vec<SourceBreakpoint>,
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
        println!("[Debug] Set breakpoints for {}", file_path);
        Ok(())
    }

    pub fn get_breakpoints(&self, file_path: &str) -> Result<Vec<Breakpoint>, String> {
        let bps = self.breakpoints.lock().map_err(|e| e.to_string())?;
        Ok(bps.get(file_path).cloned().unwrap_or_default())
    }

    pub fn clear_breakpoints(&self, file_path: &str) -> Result<(), String> {
        let mut bps = self.breakpoints.lock().map_err(|e| e.to_string())?;
        bps.remove(file_path);
        println!("[Debug] Cleared breakpoints for {}", file_path);
        Ok(())
    }

    pub fn get_all_breakpoints(&self) -> Result<HashMap<String, Vec<Breakpoint>>, String> {
        let bps = self.breakpoints.lock().map_err(|e| e.to_string())?;
        Ok(bps.clone())
    }
}
