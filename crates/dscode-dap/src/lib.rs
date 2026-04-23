//! # dscode-dap
//!
//! Debug Adapter Protocol client, manager, and pool for DSCode.
//!
//! This crate provides a complete DAP client implementation with:

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(dead_code)]
//! - [`DebugAdapter`] — State-machine-based DAP client that manages a debug adapter process
//! - [`DebugManager`] — Registry for debug sessions and breakpoints
//! - [`DebugAdapterPool`] — Connection pool with one adapter per debug session
//!
//! # State Machine
//!
//! The `DebugAdapter` follows a strict lifecycle:
//!
//! ```text
//! Stopped → Starting → Initializing → Configured → Running
//!    ▲          │           │              │           │
//!    │          ▼           ▼              ▼           ▼
//!    │       Crashed ◄── Crashed ◄─── Crashed ◄── Crashed
//!    │                                                   │
//!    │                                       ShuttingDown│
//!    │                                           │      │
//!    └───────────────────────────────────────────┘      │
//!                                                ▲       │
//!                                                └───────┘
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use dscode_dap::{DebugAdapterPool, DebugManager, DebugSession, DebugState};
//!
//! # async fn example() {
//! let manager = DebugManager::new();
//! let session_id = manager.create_session("main".to_string(), "cppdbg".to_string()).unwrap();
//!
//! let pool = DebugAdapterPool::new();
//! pool.register_adapter("cppdbg".to_string(), "/usr/bin/gdb".to_string(), vec![]).await;
//!
//! let session = manager.get_session(&session_id).unwrap();
//! let adapter = pool.create_session(session).await.unwrap();
//! # }
//! ```

mod adapter;
mod error;
mod manager;
mod pool;
mod types;

pub use adapter::{DebugAdapter, DebugAdapterState};
pub use error::DapError;
pub use manager::DebugManager;
pub use pool::{DebugAdapterPool, DebugPoolStats};
pub use types::*;
