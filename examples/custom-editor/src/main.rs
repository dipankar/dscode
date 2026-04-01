//! Minimal custom editor using dscode-core and dscode-lsp.
//!
//! This example demonstrates how to compose DSCode crates independently
//! to build a lightweight code editor or IDE-like tool.

use dscode_core::TextBuffer;
use dscode_lsp::{LspServerPool, LspServerStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load source file into a rope-backed buffer
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "src/main.rs".to_string());
    let source = std::fs::read_to_string(&path)?;
    let mut buffer = TextBuffer::new(&source);

    println!("Loaded {} ({} chars)", path, buffer.len_chars());

    // Start an LSP server pool with one server per language
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

    // Register rust-analyzer for the "rust" language
    pool
        .register_server(
            "rust".to_string(),
            "rust-analyzer".to_string(),
            vec!["--stdio".to_string()],
        )
        .await;

    // Get a client for the current file's language
    let file_uri = format!("file://{}", std::fs::canonicalize(&path)?.display());
    let client = pool.get_server("rust", Some(&file_uri)).await?;

    // Request hover information at the start of the file
    match client.hover(&file_uri, 0, 0).await {
        Ok(Some(hover)) => println!("Hover: {:?}", hover),
        Ok(None) => println!("No hover info available."),
        Err(e) => eprintln!("LSP error: {}", e),
    }

    // Simulate a user edit
    buffer.insert(0, "// Hello from custom-editor\n");
    println!("Buffer after edit: {} chars", buffer.len_chars());

    // Graceful shutdown: the pool drops clients and kills child processes
    drop(pool);
    println!("Shutdown complete.");

    Ok(())
}
