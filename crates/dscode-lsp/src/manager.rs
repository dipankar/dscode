use super::client::LspClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Manages language server processes and handles communication
/// between the editor and language servers.
pub struct LspManager {
    clients: Arc<Mutex<HashMap<String, Arc<LspClient>>>>,
}

impl LspManager {
    /// Creates a new `LspManager` with no registered language servers.
    pub fn new() -> Self {
        Self { clients: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Registers a language server for the given language.
    ///
    /// - `language_id` — The language identifier (e.g., "rust", "python").
    /// - `command` — The command to spawn the language server.
    /// - `args` — Arguments to pass to the server command.
    ///
    /// If a server is already registered for the same language, it will be replaced.
    pub async fn register_server(&self, language_id: &str, command: &str, args: Vec<String>) {
        let mut clients = self.clients.lock().await;

        let client = Arc::new(LspClient::new(language_id.to_string(), command.to_string(), args));

        clients.insert(language_id.to_string(), client);
        info!(language = language_id, "Registered language server");
    }

    /// Starts the language server registered for the given language.
    ///
    /// - `language_id` — The language identifier to start the server for.
    ///
    /// Returns `Ok(())` if the server started successfully, or an error if
    /// no server is registered for the language or the start failed.
    pub async fn start_server(&self, language_id: &str) -> Result<(), String> {
        let clients = self.clients.lock().await;

        if let Some(client) = clients.get(language_id) {
            client.start().await?;
            Ok(())
        } else {
            Err(format!("No language server registered for {}", language_id))
        }
    }

    /// Stops the language server registered for the given language.
    ///
    /// - `language_id` — The language identifier to stop the server for.
    ///
    /// Returns `Ok(())` if the server stopped successfully, or an error if
    /// no server is registered for the language or the stop failed.
    pub async fn stop_server(&self, language_id: &str) -> Result<(), String> {
        let clients = self.clients.lock().await;

        if let Some(client) = clients.get(language_id) {
            client.stop().await?;
            Ok(())
        } else {
            Err(format!("No language server registered for {}", language_id))
        }
    }

    /// Returns the LSP client for the given language, if registered.
    ///
    /// - `language_id` — The language identifier to look up.
    pub async fn get_client(&self, language_id: &str) -> Option<Arc<LspClient>> {
        let clients = self.clients.lock().await;
        clients.get(language_id).cloned()
    }

    /// Register default language servers. These are lazy-started when first needed.
    pub fn initialize_defaults(&self) {
        info!("Default language servers will be registered on session init");
    }

    /// Register all default language servers asynchronously
    pub async fn register_defaults(&self) {
        // Python - pyright
        self.register_server("python", "pyright-langserver", vec!["--stdio".to_string()]).await;

        // Rust - rust-analyzer
        self.register_server("rust", "rust-analyzer", vec![]).await;

        // Go - gopls
        self.register_server("go", "gopls", vec![]).await;

        // JSON - vscode-json-language-server
        self.register_server("json", "vscode-json-language-server", vec!["--stdio".to_string()])
            .await;

        info!("Default language servers registered");
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LspClientState;

    #[tokio::test]
    async fn test_lsp_manager_new() {
        let manager = LspManager::new();
        // A new manager should have no registered clients
        let client = manager.get_client("rust").await;
        assert!(client.is_none());
        let client = manager.get_client("python").await;
        assert!(client.is_none());
    }

    #[tokio::test]
    async fn test_lsp_manager_get_nonexistent() {
        let manager = LspManager::new();
        // Getting a client for an unregistered language should return None
        assert!(manager.get_client("javascript").await.is_none());
        assert!(manager.get_client("typescript").await.is_none());
        assert!(manager.get_client("go").await.is_none());
    }

    #[tokio::test]
    async fn test_lsp_manager_register_and_get() {
        let manager = LspManager::new();

        // Register a server
        manager.register_server("rust", "rust-analyzer", vec![]).await;

        // Should now be retrievable
        let client = manager.get_client("rust").await;
        assert!(client.is_some(), "Registered client should be retrievable");

        // The retrieved client should be in Stopped state
        let client = client.unwrap();
        assert_eq!(client.get_state().await, LspClientState::Stopped);
    }

    #[tokio::test]
    async fn test_lsp_manager_register_overwrites() {
        let manager = LspManager::new();

        // Register twice with the same language_id
        manager.register_server("python", "pyright-old", vec![]).await;
        manager.register_server("python", "pyright-new", vec!["--stdio".to_string()]).await;

        // Should still return a client (the latest one)
        let client = manager.get_client("python").await;
        assert!(client.is_some());
    }

    #[tokio::test]
    async fn test_lsp_manager_register_defaults() {
        let manager = LspManager::new();

        // Before registering defaults, no languages should be available
        assert!(manager.get_client("python").await.is_none());
        assert!(manager.get_client("rust").await.is_none());
        assert!(manager.get_client("go").await.is_none());
        assert!(manager.get_client("json").await.is_none());

        // Register defaults
        manager.register_defaults().await;

        // After registering defaults, the default languages should be available
        assert!(manager.get_client("python").await.is_some());
        assert!(manager.get_client("rust").await.is_some());
        assert!(manager.get_client("go").await.is_some());
        assert!(manager.get_client("json").await.is_some());
    }

    #[tokio::test]
    async fn test_lsp_manager_start_unregistered() {
        let manager = LspManager::new();

        // Attempting to start a server that was never registered should fail
        let result = manager.start_server("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No language server registered"));
    }

    #[tokio::test]
    async fn test_lsp_manager_stop_unregistered() {
        let manager = LspManager::new();

        // Attempting to stop a server that was never registered should fail
        let result = manager.stop_server("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No language server registered"));
    }

    #[tokio::test]
    async fn test_lsp_manager_multiple_registrations() {
        let manager = LspManager::new();

        manager.register_server("rust", "rust-analyzer", vec![]).await;
        manager.register_server("python", "pyright-langserver", vec!["--stdio".to_string()]).await;
        manager.register_server("go", "gopls", vec![]).await;

        assert!(manager.get_client("rust").await.is_some());
        assert!(manager.get_client("python").await.is_some());
        assert!(manager.get_client("go").await.is_some());
        assert!(manager.get_client("javascript").await.is_none());
    }
}