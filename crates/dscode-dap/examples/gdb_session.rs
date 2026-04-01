//! Debug session example — create a GDB-backed debug session.
//!
//! Run with: cargo run --example gdb_session

use dscode_dap::{DebugAdapterPool, DebugManager, SourceBreakpoint};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Register the debug adapter
    let pool = DebugAdapterPool::new();
    pool
        .register_adapter(
            "cppdbg".to_string(),
            "/usr/bin/gdb".to_string(),
            vec!["--interpreter=dap".to_string()],
        )
        .await;

    // Create a debug session
    let manager = DebugManager::new();
    let session_id = manager.create_session("main".to_string(), "cppdbg".to_string())?;
    let session = manager.get_session(&session_id).unwrap();

    // Start the adapter
    let adapter = pool.create_session(session).await?;

    // Initialize and set breakpoints
    adapter.initialize().await?;
    adapter
        .set_breakpoints(
            serde_json::json!({ "path": "/path/to/source.cpp" }),
            vec![SourceBreakpoint {
                line: 10,
                column: None,
                condition: None,
                hit_condition: None,
                log_message: None,
            }],
        )
        .await?;

    println!("Debug session initialized with breakpoints.");
    Ok(())
}
