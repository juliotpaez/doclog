use std::sync::Arc;

use yansi::Style;

use crate::blocks::StackTraceBlock;
use crate::constants::{
    BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, TOP_RIGHT_CORNER, VERTICAL_BAR, VERTICAL_RIGHT_BAR,
};
use crate::utils::text::indent_text;
use crate::Log;

/// A error stack block.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StackBlock {
    message: Arc<String>,
    traces: Vec<StackTraceBlock>,
    cause: Option<Box<StackBlock>>,
    show_stack_numbers: bool,
}

impl StackBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(message: Arc<String>) -> StackBlock {
        StackBlock {
            message,
            traces: vec![],
            cause: None,
            show_stack_numbers: true,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The message of the stack
    pub fn get_message(&self) -> &Arc<String> {
        &self.message
    }

    /// The traces of the stack.
    pub fn get_traces(&self) -> &Vec<StackTraceBlock> {
        &self.traces
    }

    /// The cause of the current error stack.
    pub fn get_cause(&self) -> &Option<Box<StackBlock>> {
        &self.cause
    }

    /// Whether to print the stack numbers in every trace or not.
    pub fn get_show_stack_numbers(&self) -> bool {
        self.show_stack_numbers
    }

    // SETTERS ----------------------------------------------------------------

    pub fn message(mut self, message: Arc<String>) -> Self {
        self.message = message;
        self
    }

    pub fn message_str(mut self, message: &str) -> Self {
        self.message = Arc::new(message.to_string());
        self
    }

    pub fn show_stack_numbers(mut self, show_stack_numbers: bool) -> Self {
        self.show_stack_numbers = show_stack_numbers;
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a trace to the block.
    pub fn trace<F>(mut self, location: Arc<String>, builder: F) -> Self
    where
        F: FnOnce(StackTraceBlock) -> StackTraceBlock,
    {
        let trace = StackTraceBlock::new(location);
        let trace = builder(trace);
        self.traces.push(trace);
        self
    }

    /// Adds a trace to the block.
    pub fn trace_str<F>(self, location: &str, builder: F) -> Self
    where
        F: FnOnce(StackTraceBlock) -> StackTraceBlock,
    {
        self.trace(Arc::new(location.to_string()), builder)
    }

    /// Sets a cause to the block.
    pub fn cause<F>(mut self, message: Arc<String>, builder: F) -> Self
    where
        F: FnOnce(StackBlock) -> StackBlock,
    {
        let stack = StackBlock::new(message);
        let stack = builder(stack);
        self.cause = Some(Box::new(stack));
        self
    }

    /// Sets a cause to the block.
    pub fn cause_str<F>(self, message: &str, builder: F) -> Self
    where
        F: FnOnce(StackBlock) -> StackBlock,
    {
        self.cause(Arc::new(message.to_string()), builder)
    }

    /// Clears the cause of the block.
    pub fn clear_cause(mut self) -> Self {
        self.cause = None;
        self
    }

    /// Count traces of the stack an its cause recursively.
    fn count_traces(&self) -> usize {
        self.traces.len() + self.cause.as_ref().map_or(0, |v| v.count_traces())
    }

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let max_trace_digits = format!("{}", self.count_traces()).len();
        self.to_text_with_options(log, in_ansi, buffer, 0, max_trace_digits, false)
    }

    fn to_text_with_options(
        &self,
        log: &Log,
        in_ansi: bool,
        buffer: &mut String,
        initial_trace_number: usize,
        max_trace_digits: usize,
        is_cause: bool,
    ) {
        let mut next_trace_number = 0;

        if in_ansi {
            let message = if is_cause {
                indent_text(
                    self.get_message(),
                    format!(
                        "{}             ",
                        Style::new(log.level().color()).bold().paint(VERTICAL_BAR)
                    )
                    .as_str(),
                    false,
                )
            } else {
                indent_text(
                    self.get_message(),
                    format!(
                        "{}  ",
                        Style::new(log.level().color()).bold().paint(VERTICAL_BAR)
                    )
                    .as_str(),
                    false,
                )
            };

            // CAUSE
            if is_cause {
                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint(VERTICAL_BAR)
                        .to_string()
                        .as_str(),
                );
                buffer.push('\n');

                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint(format!(
                            "{}{} Caused by:",
                            VERTICAL_RIGHT_BAR, HORIZONTAL_BAR
                        ))
                        .to_string()
                        .as_str(),
                );
            } else {
                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint(format!("{}{}", BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR))
                        .to_string()
                        .as_str(),
                );
            }
            buffer.push_str(" ");
            buffer.push_str(message.as_str());

            // TRACES
            let trace_prefix = format!(
                "{}  ",
                Style::new(log.level().color()).bold().paint(VERTICAL_BAR)
            );
            let full_trace_prefix = if self.show_stack_numbers {
                format!("{}{}", trace_prefix, " ".repeat(max_trace_digits + 3))
            } else {
                trace_prefix.clone()
            };

            let mut trace_buffer = String::new();
            for trace in self.traces.iter() {
                buffer.push('\n');
                buffer.push_str(trace_prefix.as_str());

                if self.show_stack_numbers {
                    let number = self.traces.len() - next_trace_number + initial_trace_number;
                    next_trace_number += 1;

                    buffer.push_str(
                        Style::new(log.level().color())
                            .bold()
                            .paint(format!("[{:>width$}]", number, width = max_trace_digits))
                            .to_string()
                            .as_str(),
                    );
                    buffer.push_str(" ");
                }

                trace_buffer.clear();
                trace.to_text(log, in_ansi, &mut trace_buffer);
                buffer.push_str(
                    indent_text(&trace_buffer, full_trace_prefix.as_str(), false).as_str(),
                );
            }

            // CAUSE
            if let Some(cause) = &self.cause {
                buffer.push('\n');
                cause.to_text_with_options(
                    log,
                    in_ansi,
                    buffer,
                    next_trace_number + initial_trace_number,
                    max_trace_digits,
                    true,
                );
            }

            // END
            if !is_cause {
                buffer.push('\n');
                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint(format!("{}{}", TOP_RIGHT_CORNER, HORIZONTAL_BAR))
                        .to_string()
                        .as_str(),
                );
            }
        } else {
            let message = if is_cause {
                indent_text(
                    self.get_message(),
                    format!("{}             ", VERTICAL_BAR).as_str(),
                    false,
                )
            } else {
                indent_text(
                    self.get_message(),
                    format!("{}  ", VERTICAL_BAR).as_str(),
                    false,
                )
            };

            // CAUSE
            if is_cause {
                buffer.push_str(VERTICAL_BAR);
                buffer.push('\n');

                buffer.push_str(VERTICAL_RIGHT_BAR);
                buffer.push_str(HORIZONTAL_BAR);
                buffer.push_str(" Caused by:");
            } else {
                buffer.push_str(BOTTOM_RIGHT_CORNER);
                buffer.push_str(HORIZONTAL_BAR);
            }
            buffer.push_str(" ");
            buffer.push_str(message.as_str());

            // TRACES
            let trace_prefix = format!("{}  ", VERTICAL_BAR);
            let full_trace_prefix = if self.show_stack_numbers {
                format!("{}{}", trace_prefix, " ".repeat(max_trace_digits + 3))
            } else {
                trace_prefix.clone()
            };

            let mut trace_buffer = String::new();
            for trace in self.traces.iter() {
                buffer.push('\n');
                buffer.push_str(trace_prefix.as_str());

                if self.show_stack_numbers {
                    let number = self.traces.len() - next_trace_number + initial_trace_number;
                    next_trace_number += 1;

                    buffer.push_str("[");
                    buffer
                        .push_str(format!("{:>width$}", number, width = max_trace_digits).as_str());
                    buffer.push_str("] ");
                }

                trace_buffer.clear();
                trace.to_text(log, in_ansi, &mut trace_buffer);
                buffer.push_str(
                    indent_text(&trace_buffer, full_trace_prefix.as_str(), false).as_str(),
                );
            }

            // CAUSE
            if let Some(cause) = &self.cause {
                buffer.push('\n');
                cause.to_text_with_options(
                    log,
                    in_ansi,
                    buffer,
                    next_trace_number + initial_trace_number,
                    max_trace_digits,
                    true,
                );
            }

            // END
            if !is_cause {
                buffer.push('\n');
                buffer.push_str(TOP_RIGHT_CORNER);
                buffer.push_str(HORIZONTAL_BAR);
            }
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
        // MESSAGE
        let log = Log::info().stack_str("This is\na message", |stack| {
            stack.show_stack_numbers(false)
        });
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!(
                "{}{} This is\n{}  a message\n{}{}",
                BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, VERTICAL_BAR, TOP_RIGHT_CORNER, HORIZONTAL_BAR
            )
        );

        // MESSAGE + SHOW_NUMBERS
        let log =
            Log::info().stack_str("This is\na message", |stack| stack.show_stack_numbers(true));
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!(
                "{}{} This is\n{}  a message\n{}{}",
                BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, VERTICAL_BAR, TOP_RIGHT_CORNER, HORIZONTAL_BAR
            )
        );

        // MESSAGE + TRACES
        let log = Log::info().stack_str("Message", |stack| {
            stack
                .show_stack_numbers(false)
                .trace_str("/path/to/file.test", |trace| trace)
                .trace_str("/path/to/file2.test", |trace| trace)
        });
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!(
                "{}{} Message\n{}  /path/to/file.test\n{}  /path/to/file2.test\n{}{}",
                BOTTOM_RIGHT_CORNER,
                HORIZONTAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                TOP_RIGHT_CORNER,
                HORIZONTAL_BAR
            )
        );

        // MESSAGE + TRACES + SHOW_NUMBERS
        let log = Log::info().stack_str("Message", |stack| {
            stack
                .show_stack_numbers(true)
                .trace_str("file10", |trace| trace)
                .trace_str("file09", |trace| trace)
                .trace_str("file08", |trace| trace)
                .trace_str("file07", |trace| trace)
                .trace_str("file06", |trace| trace)
                .trace_str("file05", |trace| trace)
                .trace_str("file04", |trace| trace)
                .trace_str("file03", |trace| trace)
                .trace_str("file02", |trace| trace)
                .trace_str("file01", |trace| trace)
        });
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!(
                "{}{} Message\n{}  [10] file10\n{}  [ 9] file09\n{}  [ 8] file08\n{}  [ 7] file07\n{}  [ 6] file06\n{}  [ 5] file05\n{}  [ 4] file04\n{}  [ 3] file03\n{}  [ 2] file02\n{}  [ 1] file01\n{}{}",
                BOTTOM_RIGHT_CORNER,
                HORIZONTAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                TOP_RIGHT_CORNER,
                HORIZONTAL_BAR
            )
        );
    }

    #[test]
    fn test_plain_with_cause() {
        // MESSAGE
        let log = Log::info().stack_str("", |stack| {
            stack.cause_str("This is\na message", |stack| stack)
        });
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!(
                "{}{} \n{}\n{}{} Caused by: This is\n{}             a message\n{}{}",
                BOTTOM_RIGHT_CORNER,
                HORIZONTAL_BAR,
                VERTICAL_BAR,
                VERTICAL_RIGHT_BAR,
                HORIZONTAL_BAR,
                VERTICAL_BAR,
                TOP_RIGHT_CORNER,
                HORIZONTAL_BAR
            )
        );

        // MESSAGE + TRACES
        let log = Log::info().stack_str("", |stack| {
            stack
                .show_stack_numbers(true)
                .trace_str("File2", |trace| trace)
                .trace_str("File1", |trace| trace)
                .cause_str("Cause 1", |stack| {
                    stack
                        .trace_str("File3", |trace| trace)
                        .cause_str("Cause 2", |stack| stack.trace_str("File4", |trace| trace))
                })
        });
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!(
                "{}{} \n{}  [2] File2\n{}  [1] File1\n{}\n{}{} Caused by: Cause 1\n{}  [3] File3\n{}\n{}{} Caused by: Cause 2\n{}  [4] File4\n{}{}",
                BOTTOM_RIGHT_CORNER,
                HORIZONTAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_RIGHT_BAR,
                HORIZONTAL_BAR,
                VERTICAL_BAR,
                VERTICAL_BAR,
                VERTICAL_RIGHT_BAR,
                HORIZONTAL_BAR,
                VERTICAL_BAR,
                TOP_RIGHT_CORNER,
                HORIZONTAL_BAR
            )
        );
    }

    #[test]
    fn test_ansi() {
        // MESSAGE
        let log = Log::info().stack_str("This is\na message", |stack| {
            stack.show_stack_numbers(false)
        });
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} This is\n{}  a message\n{}",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR)),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", TOP_RIGHT_CORNER, HORIZONTAL_BAR)),
            )
        );

        // MESSAGE + SHOW_NUMBERS
        let log =
            Log::info().stack_str("This is\na message", |stack| stack.show_stack_numbers(true));
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} This is\n{}  a message\n{}",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR)),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", TOP_RIGHT_CORNER, HORIZONTAL_BAR)),
            )
        );

        // MESSAGE + TRACES
        let log = Log::info().stack_str("Message", |stack| {
            stack
                .show_stack_numbers(false)
                .trace_str("/path/to/file.test", |trace| trace)
                .trace_str("/path/to/file2.test", |trace| trace)
        });
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} Message\n{}  /path/to/file.test\n{}  /path/to/file2.test\n{}",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR)),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", TOP_RIGHT_CORNER, HORIZONTAL_BAR)),
            )
        );

        // MESSAGE + TRACES + SHOW_NUMBERS
        let log = Log::info().stack_str("Message", |stack| {
            stack
                .show_stack_numbers(true)
                .trace_str("file10", |trace| trace)
                .trace_str("file09", |trace| trace)
                .trace_str("file08", |trace| trace)
                .trace_str("file07", |trace| trace)
                .trace_str("file06", |trace| trace)
                .trace_str("file05", |trace| trace)
                .trace_str("file04", |trace| trace)
                .trace_str("file03", |trace| trace)
                .trace_str("file02", |trace| trace)
                .trace_str("file01", |trace| trace)
        });
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} Message\n{}  {} file10\n{}  {} file09\n{}  {} file08\n{}  {} file07\n{}  {} file06\n{}  {} file05\n{}  {} file04\n{}  {} file03\n{}  {} file02\n{}  {} file01\n{}",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR)),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[10]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 9]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 8]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 7]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 6]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 5]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 4]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 3]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 2]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[ 1]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", TOP_RIGHT_CORNER, HORIZONTAL_BAR)),
            )
        );
    }

    #[test]
    fn test_ansi_with_cause() {
        // MESSAGE
        let log = Log::info().stack_str("", |stack| {
            stack.cause_str("This is\na message", |stack| stack)
        });
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} \n{}\n{} This is\n{}             a message\n{}",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR)),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color()).bold().paint(format!(
                    "{}{} Caused by:",
                    VERTICAL_RIGHT_BAR, HORIZONTAL_BAR
                )),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", TOP_RIGHT_CORNER, HORIZONTAL_BAR)),
            )
        );

        // MESSAGE + TRACES
        let log = Log::info().stack_str("", |stack| {
            stack
                .show_stack_numbers(true)
                .trace_str("File2", |trace| trace)
                .trace_str("File1", |trace| trace)
                .cause_str("Cause 1", |stack| {
                    stack
                        .trace_str("File3", |trace| trace)
                        .cause_str("Cause 2", |stack| stack.trace_str("File4", |trace| trace))
                })
        });
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} \n{}  {} File2\n{}  {} File1\n{}\n{} Cause 1\n{}  {} File3\n{}\n{} Cause 2\n{}  {} File4\n{}",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR)),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[2]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[1]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color()).bold().paint(format!(
                    "{}{} Caused by:",
                    VERTICAL_RIGHT_BAR, HORIZONTAL_BAR
                )),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[3]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color()).bold().paint(format!(
                    "{}{} Caused by:",
                    VERTICAL_RIGHT_BAR, HORIZONTAL_BAR
                )),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(VERTICAL_BAR),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint("[4]"),
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(format!("{}{}", TOP_RIGHT_CORNER, HORIZONTAL_BAR)),
            )
        );
    }
}
