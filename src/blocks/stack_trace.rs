use std::option::Option::Some;
use std::sync::Arc;

use yansi::Style;

use crate::utils::text::{indent_text, remove_jump_lines};
use crate::Log;

/// A trace message of a stack block.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StackTraceBlock {
    location: Arc<String>,
    inner_path: Option<Arc<String>>,
    line: Option<usize>,
    column: Option<usize>,
    message: Option<Arc<String>>,
}

impl StackTraceBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(location: Arc<String>) -> StackTraceBlock {
        StackTraceBlock {
            location,
            inner_path: None,
            line: None,
            column: None,
            message: None,
        }
    }

    pub fn new_str(location: &str) -> StackTraceBlock {
        Self::new(Arc::new(location.to_string()))
    }

    // GETTERS ----------------------------------------------------------------

    /// The file path of the trace.
    pub fn get_location(&self) -> &Arc<String> {
        &self.location
    }

    /// The relative path of the trace inside a file, e.g. class.method.
    pub fn get_inner_path(&self) -> &Option<Arc<String>> {
        &self.inner_path
    }

    /// The line of the file where the stack trace is produced.
    pub fn get_line(&self) -> Option<usize> {
        self.line
    }

    /// The column of the file where the stack trace is produced.
    pub fn get_column(&self) -> Option<usize> {
        self.column
    }

    /// The message to show.
    pub fn get_message(&self) -> &Option<Arc<String>> {
        &self.message
    }

    // SETTERS ----------------------------------------------------------------

    pub fn location(mut self, location: Arc<String>) -> Self {
        self.location = location;
        self
    }
    pub fn location_str(mut self, location: &str) -> Self {
        self.location = Arc::new(location.to_string());
        self
    }

    pub fn inner_path(mut self, inner_path: Arc<String>) -> Self {
        self.inner_path = Some(inner_path);
        self
    }

    pub fn inner_path_str(mut self, inner_path: &str) -> Self {
        self.inner_path = Some(Arc::new(inner_path.to_string()));
        self
    }

    pub fn clear_inner_path(mut self) -> Self {
        self.inner_path = None;
        self
    }

    pub fn line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn clear_line(mut self) -> Self {
        self.line = None;
        self
    }

    pub fn column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn clear_column(mut self) -> Self {
        self.column = None;
        self
    }

    pub fn message(mut self, message: Arc<String>) -> Self {
        self.message = Some(message);
        self
    }

    pub fn message_str(mut self, message: &str) -> Self {
        self.message = Some(Arc::new(message.to_string()));
        self
    }

    pub fn clear_message(mut self) -> Self {
        self.message = None;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let location = remove_jump_lines(&self.location);
        let inner_path = if let Some(inner_path) = &self.inner_path {
            Some(remove_jump_lines(&inner_path))
        } else {
            None
        };

        let message = if let Some(message) = &self.message {
            Some(indent_text(message, "    ", false))
        } else {
            None
        };

        buffer.push_str(location.as_str());

        if let Some(line) = self.line {
            buffer.push_str(":");
            buffer.push_str(format!("{}", line).as_str());

            if let Some(column) = self.column {
                buffer.push_str(":");
                buffer.push_str(format!("{}", column).as_str());
            }
        } else if let Some(column) = self.column {
            buffer.push_str(":??:");
            buffer.push_str(format!("{}", column).as_str());
        }

        if in_ansi {
            if let Some(inner_path) = inner_path {
                buffer.push_str(" ");
                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint("at")
                        .to_string()
                        .as_str(),
                );
                buffer.push_str(" ");
                buffer.push_str(inner_path.as_str());
            }

            if let Some(message) = message {
                buffer.push_str(" ");
                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint("-")
                        .to_string()
                        .as_str(),
                );
                buffer.push_str(" ");
                buffer.push_str(message.as_str());
            }
        } else {
            if let Some(inner_path) = inner_path {
                buffer.push_str(" at ");
                buffer.push_str(inner_path.as_str());
            }

            if let Some(message) = message {
                buffer.push_str(" - ");
                buffer.push_str(message.as_str());
            }
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::{Log, LogLevel};

    use super::*;

    #[test]
    fn test_plain() {
        // LOCATION
        let mut text = String::new();
        let log = Log::info();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test");
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test"));

        // LOCATION + LINE
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test").line(15);
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test:15"));

        // LOCATION + COLUMN
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test").column(24);
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test:??:24"));

        // LOCATION + LINE + COLUMN
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test")
            .line(15)
            .column(24);
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test:15:24"));

        // LOCATION + INNER_PATH
        let mut text = String::new();
        let stack_trace =
            StackTraceBlock::new_str("/path/t\no/file.test").inner_path_str("path::t\no::class");
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test at path::to::class"));

        // LOCATION + MESSAGE
        let mut text = String::new();
        let stack_trace =
            StackTraceBlock::new_str("/path/to/file.test").message_str("Multiline\nmessage");
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test - Multiline\n    message"));

        // LOCATION + ALL
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test")
            .inner_path_str("path::t\no::class")
            .line(15)
            .column(24)
            .message_str("Multiline\nmessage");
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(
            text,
            format!("/path/to/file.test:15:24 at path::to::class - Multiline\n    message")
        );
    }

    #[test]
    fn test_ansi() {
        // LOCATION
        let mut text = String::new();
        let log = Log::info();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test");
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(text, format!("/path/to/file.test"));

        // LOCATION + LINE
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test").line(15);
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(text, format!("/path/to/file.test:15"));

        // LOCATION + COLUMN
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test").column(24);
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(text, format!("/path/to/file.test:??:24"));

        // LOCATION + LINE + COLUMN
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test")
            .line(15)
            .column(24);
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(text, format!("/path/to/file.test:15:24"));

        // LOCATION + INNER_PATH
        let mut text = String::new();
        let stack_trace =
            StackTraceBlock::new_str("/path/to/file.test").inner_path_str("path::t\no::class");
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(
            text,
            format!(
                "/path/to/file.test {} path::to::class",
                Style::new(LogLevel::info().color()).bold().paint("at"),
            )
        );

        // LOCATION + MESSAGE
        let mut text = String::new();
        let stack_trace =
            StackTraceBlock::new_str("/path/to/file.test").message_str("Multiline\nmessage");
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(
            text,
            format!(
                "/path/to/file.test {} Multiline\n    message",
                Style::new(LogLevel::info().color()).bold().paint("-")
            )
        );

        // LOCATION + ALL
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new_str("/path/to/file.test")
            .inner_path_str("path::t\no::class")
            .line(15)
            .column(24)
            .message_str("Multiline\nmessage");
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(
            text,
            format!(
                "/path/to/file.test:15:24 {} path::to::class {} Multiline\n    message",
                Style::new(LogLevel::info().color()).bold().paint("at"),
                Style::new(LogLevel::info().color()).bold().paint("-")
            )
        );
    }
}
