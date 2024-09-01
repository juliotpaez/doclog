use std::cmp::Ordering;

use yansi::Color;

/// The trace log level. Level = 10.
const TRACE: LogLevel = LogLevel::new(10, Color::Fixed(102), "trace", '•');

/// The debug log level. Level = 20.
const DEBUG: LogLevel = LogLevel::new(20, Color::Green, "debug", '•');

/// The info log level. Level = 30.
const INFO: LogLevel = LogLevel::new(30, Color::Blue, "info", 'ℹ');

/// The warn log level. Level = 40.
const WARN: LogLevel = LogLevel::new(40, Color::Yellow, "warn", '⚠');

/// The error log level. Level = 50.
const ERROR: LogLevel = LogLevel::new(50, Color::Red, "error", '×');

/// The different levels of logging.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LogLevel {
    level: u8,
    color: Color,
    tag: &'static str,
    symbol: char,
}

impl LogLevel {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new log level.
    pub const fn new(level: u8, color: Color, tag: &'static str, symbol: char) -> LogLevel {
        LogLevel {
            level,
            tag,
            color,
            symbol,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns a number that defines an order between log levels.
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Returns the tag that represents the log level.
    pub const fn tag(&self) -> &'static str {
        self.tag
    }

    /// Returns the color that represents the log level.
    pub const fn color(&self) -> Color {
        self.color
    }

    /// Returns the symbol that represents the log level.
    pub const fn symbol(&self) -> char {
        self.symbol
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
