use std::cmp::Ordering;
use std::ops::Deref;

use yansi::Color;

lazy_static! {
    /// The trace log level. Level = 1000.
    pub static ref TRACE: LogLevel = LogLevel::new(1000, "trace".to_string(), Color::Fixed(102));

    /// The debug log level. Level = 2000.
    pub static ref DEBUG: LogLevel = LogLevel::new(2000, "debug".to_string(), Color::Green);

    /// The info log level. Level = 3000.
    pub static ref INFO: LogLevel = LogLevel::new(3000, "info".to_string(), Color::Blue);

    /// The warn log level. Level = 4000.
    pub static ref WARN: LogLevel = LogLevel::new(4000, "warn".to_string(), Color::Yellow);

    /// The error log level. Level = 5000.
    pub static ref ERROR: LogLevel = LogLevel::new(5000, "error".to_string(), Color::Red);
}

/// The different levels of logging.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LogLevel {
    level: usize,
    tag: String,
    color: Color,
}

impl LogLevel {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new log level.
    pub const fn new(level: usize, tag: String, color: Color) -> LogLevel {
        LogLevel { level, tag, color }
    }

    // GETTERS ----------------------------------------------------------------

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn color(&self) -> Color {
        self.color
    }

    // STATIC METHODS ---------------------------------------------------------

    pub fn trace() -> LogLevel {
        TRACE.deref().clone()
    }

    pub fn debug() -> LogLevel {
        DEBUG.deref().clone()
    }

    pub fn info() -> LogLevel {
        INFO.deref().clone()
    }

    pub fn warn() -> LogLevel {
        WARN.deref().clone()
    }

    pub fn error() -> LogLevel {
        ERROR.deref().clone()
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.level.partial_cmp(&other.level)
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.level.cmp(&other.level)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;

    #[test]
    fn test_order() {
        assert!(
            TRACE.deref() < DEBUG.deref(),
            "TRACE is not less than DEBUG"
        );
        assert!(DEBUG.deref() < INFO.deref(), "DEBUG is not less than INFO");
        assert!(INFO.deref() < WARN.deref(), "INFO is not less than WARN");
        assert!(WARN.deref() < ERROR.deref(), "WARN is not less than ERROR");
    }
}
