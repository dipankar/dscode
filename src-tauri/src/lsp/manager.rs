use super::client::LspClient;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct LspManager {
    clients: Arc<Mutex<HashMap<String, Arc<LspClient>>>>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_server(&self, language_id: &str, command: &str, args: Vec<String>) {
        let mut clients = self.clients.lock().unwrap();

        let client = Arc::new(LspClient::new(
            language_id.to_string(),
            command.to_string(),
            args,
        ));

        clients.insert(language_id.to_string(), client);
        println!("[LSP Manager] Registered language server for {}", language_id);
    }

    pub fn start_server(&self, language_id: &str) -> Result<(), String> {
        let clients = self.clients.lock().unwrap();

        if let Some(client) = clients.get(language_id) {
            client.start()?;
            Ok(())
        } else {
            Err(format!("No language server registered for {}", language_id))
        }
    }

    pub fn stop_server(&self, language_id: &str) -> Result<(), String> {
        let clients = self.clients.lock().unwrap();

        if let Some(client) = clients.get(language_id) {
            client.stop()?;
            Ok(())
        } else {
            Err(format!("No language server registered for {}", language_id))
        }
    }

    pub fn get_client(&self, language_id: &str) -> Option<Arc<LspClient>> {
        let clients = self.clients.lock().unwrap();
        clients.get(language_id).cloned()
    }

    pub fn initialize_defaults(&self) {
        // Register common language servers

        // TypeScript/JavaScript - handled by Monaco
        // But we can add tsserver for better support later

        // Python - pyright
        self.register_server(
            "python",
            "pyright-langserver",
            vec!["--stdio".to_string()],
        );

        // Rust - rust-analyzer
        self.register_server(
            "rust",
            "rust-analyzer",
            vec![],
        );

        // Go - gopls
        self.register_server(
            "go",
            "gopls",
            vec![],
        );

        // JSON - vscode-json-language-server
        self.register_server(
            "json",
            "vscode-json-language-server",
            vec!["--stdio".to_string()],
        );

        println!("[LSP Manager] Default language servers registered");
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}
