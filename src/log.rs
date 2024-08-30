use std::fmt::Display;
use std::fs;
use std::path::Path;

use crate::blocks::LogBlock;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::{LogContent, LogLevel};

/// A configured log.
#[derive(Debug, Clone)]
pub struct Log<'a> {
    level: LogLevel,
    content: LogContent<'a>,
    cause: Option<Box<Log<'a>>>,
}

impl<'a> Log<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new log.
    pub fn new(level: LogLevel) -> Log<'a> {
        Log {
            level,
            content: LogContent::new(),
            cause: None,
        }
    }

    /// Builds a new log with a trace level.
    pub fn trace() -> Log<'a> {
        Self::new(LogLevel::trace())
    }

    /// Builds a new log with a debug level.
    pub fn debug() -> Log<'a> {
        Self::new(LogLevel::debug())
    }

    /// Builds a new log with an info level.
    pub fn info() -> Log<'a> {
        Self::new(LogLevel::info())
    }

    /// Builds a new log with a warn level.
    pub fn warn() -> Log<'a> {
        Self::new(LogLevel::warn())
    }

    /// Builds a new log with an error level.
    pub fn error() -> Log<'a> {
        Self::new(LogLevel::error())
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the log level.
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Returns the log content.
    pub fn content(&self) -> &LogContent<'a> {
        &self.content
    }

    /// Returns a mutable reference to the log content.
    pub fn content_mut(&mut self) -> &mut LogContent<'a> {
        &mut self.content
    }

    /// Returns the cause of this log.
    pub fn cause(&self) -> &Option<Box<Log<'a>>> {
        &self.cause
    }

    /// Returns a mutable reference to the cause of this log.
    pub fn cause_mut(&mut self) -> &mut Option<Box<Log<'a>>> {
        &mut self.cause
    }

    // GETTERS ----------------------------------------------------------------

    /// Sets the level of this log.
    pub fn set_level<F>(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Sets the cause of this log.
    pub fn set_cause<F>(mut self, builder: F) -> Self
    where
        F: FnOnce(Log) -> Log,
    {
        let new_log = Log::new(self.level);
        let new_log = builder(new_log);
        self.cause = Some(Box::new(new_log));
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a new block.
    pub fn add_block(mut self, block: impl Into<LogBlock<'a>>) -> Self {
        self.content = self.content.add_block(block.into());
        self
    }

    /// Logs in the console the plain text version of the log.
    pub fn log_plain_text(&self) {
        println!("{}", self.to_plain_text());
    }

    /// Logs in the console the styled text version of the log.
    pub fn log_styled_text(&self) {
        println!("{}", self.to_styled_text());
    }

    /// Logs in the console the text version of the log. Whether it is styled or plain text
    /// depends on whether the ANSI colors are supported in the executing terminal or not.
    pub fn log(&self) {
        println!("{}", self.to_text());
    }

    /// Appends the log into the specified file as plain text.
    pub fn append_plain_to_file(&self, file: &Path) -> std::io::Result<()> {
        let content = self.to_plain_text();
        fs::write(file, content)
    }

    /// Appends the log into the specified file as styled text.
    pub fn append_styled_to_file(&self, file: &Path) -> std::io::Result<()> {
        let content = self.to_plain_text();
        fs::write(file, content)
    }

    /// Returns the log as a plain text.
    pub fn to_plain_text(&self) -> String {
        self.print_to_string(self.level, PrinterFormat::Plain)
    }

    /// Returns the log as a styled text.
    pub fn to_styled_text(&self) -> String {
        self.print_to_string(self.level, PrinterFormat::Styled)
    }

    /// Returns the log as text. Whether it is styled or plain text
    /// depends on whether the ANSI colors are supported in the executing terminal or not.
    pub fn to_text(&self) -> String {
        self.print_to_string(self.level, PrinterFormat::Default)
    }

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> Log<'static> {
        Log {
            level: self.level,
            content: self.content.make_owned(),
            cause: self.cause.map(|v| Box::new(v.make_owned())),
        }
    }
}

impl<'a> Printable for Log<'a> {
    fn print<'b>(&'b self, printer: &mut Printer<'b>) {
        // Print content.
        self.content.print(printer);

        // Print cause.
        if let Some(cause) = &self.cause {
            printer.push_plain_text("\n");
            cause.print(printer);
        }
    }
}

impl<'a> Display for Log<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = Printer::new(self.level, PrinterFormat::Plain);
        self.print(&mut printer);
        printer.fmt(f, PrinterFormat::Plain)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::Log;

    #[test]
    fn test_display() {
        println!("{}", Log::error());
    }
}
