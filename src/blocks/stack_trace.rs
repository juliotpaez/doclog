use std::option::Option::Some;

use arcstr::ArcStr;

use crate::utils::text::{color_bold_if, indent_text, remove_jump_lines};
use crate::Log;

/// A trace message of a stack block.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StackTraceBlock {
    location: ArcStr,
    inner_path: Option<ArcStr>,
    line: Option<usize>,
    column: Option<usize>,
    message: Option<ArcStr>,
}

impl StackTraceBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(location: ArcStr) -> StackTraceBlock {
        StackTraceBlock {
            location,
            inner_path: None,
            line: None,
            column: None,
            message: None,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The file path of the trace.
    pub fn get_location(&self) -> &ArcStr {
        &self.location
    }

    /// The relative path of the trace inside a file, e.g. class.method.
    pub fn get_inner_path(&self) -> &Option<ArcStr> {
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
    pub fn get_message(&self) -> &Option<ArcStr> {
        &self.message
    }

    // SETTERS ----------------------------------------------------------------

    pub fn location(mut self, location: ArcStr) -> Self {
        self.location = location;
        self
    }

    pub fn inner_path(mut self, inner_path: ArcStr) -> Self {
        self.inner_path = Some(inner_path);
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

    pub fn message(mut self, message: ArcStr) -> Self {
        self.message = Some(message);
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
            buffer.push(':');
            buffer.push_str(format!("{}", line).as_str());

            if let Some(column) = self.column {
                buffer.push(':');
                buffer.push_str(format!("{}", column).as_str());
            }
        } else if let Some(column) = self.column {
            buffer.push_str(":??:");
            buffer.push_str(format!("{}", column).as_str());
        }

        if let Some(inner_path) = inner_path {
            buffer.push(' ');
            buffer.push_str(&color_bold_if(
                "at".to_string(),
                log.level().color(),
                in_ansi,
            ));
            buffer.push(' ');
            buffer.push_str(inner_path.as_str());
        }

        if let Some(message) = message {
            buffer.push(' ');
            buffer.push_str(&color_bold_if(
                "-".to_string(),
                log.level().color(),
                in_ansi,
            ));
            buffer.push(' ');
            buffer.push_str(message.as_str());
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use yansi::Style;

    use crate::{Log, LogLevel};

    use super::*;

    #[test]
    fn test_plain() {
        // LOCATION
        let mut text = String::new();
        let log = Log::info();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into());
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test"));

        // LOCATION + LINE
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into()).line(15);
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test:15"));

        // LOCATION + COLUMN
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into()).column(24);
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test:??:24"));

        // LOCATION + LINE + COLUMN
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into())
            .line(15)
            .column(24);
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test:15:24"));

        // LOCATION + INNER_PATH
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/t\no/file.test".into())
            .inner_path("path::t\no::class".into());
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/t o/file.test at path::t o::class"));

        // LOCATION + MESSAGE
        let mut text = String::new();
        let stack_trace =
            StackTraceBlock::new("/path/to/file.test".into()).message("Multiline\nmessage".into());
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(text, format!("/path/to/file.test - Multiline\n    message"));

        // LOCATION + ALL
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into())
            .inner_path("path::t\no::class".into())
            .line(15)
            .column(24)
            .message("Multiline\nmessage".into());
        stack_trace.to_text(&log, false, &mut text);

        assert_eq!(
            text,
            format!("/path/to/file.test:15:24 at path::t o::class - Multiline\n    message")
        );
    }

    #[test]
    fn test_ansi() {
        // LOCATION
        let mut text = String::new();
        let log = Log::info();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into());
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(text, format!("/path/to/file.test"));

        // LOCATION + LINE
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into()).line(15);
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(text, format!("/path/to/file.test:15"));

        // LOCATION + COLUMN
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into()).column(24);
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(text, format!("/path/to/file.test:??:24"));

        // LOCATION + LINE + COLUMN
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into())
            .line(15)
            .column(24);
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(text, format!("/path/to/file.test:15:24"));

        // LOCATION + INNER_PATH
        let mut text = String::new();
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into())
            .inner_path("path::t\no::class".into());
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(
            text,
            format!(
                "/path/to/file.test {} path::t o::class",
                Style::new(LogLevel::info().color()).bold().paint("at"),
            )
        );

        // LOCATION + MESSAGE
        let mut text = String::new();
        let stack_trace =
            StackTraceBlock::new("/path/to/file.test".into()).message("Multiline\nmessage".into());
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
        let stack_trace = StackTraceBlock::new("/path/to/file.test".into())
            .inner_path("path::t\no::class".into())
            .line(15)
            .column(24)
            .message("Multiline\nmessage".into());
        stack_trace.to_text(&log, true, &mut text);

        assert_eq!(
            text,
            format!(
                "/path/to/file.test:15:24 {} path::t o::class {} Multiline\n    message",
                Style::new(LogLevel::info().color()).bold().paint("at"),
                Style::new(LogLevel::info().color()).bold().paint("-")
            )
        );
    }
}
