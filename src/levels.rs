use std::cmp::Ordering;

use arcstr::ArcStr;
use yansi::Color;

/// The trace log level. Level = 1000.
pub static TRACE: LogLevel = LogLevel::new(1000, arcstr::literal!("trace"), Color::Fixed(102));

/// The debug log level. Level = 2000.
pub static DEBUG: LogLevel = LogLevel::new(2000, arcstr::literal!("debug"), Color::Green);

/// The info log level. Level = 3000.
pub static INFO: LogLevel = LogLevel::new(3000, arcstr::literal!("info"), Color::Blue);

/// The warn log level. Level = 4000.
pub static WARN: LogLevel = LogLevel::new(4000, arcstr::literal!("warn"), Color::Yellow);

/// The error log level. Level = 5000.
pub static ERROR: LogLevel = LogLevel::new(5000, arcstr::literal!("error"), Color::Red);

/// The different levels of logging.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LogLevel {
    level: usize,
    tag: ArcStr,
    color: Color,
}

impl LogLevel {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new log level.
    pub const fn new(level: usize, tag: ArcStr, color: Color) -> LogLevel {
        LogLevel { level, tag, color }
    }

    // GETTERS ----------------------------------------------------------------

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn tag(&self) -> &ArcStr {
        &self.tag
    }

    pub fn color(&self) -> Color {
        self.color
    }

    // STATIC METHODS ---------------------------------------------------------

    pub fn trace() -> LogLevel {
        TRACE.clone()
    }

    pub fn debug() -> LogLevel {
        DEBUG.clone()
    }

    pub fn info() -> LogLevel {
        INFO.clone()
    }

    pub fn warn() -> LogLevel {
        WARN.clone()
    }

    pub fn error() -> LogLevel {
        ERROR.clone()
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
    use super::*;

    #[test]
    fn test_order() {
        assert!(TRACE < DEBUG, "TRACE is not less than DEBUG");
        assert!(DEBUG < INFO, "DEBUG is not less than INFO");
        assert!(INFO < WARN, "INFO is not less than WARN");
        assert!(WARN < ERROR, "WARN is not less than ERROR");
    }
}
