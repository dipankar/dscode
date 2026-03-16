# Telemetry & Analytics

## Overview

DSCode includes a comprehensive, **privacy-first** telemetry system for performance monitoring, usage analytics, and crash reporting.

## Architecture

```
┌──────────────────────────────────────────┐
│  All Processes (Main, Extension, LSP)    │
│  emit_telemetry_event()                  │
└────────────┬─────────────────────────────┘
             │ nng pipeline
┌────────────▼─────────────────────────────┐
│  Telemetry Service (separate process)    │
│  ┌────────────────────────────────────┐  │
│  │ Event Collection                   │  │
│  │ ┌────────┐  ┌──────────────────┐  │  │
│  │ │ Filter │→│ SQLite Storage    │  │  │
│  │ └────────┘  └──────────────────┘  │  │
│  │                                    │  │
│  │ ┌─────────────────────────────┐   │  │
│  │ │ Analytics Engine            │   │  │
│  │ │ (aggregation, insights)     │   │  │
│  │ └─────────────────────────────┘   │  │
│  │                                    │  │
│  │ ┌─────────────────────────────┐   │  │
│  │ │ Upload Service (opt-in)     │   │  │
│  │ │ (batched, encrypted)        │   │  │
│  │ └─────────────────────────────┘   │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

## Event Types

```rust
#[derive(Archive, Serialize, Deserialize)]
pub enum TelemetryEvent {
    Startup {
        duration_ms: u64,
        cold_start: bool,
    },

    CommandExecuted {
        command_id: String,
        source: CommandSource,
    },

    ExtensionActivated {
        extension_id: String,
        duration_ms: u64,
    },

    ExtensionError {
        extension_id: String,
        error_type: String,
    },

    LSPRequest {
        server: String,
        method: String,
        latency_ms: u64,
        success: bool,
    },

    FileOpened {
        language: String,
        size_bytes: u64,
        extension: String,
    },

    ErrorOccurred {
        error_type: String,
        stack_trace: String,
        context: HashMap<String, String>,
    },

    PerformanceMetric {
        metric: String,
        value: f64,
        unit: String,
    },
}
```

## Privacy Controls

```rust
pub struct TelemetryConfig {
    /// Master switch
    pub enabled: bool,

    /// What to collect
    pub collect_usage: bool,
    pub collect_errors: bool,
    pub collect_performance: bool,

    /// Upload to servers (separate from collection)
    pub upload_enabled: bool,
    pub upload_interval: Duration,

    /// Anonymization level
    pub anonymize_paths: bool,
    pub anonymize_identifiers: bool,
}
```

## Local Analytics Dashboard

Built-in UI showing:
- Startup time trends
- Extension performance impact
- LSP latency per language
- Memory usage over time
- Command usage frequency
- Error rates

```rust
pub struct AnalyticsDashboard {
    pub fn render(&self, ui: &mut egui::Ui) {
        ui.heading("Performance Insights");

        // Startup time chart
        self.render_startup_chart(ui);

        // Extension impact
        self.render_extension_impact(ui);

        // LSP performance
        self.render_lsp_performance(ui);
    }
}
```

## Crash Reporting

```rust
pub fn install_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = backtrace::Backtrace::new();

        let event = TelemetryEvent::ErrorOccurred {
            error_type: "panic".to_string(),
            stack_trace: format!("{:?}", backtrace),
            context: HashMap::from([
                ("location", panic_info.location().to_string()),
                ("message", panic_info.to_string()),
            ]),
        };

        // Auto-save all open files before crashing
        emergency_save_all();

        // Send crash report
        telemetry::send_event(event);

        // Show crash dialog
        show_crash_dialog(panic_info, backtrace);
    }));
}
```

## GDPR Compliance

- All telemetry opt-in
- Data export available
- Data deletion on request
- Transparent about what's collected
- Local-first (upload is optional)
