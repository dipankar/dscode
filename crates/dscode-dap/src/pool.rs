/**
 * Debug Adapter Pool
 *
 * Manages multiple debug adapter instances with:
 * - One adapter per debug session
 * - Session lifecycle management
 * - Request routing
 */
use crate::adapter::DebugAdapter;
use crate::types::{DebugSession, DebugState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

type AdapterConfigMap = Arc<RwLock<HashMap<String, (String, Vec<String>)>>>;

/// Information about a running debug adapter
pub(crate) struct DebugAdapterInfo {
    #[allow(dead_code)]
    pub(crate) session_id: String,
    pub(crate) adapter: Arc<DebugAdapter>,
    pub(crate) state: DebugState,
}

/// Pool for managing multiple debug adapters
pub struct DebugAdapterPool {
    /// All running debug adapters
    adapters: Arc<RwLock<HashMap<String, DebugAdapterInfo>>>,

    /// Adapter configurations: adapter_type -> (command, args)
    configurations: AdapterConfigMap,
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
        info!(adapter_type = %adapter_type, "Registered adapter configuration");
    }

    /// Create and start a new debug session
    pub async fn create_session(&self, session: DebugSession) -> Result<Arc<DebugAdapter>, String> {
        // Get adapter configuration
        let (command, args) = {
            let configs = self.configurations.read().await;
            configs
                .get(&session.adapter_type)
                .ok_or(format!(
                    "No configuration found for adapter type {}",
                    session.adapter_type
                ))?
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

        info!(id = %session.id, "Started adapter for session");
        Ok(adapter)
    }

    /// Get an existing debug adapter
    pub async fn get_adapter(&self, session_id: &str) -> Option<Arc<DebugAdapter>> {
        let adapters = self.adapters.read().await;
        adapters
            .get(session_id)
            .map(|info| Arc::clone(&info.adapter))
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
            adapter_info.adapter.stop().await?;
            info!(id = %session_id, "Stopped debug session");
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    pub async fn stop_all(&self) -> Result<(), String> {
        let mut adapters = self.adapters.write().await;

        for (session_id, adapter_info) in adapters.iter() {
            adapter_info.adapter.stop().await?;
            info!(id = %session_id, "Stopped debug session");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_new() {
        let pool = DebugAdapterPool::new();
        let sessions = pool.list_sessions().await;
        assert!(sessions.is_empty(), "New pool should have no sessions");
    }

    #[tokio::test]
    async fn test_pool_stats_empty() {
        let pool = DebugAdapterPool::new();
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_sessions, 0, "Empty pool should have 0 sessions");
        assert!(
            stats.states.is_empty(),
            "Empty pool should have no state counts"
        );
    }

    #[tokio::test]
    async fn test_pool_register_adapter() {
        let pool = DebugAdapterPool::new();
        pool.register_adapter("cppdbg".to_string(), "/usr/bin/gdb".to_string(), vec![])
            .await;

        // Registering a config does not create a running session
        let sessions = pool.list_sessions().await;
        assert!(
            sessions.is_empty(),
            "Registering config should not create a session"
        );

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_sessions, 0);
    }

    #[tokio::test]
    async fn test_pool_get_adapter_nonexistent() {
        let pool = DebugAdapterPool::new();
        let result = pool.get_adapter("nonexistent-session").await;
        assert!(
            result.is_none(),
            "Getting nonexistent adapter should return None"
        );
    }

    #[tokio::test]
    async fn test_pool_create_session_unconfigured() {
        let pool = DebugAdapterPool::new();
        let session = DebugSession {
            id: "test-session".to_string(),
            name: "Test".to_string(),
            state: DebugState::Stopped,
            adapter_type: "nonexistent".to_string(),
        };
        let result = pool.create_session(session).await;
        assert!(
            result.is_err(),
            "Should fail when no adapter config registered"
        );
        assert!(result.unwrap_err().contains("No configuration found"));
    }

    #[tokio::test]
    async fn test_pool_stop_session_nonexistent() {
        let pool = DebugAdapterPool::new();
        let result = pool.stop_session("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_pool_update_state_nonexistent() {
        let pool = DebugAdapterPool::new();
        let result = pool.update_state("nonexistent", DebugState::Running).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_pool_register_multiple_configs() {
        let pool = DebugAdapterPool::new();
        pool.register_adapter("cppdbg".to_string(), "/usr/bin/gdb".to_string(), vec![])
            .await;
        pool.register_adapter(
            "python".to_string(),
            "debugpy".to_string(),
            vec!["--listen".to_string()],
        )
        .await;
        pool.register_adapter("go".to_string(), "dlv".to_string(), vec![])
            .await;

        // Configs registered but no sessions started
        assert!(pool.list_sessions().await.is_empty());
    }

    #[tokio::test]
    async fn test_pool_list_sessions_empty() {
        let pool = DebugAdapterPool::new();
        let sessions = pool.list_sessions().await;
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_pool_default_trait() {
        let pool = DebugAdapterPool::default();
        assert!(pool.list_sessions().await.is_empty());
    }
}
