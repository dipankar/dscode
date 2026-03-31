//! Integration tests for dscode-terminal crate.
//!
//! Tests multiple terminal components working together: TerminalManager,
//! TerminalEventSender trait, TerminalProfile, and TerminalOptions.

use dscode_terminal::{
    TerminalEventSender, TerminalInfo, TerminalManager, TerminalOptions, TerminalProfile, TerminalState,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ── Mock TerminalEventSender for integration testing ────────────────────────

/// A mock event sender that records all events for verification.
struct MockEventSender {
    outputs: Arc<Mutex<Vec<(String, String)>>>,
    closes: Arc<Mutex<Vec<String>>>,
}

impl MockEventSender {
    fn new() -> Self {
        Self {
            outputs: Arc::new(Mutex::new(Vec::new())),
            closes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_outputs(&self) -> Vec<(String, String)> {
        self.outputs.lock().unwrap().clone()
    }

    fn get_closes(&self) -> Vec<String> {
        self.closes.lock().unwrap().clone()
    }

    fn output_count(&self) -> usize {
        self.outputs.lock().unwrap().len()
    }

    fn close_count(&self) -> usize {
        self.closes.lock().unwrap().len()
    }
}

impl TerminalEventSender for MockEventSender {
    fn send_output(&self, terminal_id: &str, data: &str) {
        self.outputs
            .lock()
            .unwrap()
            .push((terminal_id.to_string(), data.to_string()));
    }

    fn send_close(&self, terminal_id: &str) {
        self.closes.lock().unwrap().push(terminal_id.to_string());
    }
}

// ── TerminalManager creation tests ────────────────────────────────────────────

#[test]
fn test_terminal_manager_new_empty() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));
    assert!(manager.list_terminals().is_empty());
    assert!(manager.list_profiles().is_empty());
}

#[test]
fn test_terminal_manager_with_sender() {
    let sender = MockEventSender::new();
    let _manager = TerminalManager::new(Box::new(sender));
    // Manager should be created successfully with the mock sender
}

// ── Mock TerminalEventSender integration tests ──────────────────────────────

#[test]
fn test_mock_sender_records_output() {
    let sender = MockEventSender::new();
    sender.send_output("term-1", "Hello, World!");
    sender.send_output("term-1", "Second line");
    sender.send_output("term-2", "Other terminal");

    assert_eq!(sender.output_count(), 3);
    assert_eq!(sender.get_closes().len(), 0);

    let outputs = sender.get_outputs();
    assert_eq!(outputs[0], ("term-1".to_string(), "Hello, World!".to_string()));
    assert_eq!(outputs[1], ("term-1".to_string(), "Second line".to_string()));
    assert_eq!(outputs[2], ("term-2".to_string(), "Other terminal".to_string()));
}

#[test]
fn test_mock_sender_records_close() {
    let sender = MockEventSender::new();
    sender.send_close("term-1");
    sender.send_close("term-2");

    assert_eq!(sender.close_count(), 2);
    let closes = sender.get_closes();
    assert_eq!(closes[0], "term-1");
    assert_eq!(closes[1], "term-2");
}

#[test]
fn test_mock_sender_through_trait_object() {
    let sender: Box<dyn TerminalEventSender> = Box::new(MockEventSender::new());
    // Should be able to call trait methods through the trait object
    sender.send_output("term-1", "data");
    sender.send_close("term-1");
}

#[test]
fn test_mock_sender_through_arc() {
    // Test the pattern used by SharedEventSender internally
    let sender: Arc<dyn TerminalEventSender> = Arc::new(MockEventSender::new());
    sender.send_output("term-1", "data from arc");
    sender.send_close("term-1");
}

#[test]
fn test_mock_sender_empty_state() {
    let sender = MockEventSender::new();
    assert_eq!(sender.output_count(), 0);
    assert_eq!(sender.close_count(), 0);
}

#[test]
fn test_mock_sender_large_data() {
    let sender = MockEventSender::new();
    let large_data = "x".repeat(100_000);
    sender.send_output("term-1", &large_data);

    assert_eq!(sender.output_count(), 1);
    let outputs = sender.get_outputs();
    assert_eq!(outputs[0].1.len(), 100_000);
}

// ── Terminal profiles configuration tests ────────────────────────────────────

#[test]
fn test_terminal_profile_creation() {
    let profile = TerminalProfile {
        id: "bash-profile".to_string(),
        name: "Bash".to_string(),
        shell: "/bin/bash".to_string(),
        args: vec!["-l".to_string()],
        env: HashMap::new(),
        cwd: None,
        icon: None,
        color: None,
    };

    assert_eq!(profile.id, "bash-profile");
    assert_eq!(profile.name, "Bash");
    assert_eq!(profile.shell, "/bin/bash");
    assert_eq!(profile.args, vec!["-l"]);
}

#[test]
fn test_terminal_profile_with_env() {
    let mut env = HashMap::new();
    env.insert("TERM".to_string(), "xterm-256color".to_string());
    env.insert("LANG".to_string(), "en_US.UTF-8".to_string());

    let profile = TerminalProfile {
        id: "zsh-profile".to_string(),
        name: "ZSH".to_string(),
        shell: "/bin/zsh".to_string(),
        args: vec![],
        env: env.clone(),
        cwd: Some("/home/user".to_string()),
        icon: Some("terminal".to_string()),
        color: Some("green".to_string()),
    };

    assert_eq!(profile.env.get("TERM").unwrap(), "xterm-256color");
    assert_eq!(profile.cwd, Some("/home/user".to_string()));
    assert_eq!(profile.icon, Some("terminal".to_string()));
    assert_eq!(profile.color, Some("green".to_string()));
}

#[test]
fn test_terminal_profile_register_and_list() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));

    let profile1 = TerminalProfile {
        id: "bash".to_string(),
        name: "Bash".to_string(),
        shell: "/bin/bash".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        icon: None,
        color: None,
    };

    let profile2 = TerminalProfile {
        id: "zsh".to_string(),
        name: "ZSH".to_string(),
        shell: "/bin/zsh".to_string(),
        args: vec!["-l".to_string()],
        env: HashMap::new(),
        cwd: None,
        icon: None,
        color: None,
    };

    assert!(manager.register_profile(profile1).is_ok());
    assert!(manager.register_profile(profile2).is_ok());

    let profiles = manager.list_profiles();
    assert_eq!(profiles.len(), 2);
}

#[test]
fn test_terminal_profile_duplicate_rejected() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));

    let profile = TerminalProfile {
        id: "duplicate".to_string(),
        name: "Duplicate".to_string(),
        shell: "/bin/bash".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        icon: None,
        color: None,
    };

    assert!(manager.register_profile(profile.clone()).is_ok());
    let result = manager.register_profile(profile);
    assert!(result.is_err());
}

#[test]
fn test_terminal_profile_unregister() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));

    let profile = TerminalProfile {
        id: "temp-profile".to_string(),
        name: "Temp".to_string(),
        shell: "/bin/bash".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        icon: None,
        color: None,
    };

    manager.register_profile(profile).unwrap();
    assert!(manager.get_profile("temp-profile").is_ok());

    manager.unregister_profile("temp-profile").unwrap();
    assert!(manager.get_profile("temp-profile").is_err());
}

#[test]
fn test_terminal_profile_unregister_nonexistent() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));
    let result = manager.unregister_profile("nonexistent");
    assert!(result.is_err());
}

// ── Terminal options configuration tests ──────────────────────────────────────

#[test]
fn test_terminal_options_default() {
    let options = TerminalOptions {
        name: None,
        shell_path: None,
        shell_args: vec![],
        cwd: None,
        env: HashMap::new(),
        profile_id: None,
    };

    assert!(options.name.is_none());
    assert!(options.shell_path.is_none());
    assert!(options.shell_args.is_empty());
    assert!(options.cwd.is_none());
    assert!(options.env.is_empty());
    assert!(options.profile_id.is_none());
}

#[test]
fn test_terminal_options_with_profile() {
    let options = TerminalOptions {
        name: Some("My Terminal".to_string()),
        shell_path: Some("/bin/fish".to_string()),
        shell_args: vec!["-l".to_string()],
        cwd: Some("/home/user/projects".to_string()),
        env: HashMap::new(),
        profile_id: Some("fish-profile".to_string()),
    };

    assert_eq!(options.name, Some("My Terminal".to_string()));
    assert_eq!(options.shell_path, Some("/bin/fish".to_string()));
    assert_eq!(options.profile_id, Some("fish-profile".to_string()));
}

#[test]
fn test_terminal_options_with_env() {
    let mut env = HashMap::new();
    env.insert("NODE_ENV".to_string(), "development".to_string());
    env.insert("PORT".to_string(), "3000".to_string());

    let options = TerminalOptions {
        name: None,
        shell_path: None,
        shell_args: vec![],
        cwd: None,
        env,
        profile_id: None,
    };

    assert_eq!(options.env.get("NODE_ENV").unwrap(), "development");
    assert_eq!(options.env.get("PORT").unwrap(), "3000");
}

// ── TerminalInfo tests ───────────────────────────────────────────────────────

#[test]
fn test_terminal_info_creation() {
    let info = TerminalInfo {
        id: "terminal-1".to_string(),
        name: "Main Terminal".to_string(),
        shell: "/bin/bash".to_string(),
        cwd: "/home/user".to_string(),
    };

    assert_eq!(info.id, "terminal-1");
    assert_eq!(info.name, "Main Terminal");
    assert_eq!(info.shell, "/bin/bash");
    assert_eq!(info.cwd, "/home/user");
}

#[test]
fn test_terminal_info_clone() {
    let info = TerminalInfo {
        id: "terminal-1".to_string(),
        name: "Main".to_string(),
        shell: "/bin/zsh".to_string(),
        cwd: "/tmp".to_string(),
    };

    let cloned = info.clone();
    assert_eq!(cloned.id, info.id);
    assert_eq!(cloned.name, info.name);
    assert_eq!(cloned.shell, info.shell);
    assert_eq!(cloned.cwd, info.cwd);
}

// ── TerminalState tests ───────────────────────────────────────────────────────

#[test]
fn test_terminal_state_variants() {
    assert_eq!(TerminalState::Created, TerminalState::Created);
    assert_eq!(TerminalState::Running, TerminalState::Running);
    assert_eq!(TerminalState::ShuttingDown, TerminalState::ShuttingDown);
    assert_eq!(TerminalState::Closed, TerminalState::Closed);

    assert_ne!(TerminalState::Created, TerminalState::Running);
    assert_ne!(TerminalState::Running, TerminalState::ShuttingDown);
    assert_ne!(TerminalState::ShuttingDown, TerminalState::Closed);
}

#[test]
fn test_terminal_state_copy_equality() {
    let state1 = TerminalState::Running;
    let state2 = state1; // Copy
    assert_eq!(state1, state2);
}

// ── Serde round-trip tests ───────────────────────────────────────────────────

#[test]
fn test_terminal_info_serde_roundtrip() {
    let info = TerminalInfo {
        id: "terminal-42".to_string(),
        name: "Test Terminal".to_string(),
        shell: "/bin/fish".to_string(),
        cwd: "/home/user/code".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: TerminalInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, info.id);
    assert_eq!(deserialized.name, info.name);
    assert_eq!(deserialized.shell, info.shell);
    assert_eq!(deserialized.cwd, info.cwd);
}

#[test]
fn test_terminal_profile_serde_roundtrip() {
    let mut env = HashMap::new();
    env.insert("EDITOR".to_string(), "vim".to_string());

    let profile = TerminalProfile {
        id: "vim-profile".to_string(),
        name: "Vim Shell".to_string(),
        shell: "/bin/bash".to_string(),
        args: vec!["-l".to_string()],
        env: env.clone(),
        cwd: Some("/home/user".to_string()),
        icon: Some("terminal".to_string()),
        color: Some("blue".to_string()),
    };

    let json = serde_json::to_string(&profile).unwrap();
    let deserialized: TerminalProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, profile.id);
    assert_eq!(deserialized.name, profile.name);
    assert_eq!(deserialized.shell, profile.shell);
    assert_eq!(deserialized.args, profile.args);
    assert_eq!(deserialized.env, profile.env);
    assert_eq!(deserialized.cwd, profile.cwd);
    assert_eq!(deserialized.icon, profile.icon);
    assert_eq!(deserialized.color, profile.color);
}

#[test]
fn test_terminal_options_serde_roundtrip() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());

    let options = TerminalOptions {
        name: Some("Custom Term".to_string()),
        shell_path: Some("/bin/zsh".to_string()),
        shell_args: vec!["-l".to_string(), "-i".to_string()],
        cwd: Some("/tmp".to_string()),
        env: env.clone(),
        profile_id: Some("zsh-profile".to_string()),
    };

    let json = serde_json::to_string(&options).unwrap();
    let deserialized: TerminalOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, options.name);
    assert_eq!(deserialized.shell_path, options.shell_path);
    assert_eq!(deserialized.shell_args, options.shell_args);
    assert_eq!(deserialized.cwd, options.cwd);
    assert_eq!(deserialized.env, options.env);
    assert_eq!(deserialized.profile_id, options.profile_id);
}

#[test]
fn test_terminal_options_camel_case_serialization() {
    let options = TerminalOptions {
        name: None,
        shell_path: Some("/bin/bash".to_string()),
        shell_args: vec![],
        cwd: None,
        env: HashMap::new(),
        profile_id: Some("default".to_string()),
    };

    let json = serde_json::to_string(&options).unwrap();
    // With #[serde(rename_all = "camelCase")], keys should be camelCase
    assert!(json.contains("shellPath"), "Expected camelCase 'shellPath' in JSON: {}", json);
    assert!(json.contains("shellArgs"), "Expected camelCase 'shellArgs' in JSON: {}", json);
    assert!(json.contains("profileId"), "Expected camelCase 'profileId' in JSON: {}", json);
}

// ── Cross-component integration: profiles + options ──────────────────────────

#[test]
fn test_manager_profile_lookup_after_register() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));

    let profile = TerminalProfile {
        id: "dev-profile".to_string(),
        name: "Dev Shell".to_string(),
        shell: "/bin/bash".to_string(),
        args: vec!["-l".to_string()],
        env: HashMap::new(),
        cwd: Some("/home/user/dev".to_string()),
        icon: Some("code".to_string()),
        color: None,
    };

    manager.register_profile(profile).unwrap();

    // Look up the profile
    let found = manager.get_profile("dev-profile").unwrap();
    assert_eq!(found.id, "dev-profile");
    assert_eq!(found.name, "Dev Shell");
    assert_eq!(found.shell, "/bin/bash");
    assert_eq!(found.args, vec!["-l"]);
    assert_eq!(found.cwd, Some("/home/user/dev".to_string()));
}

#[test]
fn test_manager_multiple_profiles_with_different_shells() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));

    let profiles = vec![
        TerminalProfile {
            id: "bash".to_string(),
            name: "Bash".to_string(),
            shell: "/bin/bash".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            icon: None,
            color: None,
        },
        TerminalProfile {
            id: "zsh".to_string(),
            name: "ZSH".to_string(),
            shell: "/bin/zsh".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            icon: None,
            color: None,
        },
        TerminalProfile {
            id: "fish".to_string(),
            name: "Fish".to_string(),
            shell: "/usr/bin/fish".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            icon: None,
            color: None,
        },
    ];

    for profile in profiles {
        assert!(manager.register_profile(profile).is_ok());
    }

    let listed = manager.list_profiles();
    assert_eq!(listed.len(), 3);

    // Each profile should be retrievable
    assert!(manager.get_profile("bash").is_ok());
    assert!(manager.get_profile("zsh").is_ok());
    assert!(manager.get_profile("fish").is_ok());
}

#[test]
fn test_manager_close_all_when_empty() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));
    // Closing all terminals when none exist should not panic
    manager.close_all();
    assert!(manager.list_terminals().is_empty());
}

#[test]
fn test_manager_close_nonexistent_terminal() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));
    let result = manager.close_terminal("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_manager_write_to_nonexistent_terminal() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));
    let result = manager.write_to_terminal("nonexistent", "hello");
    assert!(result.is_err());
}

#[test]
fn test_manager_resize_nonexistent_terminal() {
    let manager = TerminalManager::new(Box::new(MockEventSender::new()));
    let result = manager.resize_terminal("nonexistent", 80, 24);
    assert!(result.is_err());
}

// ── EventSender + TerminalManager integration ────────────────────────────────

#[test]
fn test_mock_sender_independent_of_manager() {
    // Verify the mock sender records events even without a real PTY
    let sender = MockEventSender::new();

    // Simulate terminal output
    sender.send_output("term-1", "Hello from terminal\r\n");
    sender.send_output("term-1", "prompt> ");
    sender.send_close("term-1");

    assert_eq!(sender.output_count(), 2);
    assert_eq!(sender.close_count(), 1);

    let outputs = sender.get_outputs();
    assert_eq!(outputs[0].1, "Hello from terminal\r\n");
    assert_eq!(outputs[1].1, "prompt> ");
}

#[test]
fn test_event_sender_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let sender = Arc::new(MockEventSender::new());

    let mut handles = vec![];
    for i in 0..4 {
        let sender_clone = Arc::clone(&sender);
        handles.push(thread::spawn(move || {
            let term_id = format!("term-{}", i);
            sender_clone.send_output(&term_id, "data");
            sender_clone.send_close(&term_id);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(sender.output_count(), 4);
    assert_eq!(sender.close_count(), 4);
}