//! # dscode-terminal
//!
//! Terminal manager and PTY lifecycle management for DSCode.
//!
//! This crate provides a portable PTY-backed terminal manager that can be used

// TODO: re-enable warn(missing_docs) after filling in remaining docs
#![allow(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(dead_code)]
//! independently of any UI framework. Event forwarding is abstracted behind the
//! [`TerminalEventSender`] trait, with an optional Tauri implementation gated
//! behind the `tauri` feature flag.
//!
//! ## Feature Flags
//!
//! - **tauri** — Enables [`TauriEventSender`], a [`TerminalEventSender`]
//!   implementation that forwards PTY output and close events to a Tauri
//!   frontend via the Tauri event system.

mod error;
mod manager;

pub use error::TerminalError;
pub use manager::{
    TerminalInfo, TerminalInstance, TerminalManager, TerminalOptions, TerminalProfile,
    TerminalState,
};

/// Trait abstracting how terminal events are forwarded to a consumer.
///
/// Implement this trait to integrate the terminal manager with your preferred
/// UI or event system. The crate ships a Tauri-based implementation behind the
/// `tauri` feature flag.
pub trait TerminalEventSender: Send + Sync {
    /// Forward PTY output data for the given terminal.
    fn send_output(&self, terminal_id: &str, data: &str);

    /// Signal that the terminal with the given ID has closed.
    fn send_close(&self, terminal_id: &str);
}

// ── Tauri integration ──────────────────────────────────────────────────────

#[cfg(feature = "tauri")]
mod tauri_sender;

#[cfg(feature = "tauri")]
pub use tauri_sender::TauriEventSender;
