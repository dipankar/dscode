/**
 * Debug Adapter Pool
 *
 * Manages multiple debug adapter instances with:
 * - One adapter per debug session
 * - Session lifecycle management
 * - Request routing
 */

use super::adapter::DebugAdapter;
use super::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Information about a running debug adapter
pub struct DebugAdapterInfo {
    pub session_id: String,
    pub adapter: Arc<DebugAdapter>,
    pub state: DebugState,
}

/// Pool for managing multiple debug adapters
pub struct DebugAdapterPool {
    /// All running debug adapters
    adapters: Arc<RwLock<HashMap<String, DebugAdapterInfo>>>,

    /// Adapter configurations: adapter_type -> (command, args)
    configurations: Arc<RwLock<HashMap<String, (String, Vec<String>)>>>,
}

impl DebugAdapterPool {
    pub fn new() -> Self {
        Self {
            adapters: Arc::new(RwLock::new(HashMap::new())),
            configurations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a debug adapter configuration
    pub async fn register_adapter(
        &self,
        adapter_type: String,
        adapter_command: String,
        adapter_args: Vec<String>,
    ) {
        let mut configs = self.configurations.write().await;
        configs.insert(adapter_type.clone(), (adapter_command, adapter_args));
        println!("[Debug Pool] Registered adapter configuration for {}", adapter_type);
    }

    /// Create and start a new debug session
    pub async fn create_session(
        &self,
        session: DebugSession,
    ) -> Result<Arc<DebugAdapter>, String> {
        // Get adapter configuration
        let (command, args) = {
            let configs = self.configurations.read().await;
            configs
                .get(&session.adapter_type)
                .ok_or(format!("No configuration found for adapter type {}", session.adapter_type))?
                .clone()
        };

        // Create adapter
        let adapter = Arc::new(DebugAdapter::new(session.clone(), command, args));

        // Start the adapter
        adapter.start().await?;

        // Add to pool
        let adapter_info = DebugAdapterInfo {
            session_id: session.id.clone(),
            adapter: Arc::clone(&adapter),
            state: DebugState::Stopped,
        };

        let mut adapters = self.adapters.write().await;
        adapters.insert(session.id.clone(), adapter_info);

        println!("[Debug Pool] Started adapter for session {}", session.id);
        Ok(adapter)
    }

    /// Get an existing debug adapter
    pub async fn get_adapter(&self, session_id: &str) -> Option<Arc<DebugAdapter>> {
        let adapters = self.adapters.read().await;
        adapters.get(session_id).map(|info| Arc::clone(&info.adapter))
    }

    /// Update session state
    pub async fn update_state(&self, session_id: &str, state: DebugState) -> Result<(), String> {
        let mut adapters = self.adapters.write().await;
        if let Some(info) = adapters.get_mut(session_id) {
            info.state = state;
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Stop a debug session
    pub async fn stop_session(&self, session_id: &str) -> Result<(), String> {
        let mut adapters = self.adapters.write().await;

        if let Some(adapter_info) = adapters.remove(session_id) {
            adapter_info.adapter.stop()?;
            println!("[Debug Pool] Stopped session {}", session_id);
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Stop all debug sessions
    pub async fn stop_all(&self) -> Result<(), String> {
        let mut adapters = self.adapters.write().await;

        for (session_id, adapter_info) in adapters.iter() {
            adapter_info.adapter.stop()?;
            println!("[Debug Pool] Stopped session {}", session_id);
        }

        adapters.clear();
        Ok(())
    }

    /// Get list of active sessions
    pub async fn list_sessions(&self) -> Vec<String> {
        let adapters = self.adapters.read().await;
        adapters.keys().cloned().collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> DebugPoolStats {
        let adapters = self.adapters.read().await;
        let total_sessions = adapters.len();

        let mut states: HashMap<DebugState, usize> = HashMap::new();
        for info in adapters.values() {
            *states.entry(info.state.clone()).or_insert(0) += 1;
        }

        DebugPoolStats {
            total_sessions,
            states,
        }
    }
}

impl Default for DebugAdapterPool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DebugPoolStats {
    pub total_sessions: usize,
    pub states: HashMap<DebugState, usize>,
}
