use crate::TerminalEventSender;
use tauri::{AppHandle, Emitter};

/// [`TerminalEventSender`] implementation that forwards events through the
/// Tauri event system.
///
/// Requires the `tauri` feature flag.
///
/// # Events emitted
///
/// | Event name                    | Payload          |
/// |-------------------------------|------------------|
/// | `terminal-data:{terminal_id}` | `{ "data": … }` |
/// | `terminal-closed`             | `{terminal_id}`  |
pub struct TauriEventSender {
    app_handle: AppHandle,
}

impl TauriEventSender {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl TerminalEventSender for TauriEventSender {
    fn send_output(&self, terminal_id: &str, data: &str) {
        let event_name = format!("terminal-data:{}", terminal_id);
        self.app_handle
            .emit(&event_name, serde_json::json!({ "data": data }))
            .ok();
    }

    fn send_close(&self, terminal_id: &str) {
        self.app_handle.emit("terminal-closed", terminal_id).ok();
    }
}