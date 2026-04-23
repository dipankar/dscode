//! Integration tests for dscode-lsp crate.
//!
//! Tests multiple LSP components working together: LspManager, LspServerPool,
//! LspServerStrategy, and LspClientState transitions.

use dscode_lsp::{
    LspClient, LspClientState, LspManager, LspPoolStats, LspServerPool, LspServerStrategy,
};

// ── LspManager integration tests ────────────────────────────────────────────

#[tokio::test]
async fn test_lsp_manager_register_and_lookup() {
    let manager = LspManager::new();

    // Initially, no servers are registered
    assert!(manager.get_client("rust").await.is_none());
    assert!(manager.get_client("python").await.is_none());

    // Register a server
    manager
        .register_server("rust", "rust-analyzer", vec![])
        .await;

    // Now it should be findable
    let client = manager.get_client("rust").await;
    assert!(client.is_some());

    let client = client.unwrap();
    assert_eq!(client.get_state().await, LspClientState::Stopped);

    // Other languages should still be absent
    assert!(manager.get_client("python").await.is_none());
}

#[tokio::test]
async fn test_lsp_manager_register_multiple_servers() {
    let manager = LspManager::new();

    manager
        .register_server("rust", "rust-analyzer", vec![])
        .await;
    manager
        .register_server("python", "pyright-langserver", vec!["--stdio".to_string()])
        .await;
    manager.register_server("go", "gopls", vec![]).await;

    // All three should be findable
    assert!(manager.get_client("rust").await.is_some());
    assert!(manager.get_client("python").await.is_some());
    assert!(manager.get_client("go").await.is_some());

    // Unregistered language should still be absent
    assert!(manager.get_client("javascript").await.is_none());
}

#[tokio::test]
async fn test_lsp_manager_replace_server() {
    let manager = LspManager::new();

    manager
        .register_server("rust", "rust-analyzer", vec![])
        .await;

    // Re-register the same language with a different command
    manager
        .register_server(
            "rust",
            "rust-analyzer-alt",
            vec!["--alternative".to_string()],
        )
        .await;

    let client = manager.get_client("rust").await.unwrap();
    assert_eq!(client.get_state().await, LspClientState::Stopped);
}

#[tokio::test]
async fn test_lsp_manager_start_nonexistent_server() {
    let manager = LspManager::new();

    // Starting a server that was never registered should fail
    let result = manager.start_server("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_lsp_manager_stop_nonexistent_server() {
    let manager = LspManager::new();

    let result = manager.stop_server("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_lsp_manager_register_defaults() {
    let manager = LspManager::new();
    manager.register_defaults().await;

    // After registering defaults, common language servers should be available
    assert!(manager.get_client("python").await.is_some());
    assert!(manager.get_client("rust").await.is_some());
    assert!(manager.get_client("go").await.is_some());
    assert!(manager.get_client("json").await.is_some());
}

// ── LspServerPool integration tests ─────────────────────────────────────────

#[tokio::test]
async fn test_pool_register_and_stats() {
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

    // Initially, pool is empty
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_servers, 0);
    assert_eq!(stats.total_languages, 0);

    // Register configurations
    pool.register_server("rust".to_string(), "rust-analyzer".to_string(), vec![])
        .await;
    pool.register_server(
        "python".to_string(),
        "pyright-langserver".to_string(),
        vec!["--stdio".to_string()],
    )
    .await;

    // Stats still zero because no servers have been started
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_servers, 0);
    assert_eq!(stats.total_languages, 0);
}

#[tokio::test]
async fn test_pool_list_servers_empty() {
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
    let servers = pool.list_servers().await;
    assert!(servers.is_empty());
}

#[tokio::test]
async fn test_pool_get_server_not_configured() {
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

    // Getting a server for an unconfigured language should fail
    let result = pool.get_server("rust", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_pool_stop_nonexistent_server() {
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

    let result = pool.stop_server("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_pool_stop_all_when_empty() {
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

    // Stopping all on an empty pool should succeed
    let result = pool.stop_all().await;
    assert!(result.is_ok());
}

// ── LspServerStrategy tests ─────────────────────────────────────────────────

#[test]
fn test_server_strategy_one_per_language() {
    let strategy = LspServerStrategy::OnePerLanguage;

    // Verify strategy variants can be constructed and compared
    assert!(matches!(strategy, LspServerStrategy::OnePerLanguage));
    assert!(matches!(
        LspServerStrategy::MultiplePerLanguage { max_servers: 5 },
        LspServerStrategy::MultiplePerLanguage { .. }
    ));
}

#[test]
fn test_server_strategy_multiple_per_language() {
    let strategy = LspServerStrategy::MultiplePerLanguage { max_servers: 3 };

    if let LspServerStrategy::MultiplePerLanguage { max_servers } = strategy {
        assert_eq!(max_servers, 3);
    } else {
        panic!("Expected MultiplePerLanguage variant");
    }
}

#[test]
fn test_server_strategy_clone_and_debug() {
    let strategy = LspServerStrategy::OnePerLanguage;
    let cloned = strategy.clone();
    assert!(matches!(cloned, LspServerStrategy::OnePerLanguage));

    let multi = LspServerStrategy::MultiplePerLanguage { max_servers: 10 };
    let multi_cloned = multi.clone();
    if let LspServerStrategy::MultiplePerLanguage { max_servers } = multi_cloned {
        assert_eq!(max_servers, 10);
    }
}

// ── LspClientState transition tests ──────────────────────────────────────────

#[test]
fn test_client_state_variants_are_distinct() {
    let states = [
        LspClientState::Stopped,
        LspClientState::Starting,
        LspClientState::Initializing,
        LspClientState::Ready,
        LspClientState::ShuttingDown,
        LspClientState::Crashed,
    ];

    // Every state should be distinct from every other state
    for (i, a) in states.iter().enumerate() {
        for (j, b) in states.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b, "State {:?} should differ from {:?}", a, b);
            }
        }
    }
}

#[tokio::test]
async fn test_client_new_state_is_stopped() {
    let client = LspClient::new("rust".to_string(), "rust-analyzer".to_string(), vec![]);
    assert_eq!(client.get_state().await, LspClientState::Stopped);
}

#[tokio::test]
async fn test_client_new_with_args_state_is_stopped() {
    let client = LspClient::new(
        "python".to_string(),
        "pyright-langserver".to_string(),
        vec!["--stdio".to_string()],
    );
    assert_eq!(client.get_state().await, LspClientState::Stopped);
}

#[tokio::test]
async fn test_client_start_fails_for_nonexistent_binary() {
    let client = LspClient::new(
        "test".to_string(),
        "nonexistent-lsp-binary-xyz".to_string(),
        vec![],
    );

    // Starting a client with a nonexistent binary should transition to Crashed
    let result = client.start().await;
    assert!(result.is_err());
    assert_eq!(client.get_state().await, LspClientState::Crashed);
}

#[tokio::test]
async fn test_client_invalid_transition_stop_from_stopped() {
    let client = LspClient::new("rust".to_string(), "rust-analyzer".to_string(), vec![]);

    // Attempting to stop a client that's already Stopped is an invalid transition
    let _result = client.stop().await;
    // The stop method forces a transition to Stopping then Stopped, but the
    // initial Stopped -> ShuttingDown transition should fail internally.
    // However stop() handles this gracefully by force-setting state.
    // After stop(), the client should be in Stopped state.
    assert_eq!(client.get_state().await, LspClientState::Stopped);
}

#[tokio::test]
async fn test_client_is_running_from_stopped() {
    let client = LspClient::new("rust".to_string(), "rust-analyzer".to_string(), vec![]);
    assert!(!client.is_running().await);
}

// ── LspPoolStats tests ──────────────────────────────────────────────────────

#[test]
fn test_pool_stats_debug_clone() {
    let stats = LspPoolStats {
        total_servers: 3,
        total_languages: 2,
    };

    // Debug trait
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("total_servers"));
    assert!(debug_str.contains("total_languages"));

    // Clone
    let cloned = stats.clone();
    assert_eq!(cloned.total_servers, 3);
    assert_eq!(cloned.total_languages, 2);
}

// ── Cross-component: LspManager + LspClientState ────────────────────────────

#[tokio::test]
async fn test_manager_client_state_integration() {
    let manager = LspManager::new();
    manager
        .register_server("rust", "rust-analyzer", vec![])
        .await;

    let client = manager.get_client("rust").await.unwrap();

    // Client registered via manager should start in Stopped state
    assert_eq!(client.get_state().await, LspClientState::Stopped);
    assert!(!client.is_running().await);
}

#[tokio::test]
async fn test_manager_with_pool_shared_config() {
    // Test that LspManager and LspServerPool can coexist with separate configurations
    let manager = LspManager::new();
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);

    // Register the same language in both
    manager
        .register_server("rust", "rust-analyzer", vec![])
        .await;
    pool.register_server("rust".to_string(), "rust-analyzer".to_string(), vec![])
        .await;

    // Manager should have the client
    assert!(manager.get_client("rust").await.is_some());

    // Pool should list no running servers yet
    assert!(pool.list_servers().await.is_empty());
}
