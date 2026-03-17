/**
 * LSP Server Pool
 *
 * Manages multiple language server instances with:
 * - One server per language
 * - Load balancing for multiple servers per language
 * - Server lifecycle management
 * - Request routing
 */

use super::client::LspClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Strategy for allocating language servers
#[derive(Debug, Clone)]
pub enum LspServerStrategy {
    /// One server per language (default)
    OnePerLanguage,
    /// Multiple servers per language with load balancing
    MultiplePerLanguage { max_servers: usize },
}

/// Information about a running LSP server
pub struct LspServerInfo {
    pub language_id: String,
    pub client: Arc<LspClient>,
    pub request_count: usize,
}

/// Pool for managing multiple LSP servers
pub struct LspServerPool {
    /// All running LSP servers
    servers: Arc<RwLock<HashMap<String, Vec<LspServerInfo>>>>,

    /// Allocation strategy
    strategy: LspServerStrategy,

    /// Server configurations: language_id -> (command, args)
    configurations: Arc<RwLock<HashMap<String, (String, Vec<String>)>>>,
}

impl LspServerPool {
    pub fn new(strategy: LspServerStrategy) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            configurations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a language server configuration
    pub async fn register_server(
        &self,
        language_id: String,
        server_command: String,
        server_args: Vec<String>,
    ) {
        let mut configs = self.configurations.write().await;
        configs.insert(language_id.clone(), (server_command, server_args));
        println!("[LSP Pool] Registered server configuration for {}", language_id);
    }

    /// Get or create a language server for a language
    pub async fn get_server(&self, language_id: &str) -> Result<Arc<LspClient>, String> {
        // Check if we have a running server
        {
            let servers = self.servers.read().await;
            if let Some(server_list) = servers.get(language_id) {
                if !server_list.is_empty() {
                    // Return the server with the least requests (simple load balancing)
                    let min_server = server_list
                        .iter()
                        .min_by_key(|s| s.request_count)
                        .unwrap();
                    return Ok(Arc::clone(&min_server.client));
                }
            }
        }

        // Need to start a new server
        self.start_server(language_id).await
    }

    /// Start a new language server
    async fn start_server(&self, language_id: &str) -> Result<Arc<LspClient>, String> {
        // Get server configuration
        let (command, args) = {
            let configs = self.configurations.read().await;
            configs
                .get(language_id)
                .ok_or(format!("No configuration found for language {}", language_id))?
                .clone()
        };

        // Create client
        let client = Arc::new(LspClient::new(
            language_id.to_string(),
            command,
            args,
        ));

        // Start the server
        client.start().await?;

        // Add to pool
        let server_info = LspServerInfo {
            language_id: language_id.to_string(),
            client: Arc::clone(&client),
            request_count: 0,
        };

        let mut servers = self.servers.write().await;
        servers
            .entry(language_id.to_string())
            .or_insert_with(Vec::new)
            .push(server_info);

        println!("[LSP Pool] Started server for {}", language_id);
        Ok(client)
    }

    /// Stop a language server
    pub async fn stop_server(&self, language_id: &str) -> Result<(), String> {
        let mut servers = self.servers.write().await;

        if let Some(server_list) = servers.remove(language_id) {
            for server_info in server_list {
                server_info.client.stop()?;
            }
            println!("[LSP Pool] Stopped all servers for {}", language_id);
            Ok(())
        } else {
            Err(format!("No server found for language {}", language_id))
        }
    }

    /// Stop all servers
    pub async fn stop_all(&self) -> Result<(), String> {
        let mut servers = self.servers.write().await;

        for (language_id, server_list) in servers.iter() {
            for server_info in server_list {
                server_info.client.stop()?;
            }
            println!("[LSP Pool] Stopped servers for {}", language_id);
        }

        servers.clear();
        Ok(())
    }

    /// Get list of running servers
    pub async fn list_servers(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> LspPoolStats {
        let servers = self.servers.read().await;
        let total_servers: usize = servers.values().map(|list| list.len()).sum();
        let total_languages = servers.len();

        LspPoolStats {
            total_servers,
            total_languages,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LspPoolStats {
    pub total_servers: usize,
    pub total_languages: usize,
}
