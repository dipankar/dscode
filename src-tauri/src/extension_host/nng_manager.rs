/**
 * NNG IPC Connection Manager
 *
 * Manages bidirectional NNG IPC connections to multiple extension hosts.
 * Each extension host has:
 * - Outgoing connection (Tauri->ExtHost): REQ socket
 * - Incoming connection (ExtHost->Tauri): REP socket
 */

use super::nng_ipc::{NngExtensionIpc, NngIncomingIpc, IncomingRequestHandler};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NngIpcManager {
    /// Outgoing connections (Tauri sends requests to Extension Host)
    outgoing: Arc<RwLock<HashMap<String, Arc<NngExtensionIpc>>>>,
    /// Incoming connections (Extension Host sends requests to Tauri)
    incoming: Arc<RwLock<HashMap<String, Arc<NngIncomingIpc>>>>,
}

impl NngIpcManager {
    pub fn new() -> Self {
        Self {
            outgoing: Arc::new(RwLock::new(HashMap::new())),
            incoming: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to an extension host's IPC endpoint (Tauri->ExtHost communication)
    pub async fn connect_outgoing(&self, ext_host_id: &str, ipc_url: &str) -> Result<(), String> {
        println!("[NNG Manager] Connecting outgoing to extension host '{}' at {}", ext_host_id, ipc_url);

        let ipc = NngExtensionIpc::new(ipc_url)?;
        let mut conns = self.outgoing.write().await;
        conns.insert(ext_host_id.to_string(), Arc::new(ipc));

        println!("[NNG Manager] Outgoing connection established for '{}'", ext_host_id);
        Ok(())
    }

    /// Setup incoming IPC endpoint (ExtHost->Tauri communication)
    pub async fn setup_incoming(
        &self,
        ext_host_id: &str,
        ipc_url: &str,
        handler: IncomingRequestHandler,
    ) -> Result<(), String> {
        println!("[NNG Manager] Setting up incoming for extension host '{}' at {}", ext_host_id, ipc_url);

        let mut ipc = NngIncomingIpc::new(ipc_url)?;
        ipc.set_handler(handler);

        let ipc_arc = Arc::new(ipc);
        ipc_arc.start().await?;

        let mut conns = self.incoming.write().await;
        conns.insert(ext_host_id.to_string(), ipc_arc);

        println!("[NNG Manager] Incoming connection ready for '{}'", ext_host_id);
        Ok(())
    }

    /// Disconnect from an extension host (both directions)
    pub async fn disconnect(&self, ext_host_id: &str) {
        let mut outgoing_conns = self.outgoing.write().await;
        let mut incoming_conns = self.incoming.write().await;

        let out_removed = outgoing_conns.remove(ext_host_id).is_some();

        if let Some(incoming) = incoming_conns.remove(ext_host_id) {
            incoming.stop().await;
        }

        if out_removed {
            println!("[NNG Manager] Disconnected from extension host '{}'", ext_host_id);
        }
    }

    /// Get an outgoing IPC connection for an extension host
    pub async fn get_outgoing(&self, ext_host_id: &str) -> Option<Arc<NngExtensionIpc>> {
        let conns = self.outgoing.read().await;
        conns.get(ext_host_id).cloned()
    }

    /// Legacy method for backward compatibility
    pub async fn get(&self, ext_host_id: &str) -> Option<Arc<NngExtensionIpc>> {
        self.get_outgoing(ext_host_id).await
    }

    /// Legacy connect method for backward compatibility
    pub async fn connect(&self, ext_host_id: &str, ipc_url: &str) -> Result<(), String> {
        self.connect_outgoing(ext_host_id, ipc_url).await
    }

    /// Check if connected to an extension host
    pub async fn is_connected(&self, ext_host_id: &str) -> bool {
        let conns = self.outgoing.read().await;
        conns.contains_key(ext_host_id)
    }

    /// Send a request to an extension host
    pub async fn request(&self, ext_host_id: &str, msg_type: &str, payload: serde_json::Value) -> Result<serde_json::Value, String> {
        let ipc = self.get_outgoing(ext_host_id).await
            .ok_or_else(|| format!("Extension host '{}' not connected", ext_host_id))?;

        ipc.request(msg_type, payload).await
    }
}
