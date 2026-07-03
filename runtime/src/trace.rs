use serde::{Deserialize, Serialize};

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// 一条执行日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub timestamp: u64,
    pub event: String,
    pub details: String,
    pub level: LogLevel,
}

/// Trace 日志收集器
#[derive(Clone)]
pub struct TraceLogger {
    entries: Vec<TraceEntry>,
}

impl TraceLogger {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    fn now() -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
        }
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::now() as u64
        }
    }

    pub fn log(&mut self, event: String, details: String, level: LogLevel) {
        self.entries.push(TraceEntry {
            timestamp: Self::now(),
            event,
            details,
            level,
        });
    }

    pub fn info(&mut self, event: String, details: String) {
        self.log(event, details, LogLevel::Info);
    }

    pub fn warn(&mut self, event: String, details: String) {
        self.log(event, details, LogLevel::Warn);
    }

    pub fn error(&mut self, event: String, details: String) {
        self.log(event, details, LogLevel::Error);
    }

    pub fn all(&self) -> &[TraceEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for TraceLogger {
    fn default() -> Self {
        Self::new()
    }
}
