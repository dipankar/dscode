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

pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
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
}
