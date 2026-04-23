use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub fn min_log_level() -> LogLevel {
    #[cfg(debug_assertions)]
    {
        LogLevel::Trace
    }
    #[cfg(not(debug_assertions))]
    {
        LogLevel::Warn
    }
}

pub fn should_log(level: LogLevel) -> bool {
    level >= min_log_level()
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

pub struct LogEntry {
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, module: &str, message: &str) -> Self {
        Self { level, module: module.to_string(), message: message.to_string() }
    }

    pub fn log(&self) {
        if should_log(self.level) {
            match self.level {
                LogLevel::Trace => tracing::trace!(module = %self.module, "{}", self.message),
                LogLevel::Debug => tracing::debug!(module = %self.module, "{}", self.message),
                LogLevel::Info => tracing::info!(module = %self.module, "{}", self.message),
                LogLevel::Warn => tracing::warn!(module = %self.module, "{}", self.message),
                LogLevel::Error => tracing::error!(module = %self.module, "{}", self.message),
            }
        }
    }
}

pub fn log(level: LogLevel, module: &str, message: &str) {
    LogEntry::new(level, module, message).log();
}

/// Returns the log directory path for the application.
///
/// Uses the platform-specific cache directory:
/// - macOS: `~/Library/Caches/dscode/logs/`
/// - Linux: `~/.cache/dscode/logs/`
/// - Windows: `C:\Users\<user>\AppData\Local\dscode\logs\`
fn log_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("dscode").join("logs"))
}

pub fn init() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("dscode=info"));

    let pid = std::process::id();
    let version = env!("CARGO_PKG_VERSION");
    let os_info = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);

    // Set up the layered subscriber with both stdout and file output
    use tracing_subscriber::fmt;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let stdout_layer = fmt::layer().with_writer(std::io::stdout);

    if let Some(log_path) = log_dir() {
        // Ensure the log directory exists
        if let Err(e) = std::fs::create_dir_all(&log_path) {
            eprintln!("Warning: could not create log directory {:?}: {}", log_path, e);
        }

        // Rolling file appender: daily rotation, keeps log files in the configured directory
        let file_appender = tracing_appender::rolling::daily(&log_path, "dscode.log");

        let file_layer = fmt::layer().with_writer(file_appender).with_ansi(false);

        tracing_subscriber::registry().with(filter).with(stdout_layer).with(file_layer).init();
    } else {
        // Fallback: stdout only if no cache directory can be determined
        tracing_subscriber::registry().with(filter).with(stdout_layer).init();
    }

    // Log structured fields: version, pid, and OS info
    tracing::info!(
        version = %version,
        pid = pid,
        os = %os_info,
        "DSCode logging initialized"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn test_should_log() {
        #[cfg(debug_assertions)]
        {
            assert!(should_log(LogLevel::Trace));
            assert!(should_log(LogLevel::Debug));
        }

        assert!(should_log(LogLevel::Warn));
        assert!(should_log(LogLevel::Error));
    }

    #[test]
    fn test_log_dir_returns_some_path() {
        let path = log_dir();
        // On CI or some environments, dirs::cache_dir() may return None,
        // so just verify the function doesn't panic.
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("dscode"));
            assert!(p.to_string_lossy().contains("logs"));
        }
    }
}
