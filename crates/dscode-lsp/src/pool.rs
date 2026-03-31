use super::client::LspClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Strategy for managing LSP server instances per language.
#[derive(Debug, Clone)]
pub enum LspServerStrategy {
    /// One server instance per language (default)
    OnePerLanguage,
    /// Multiple server instances per language up to a maximum
    MultiplePerLanguage { max_servers: usize },
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
pub struct LspServerPool {
    servers: Arc<RwLock<HashMap<String, Vec<Arc<LspServerInfo>>>>>,
    #[allow(dead_code)]
    strategy: LspServerStrategy,
    configurations: ServerConfigMap,
}

impl LspServerPool {
    pub fn new(strategy: LspServerStrategy) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            configurations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_server(
        &self, language_id: String, server_command: String, server_args: Vec<String>,
    ) {
        let mut configs = self.configurations.write().await;
        configs.insert(language_id.clone(), (server_command, server_args));
        info!(language = %language_id, "Registered server configuration");
    }

    pub async fn get_server(
        &self, language_id: &str, root_uri: Option<&str>,
    ) -> Result<Arc<LspClient>, String> {
        {
            let servers = self.servers.read().await;
            if let Some(server_list) = servers.get(language_id) {
                if !server_list.is_empty() {
                    let min_server = server_list
                        .iter()
                        .min_by_key(|s| s.request_count.load(std::sync::atomic::Ordering::Relaxed))
                        .expect("server list is non-empty");
                    min_server.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return Ok(Arc::clone(&min_server.client));
                }
            }
        }

        self.start_server(language_id, root_uri).await
    }

    async fn start_server(
        &self, language_id: &str, root_uri: Option<&str>,
    ) -> Result<Arc<LspClient>, String> {
        let (command, args) = {
            let configs = self.configurations.read().await;
            configs
                .get(language_id)
                .ok_or(format!("No configuration found for language {}", language_id))?
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
        servers.entry(language_id.to_string()).or_insert_with(Vec::new).push(server_info);

        info!(language = language_id, "Started and initialized LSP server");
        Ok(client)
    }

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

    pub async fn list_servers(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }

    pub async fn get_stats(&self) -> LspPoolStats {
        let servers = self.servers.read().await;
        let total_servers: usize = servers.values().map(|list| list.len()).sum();
        let total_languages = servers.len();

        LspPoolStats { total_servers, total_languages }
    }
}

/// Statistics about the LSP server pool.
#[derive(Debug, Clone)]
pub struct LspPoolStats {
    pub total_servers: usize,
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
        assert_eq!(stats.total_servers, 0, "Empty pool should have 0 total servers");
        assert_eq!(stats.total_languages, 0, "Empty pool should have 0 total languages");
    }
}