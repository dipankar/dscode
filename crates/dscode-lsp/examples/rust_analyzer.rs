//! LSP client example — register and start rust-analyzer.
//!
//! Run with: cargo run --example rust_analyzer --features ...

use dscode_lsp::{LspManager, LspServerPool, LspServerStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a manager and register rust-analyzer
    let manager = LspManager::new();
    manager
        .register_server("rust", "rust-analyzer", vec!["--stdio".to_string()])
        .await;

    // Start the server
    manager.start_server("rust").await?;

    // Or use a connection pool with a strategy
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
    pool
        .register_server(
            "rust".to_string(),
            "rust-analyzer".to_string(),
            vec!["--stdio".to_string()],
        )
        .await;

    println!("rust-analyzer registered and started.");
    Ok(())
}
