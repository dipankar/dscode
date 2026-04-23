use super::client::LspClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Strategy for managing LSP server instances per language.
#[derive(Debug, Clone)]
pub enum LspServerStrategy {
    /// Maintain a single server instance per language (default).
    OnePerLanguage,
    /// Allow multiple server instances per language, up to `max_servers`.
    MultiplePerLanguage {
        /// Maximum number of server instances per language.
        max_servers: usize,
    },
}

/// Information about a running LSP server instance.
#[allow(dead_code)]
pub(crate) struct LspServerInfo {
    #[allow(dead_code)]
    pub(crate) language_id: String,
    pub(crate) client: Arc<LspClient>,
    pub(crate) request_count: Arc<std::sync::atomic::AtomicU64>,
}

type ServerConfigMap = Arc<RwLock<HashMap<String, (String, Vec<String>)>>>;

/// Pool of LSP server instances with configurable strategies.
///
/// Manages a collection of running language server instances, lazily starting
/// them on first access via [`get_server`](LspServerPool::get_server).
/// Selects the server with the fewest active requests for load balancing.
pub struct LspServerPool {
    servers: Arc<RwLock<HashMap<String, Vec<Arc<LspServerInfo>>>>>,
    #[allow(dead_code)]
    strategy: LspServerStrategy,
    configurations: ServerConfigMap,
}

impl LspServerPool {
    /// Creates a new pool with the given server strategy.
    pub fn new(strategy: LspServerStrategy) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            configurations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a server configuration for a language without starting it.
    ///
    /// - `language_id` — The language identifier.
    /// - `server_command` — The command to spawn the language server.
    /// - `server_args` — Arguments to pass to the server command.
    pub async fn register_server(
        &self,
        language_id: String,
        server_command: String,
        server_args: Vec<String>,
    ) {
        let mut configs = self.configurations.write().await;
        configs.insert(language_id.clone(), (server_command, server_args));
        info!(language = %language_id, "Registered server configuration");
    }

    /// Gets or starts an LSP server for the given language.
    ///
    /// Returns the server with the fewest active requests (least-loaded selection).
    /// If no running server exists, starts a new one using the registered
    /// configuration and initializes it with the given root URI.
    ///
    /// - `language_id` — The language to get a server for.
    /// - `root_uri` — Optional root URI for LSP initialization.
    ///
    /// Returns a shared reference to the [`LspClient`], or an error if no
    /// configuration is registered for the language or the start fails.
    pub async fn get_server(
        &self,
        language_id: &str,
        root_uri: Option<&str>,
    ) -> Result<Arc<LspClient>, String> {
        {
            let servers = self.servers.read().await;
            if let Some(server_list) = servers.get(language_id) {
                if !server_list.is_empty() {
                    let min_server = server_list
                        .iter()
                        .min_by_key(|s| s.request_count.load(std::sync::atomic::Ordering::Relaxed))
                        .expect("server list is non-empty");
                    min_server
                        .request_count
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return Ok(Arc::clone(&min_server.client));
                }
            }
        }

        self.start_server(language_id, root_uri).await
    }

    async fn start_server(
        &self,
        language_id: &str,
        root_uri: Option<&str>,
    ) -> Result<Arc<LspClient>, String> {
        let (command, args) = {
            let configs = self.configurations.read().await;
            configs
                .get(language_id)
                .ok_or(format!(
                    "No configuration found for language {}",
                    language_id
                ))?
                .clone()
        };

        let client = Arc::new(LspClient::new(language_id.to_string(), command, args));

        client.start().await?;

        if let Some(uri) = root_uri {
            match lsp_types::Url::parse(uri) {
                Ok(url) => {
                    client.initialize(url).await.map_err(|e| {
                        error!(language = language_id, "Failed to initialize LSP: {}", e);
                        format!("LSP initialize failed: {}", e)
                    })?;
                }
                Err(e) => {
                    error!(uri = uri, "Invalid root URI: {}", e);
                }
            }
        }

        client.initialized().await.map_err(|e| {
            error!(language = language_id, "Failed to send initialized: {}", e);
            format!("LSP initialized notification failed: {}", e)
        })?;

        let server_info = Arc::new(LspServerInfo {
            language_id: language_id.to_string(),
            client: Arc::clone(&client),
            request_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        });

        let mut servers = self.servers.write().await;
        servers
            .entry(language_id.to_string())
            .or_insert_with(Vec::new)
            .push(server_info);

        info!(language = language_id, "Started and initialized LSP server");
        Ok(client)
    }

    /// Stops all running servers for the given language and removes them from the pool.
    ///
    /// - `language_id` — The language whose servers should be stopped.
    ///
    /// Returns `Ok(())` if at least one server was stopped, or an error if
    /// no server was found for the language.
    pub async fn stop_server(&self, language_id: &str) -> Result<(), String> {
        let mut servers = self.servers.write().await;

        if let Some(server_list) = servers.remove(language_id) {
            for server_info in server_list {
                server_info.client.shutdown().await.ok();
                server_info.client.stop().await?;
            }
            info!(language = language_id, "Stopped all LSP servers");
            Ok(())
        } else {
            Err(format!("No server found for language {}", language_id))
        }
    }

    /// Stops all running LSP servers and clears the pool.
    ///
    /// Returns `Ok(())` if all servers shut down successfully.
    pub async fn stop_all(&self) -> Result<(), String> {
        let mut servers = self.servers.write().await;

        for (language_id, server_list) in servers.iter() {
            for server_info in server_list {
                server_info.client.shutdown().await.ok();
                server_info.client.stop().await?;
            }
            info!(language = language_id, "Stopped LSP servers");
        }

        servers.clear();
        Ok(())
    }

    /// Returns the language identifiers for all currently running servers.
    pub async fn list_servers(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }

    /// Returns statistics about the pool's current state.
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

/// Statistics about the LSP server pool.
#[derive(Debug, Clone)]
pub struct LspPoolStats {
    /// Total number of running server instances across all languages.
    pub total_servers: usize,
    /// Number of distinct languages with at least one running server.
    pub total_languages: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_new() {
        let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
        let servers = pool.list_servers().await;
        assert!(servers.is_empty(), "New pool should have no servers");
    }

    #[tokio::test]
    async fn test_pool_stats_empty() {
        let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
        let stats = pool.get_stats().await;
        assert_eq!(
            stats.total_servers, 0,
            "Empty pool should have 0 total servers"
        );
        assert_eq!(
            stats.total_languages, 0,
            "Empty pool should have 0 total languages"
        );
    }

    #[tokio::test]
    async fn test_pool_register_server() {
        let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
        pool.register_server("rust".to_string(), "rust-analyzer".to_string(), vec![])
            .await;

        // After registering, list_servers should still be empty because
        // register only stores the configuration, not a running server
        let servers = pool.list_servers().await;
        assert!(
            servers.is_empty(),
            "Registered config does not create a running server"
        );

        // Stats should still show 0
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_servers, 0);
        assert_eq!(stats.total_languages, 0);
    }

    #[tokio::test]
    async fn test_pool_get_server_unconfigured() {
        let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

        // Trying to get a server for a language with no configuration should fail
        let result = pool.get_server("rust", None).await;
        assert!(
            result.is_err(),
            "Should fail when no configuration registered"
        );
        assert!(result.unwrap_err().contains("No configuration found"));
    }

    #[tokio::test]
    async fn test_pool_list_servers_empty() {
        let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
        let servers = pool.list_servers().await;
        assert!(servers.is_empty());
    }

    #[tokio::test]
    async fn test_pool_stats_after_register() {
        let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

        // Registering a configuration should not change stats until a server is started
        pool.register_server("python".to_string(), "pyright".to_string(), vec![])
            .await;

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_servers, 0);
        assert_eq!(stats.total_languages, 0);
    }

    #[tokio::test]
    async fn test_pool_multiple_strategy() {
        let pool = LspServerPool::new(LspServerStrategy::MultiplePerLanguage { max_servers: 3 });

        // Pool should start empty
        let servers = pool.list_servers().await;
        assert!(servers.is_empty());

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_servers, 0);
        assert_eq!(stats.total_languages, 0);
    }

    #[tokio::test]
    async fn test_pool_register_multiple_configs() {
        let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

        pool.register_server("rust".to_string(), "rust-analyzer".to_string(), vec![])
            .await;
        pool.register_server(
            "python".to_string(),
            "pyright".to_string(),
            vec!["--stdio".to_string()],
        )
        .await;
        pool.register_server("go".to_string(), "gopls".to_string(), vec![])
            .await;

        // All configs registered; servers list should still be empty
        let servers = pool.list_servers().await;
        assert!(servers.is_empty(), "Configs don't create running servers");
    }

    #[tokio::test]
    async fn test_pool_stop_unregistered() {
        let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

        // Stopping a server that was never started should return an error
        let result = pool.stop_server("rust").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No server found"));
    }
}
