use std::cmp::Ordering;

use yansi::Color;

/// The trace log level. Level = 10.
const TRACE: LogLevel = LogLevel::new(10, "trace", Color::Fixed(102));

/// The debug log level. Level = 20.
const DEBUG: LogLevel = LogLevel::new(20, "debug", Color::Green);

/// The info log level. Level = 30.
const INFO: LogLevel = LogLevel::new(30, "info", Color::Blue);

/// The warn log level. Level = 40.
const WARN: LogLevel = LogLevel::new(40, "warn", Color::Yellow);

/// The error log level. Level = 50.
const ERROR: LogLevel = LogLevel::new(50, "error", Color::Red);

/// The different levels of logging.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LogLevel {
    level: u8,
    color: Color,
    tag: &'static str,
}

impl LogLevel {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new log level.
    pub const fn new(level: u8, tag: &'static str, color: Color) -> LogLevel {
        LogLevel { level, tag, color }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns a number that defines an order between log levels.
    pub fn level(&self) -> u8 {
        self.level
    }

    /// Returns the tag that represents the log level.
    pub fn tag(&self) -> &'static str {
        self.tag
    }

    /// Returns the color that represents the log level.
    pub fn color(&self) -> Color {
        self.color
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Returns the TRACE log level.
    pub const fn trace() -> LogLevel {
        TRACE
    }

    /// Returns the DEBUG log level.
    pub const fn debug() -> LogLevel {
        DEBUG
    }

    /// Returns the INFO log level.
    pub const fn info() -> LogLevel {
        INFO
    }

    /// Returns the WARN log level.
    pub const fn warn() -> LogLevel {
        WARN
    }

    /// Returns the ERROR log level.
    pub const fn error() -> LogLevel {
        ERROR
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    use super::*;

    #[test]
    fn test_order() {
        assert!(TRACE < DEBUG, "TRACE is not less than DEBUG");
        assert!(DEBUG < INFO, "DEBUG is not less than INFO");
        assert!(INFO < WARN, "INFO is not less than WARN");
        assert!(WARN < ERROR, "WARN is not less than ERROR");
    }
}
