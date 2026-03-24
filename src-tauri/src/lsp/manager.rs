use super::client::LspClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct LspManager {
    clients: Arc<Mutex<HashMap<String, Arc<LspClient>>>>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_server(&self, language_id: &str, command: &str, args: Vec<String>) {
        let mut clients = self.clients.lock().await;

        let client = Arc::new(LspClient::new(
            language_id.to_string(),
            command.to_string(),
            args,
        ));

        clients.insert(language_id.to_string(), client);
        println!("[LSP Manager] Registered language server for {}", language_id);
    }

    pub async fn start_server(&self, language_id: &str) -> Result<(), String> {
        let clients = self.clients.lock().await;

        if let Some(client) = clients.get(language_id) {
            client.start().await?;
            Ok(())
        } else {
            Err(format!("No language server registered for {}", language_id))
        }
    }

    pub async fn stop_server(&self, language_id: &str) -> Result<(), String> {
        let clients = self.clients.lock().await;

        if let Some(client) = clients.get(language_id) {
            client.stop().await?;
            Ok(())
        } else {
            Err(format!("No language server registered for {}", language_id))
        }
    }

    pub async fn get_client(&self, language_id: &str) -> Option<Arc<LspClient>> {
        let clients = self.clients.lock().await;
        clients.get(language_id).cloned()
    }

    /// Register default language servers. These are lazy-started when first needed.
    pub fn initialize_defaults(&self) {
        println!("[LSP Manager] Default language servers will be registered on session init");
    }

    /// Register all default language servers asynchronously
    pub async fn register_defaults(&self) {
        // Python - pyright
        self.register_server(
            "python",
            "pyright-langserver",
            vec!["--stdio".to_string()],
        ).await;

        // Rust - rust-analyzer
        self.register_server(
            "rust",
            "rust-analyzer",
            vec![],
        ).await;

        // Go - gopls
        self.register_server(
            "go",
            "gopls",
            vec![],
        ).await;

        // JSON - vscode-json-language-server
        self.register_server(
            "json",
            "vscode-json-language-server",
            vec!["--stdio".to_string()],
        ).await;

        println!("[LSP Manager] Default language servers registered");
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}