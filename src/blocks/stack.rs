use crate::blocks::{StackTraceBlock, TextBlock};
use crate::constants::{
    BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, RIGHT_ARROW, TOP_RIGHT_CORNER, VERTICAL_BAR,
    VERTICAL_RIGHT_BAR,
};
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::utils::whitespaces::build_space_string;
use crate::LogLevel;
use const_format::concatcp;
use std::borrow::Cow;
use std::fmt::Display;
use std::mem;
use yansi::Style;

/// An error stack block.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct StackBlock<'a> {
    pub message: TextBlock<'a>,
    pub traces: Vec<StackTraceBlock<'a>>,
    pub cause: Option<Box<StackBlock<'a>>>,
    pub show_stack_numbers: bool,

    /// Whether to print the stack in the wrapped-by format.
    pub wrapped_by_format: bool,
}

impl<'a> StackBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new empty [StackBlock].
    #[inline(always)]
    pub fn new() -> Self {
        StackBlock::default()
    }

    // BUILDERS ---------------------------------------------------------------

    /// Sets the message.
    #[inline(always)]
    pub fn message(mut self, message: impl Into<TextBlock<'a>>) -> Self {
        self.message = message.into();
        self
    }

    /// Add a stack trace.
    #[inline(always)]
    pub fn add_stack_trace(mut self, stack_trace: StackTraceBlock<'a>) -> Self {
        self.traces.push(stack_trace);
        self
    }

    /// Sets the cause.
    #[inline(always)]
    pub fn cause(mut self, cause: StackBlock<'a>) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Sets whether to show stack numbers.
    #[inline(always)]
    pub fn show_stack_numbers(mut self, show_stack_numbers: bool) -> Self {
        self.show_stack_numbers = show_stack_numbers;
        self
    }

    /// Sets whether to print the stack in the wrapped-by format.
    #[inline(always)]
    pub fn wrapped_by_format(mut self, wrapped_by_format: bool) -> Self {
        self.wrapped_by_format = wrapped_by_format;
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Count traces of the stack and its cause recursively.
    fn count_traces(&self) -> usize {
        self.traces.len() + self.cause.as_ref().map_or(0, |v| v.count_traces())
    }

    /// Prints the stack block with the given options following the caused by format, i.e.
    /// the top error is printed first and then what caused it.
    fn print_as_caused_by(
        &self,
        printer: &mut Printer<'a>,
        initial_trace_number: usize,
        max_trace_digits: usize,
        is_cause: bool,
    ) {
        // Message
        if is_cause {
            printer.push_styled_text(
                concatcp!(
                    '\n',
                    VERTICAL_RIGHT_BAR,
                    HORIZONTAL_BAR,
                    HORIZONTAL_BAR,
                    HORIZONTAL_BAR,
                    RIGHT_ARROW,
                    " Caused by: "
                ),
                Style::new().bold().fg(printer.level.color()),
            );
        } else if self.message.is_empty() {
            printer.push_styled_text(
                concatcp!(BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, ' '),
                Style::new().bold().fg(printer.level.color()),
            );
        } else {
            printer.push_styled_text(
                concatcp!(BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, RIGHT_ARROW, ' '),
                Style::new().bold().fg(printer.level.color()),
            );
        }

        {
            let mut message_printer = printer.derive();
            self.message.print(&mut message_printer);

            let prefix = TextBlock::new().add_styled_text(
                if is_cause {
                    concatcp!(VERTICAL_BAR, "     ")
                } else {
                    concatcp!(VERTICAL_BAR, "   ")
                },
                Style::new().bold().fg(printer.level.color()),
            );

            message_printer.indent(&prefix.sections, false);
            printer.append(message_printer);
        }

        // Traces
        let trace_prefix = TextBlock::new().add_styled_text(
            concatcp!(VERTICAL_BAR, "  "),
            Style::new().bold().fg(printer.level.color()),
        );
        let full_trace_prefix = trace_prefix.clone().add_styled_text(
            build_space_string(max_trace_digits + 2),
            Style::new().bold().fg(printer.level.color()),
        );

        let mut trace_printer = printer.derive();
        let mut next_trace_number = 0;
        for trace in self.traces.iter() {
            printer.push_plain_text(Cow::Borrowed("\n"));
            trace_prefix.print(printer);

            let number = self.traces.len() - next_trace_number + initial_trace_number;
            next_trace_number += 1;

            if self.show_stack_numbers {
                printer.push_styled_text(
                    format!("[{:>width$}] ", number, width = max_trace_digits),
                    Style::new().bold().fg(printer.level.color()),
                );
            } else {
                printer.push_styled_text(" at ", Style::new().bold().fg(printer.level.color()));
            }

            trace.print(&mut trace_printer);
            trace_printer.indent(&full_trace_prefix.sections, false);
            printer.append(mem::replace(&mut trace_printer, printer.derive()));
        }

        // Cause
        if let Some(cause) = &self.cause {
            cause.print_as_caused_by(
                printer,
                next_trace_number + initial_trace_number,
                max_trace_digits,
                true,
            );
        }

        // Final line
        if !is_cause {
            printer.push_styled_text(
                concatcp!('\n', TOP_RIGHT_CORNER, HORIZONTAL_BAR),
                Style::new().bold().fg(printer.level.color()),
            );
        }
    }

    /// Prints the stack block with the given options following the wrapped by format, i.e.
    /// the innermost error is printed first and then what wrapped it.
    fn print_as_wrapped_by(
        &self,
        printer: &mut Printer<'a>,
        initial_trace_number: usize,
        max_trace_digits: usize,
        is_root: bool,
    ) {
        let is_cause = match &self.cause {
            Some(cause) => {
                cause.print_as_wrapped_by(
                    printer,
                    initial_trace_number + self.traces.len(),
                    max_trace_digits,
                    false,
                );
                true
            }
            None => false,
        };

        // Message.
        if is_cause {
            printer.push_styled_text(
                concatcp!(
                    '\n',
                    VERTICAL_RIGHT_BAR,
                    HORIZONTAL_BAR,
                    HORIZONTAL_BAR,
                    HORIZONTAL_BAR,
                    RIGHT_ARROW,
                    " Wrapped by: "
                ),
                Style::new().bold().fg(printer.level.color()),
            );
        } else if self.message.is_empty() {
            printer.push_styled_text(
                concatcp!(BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, ' '),
                Style::new().bold().fg(printer.level.color()),
            );
        } else {
            printer.push_styled_text(
                concatcp!(BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, RIGHT_ARROW, ' '),
                Style::new().bold().fg(printer.level.color()),
            );
        }

        {
            let mut message_printer = printer.derive();
            self.message.print(&mut message_printer);

            let prefix = TextBlock::new().add_styled_text(
                if is_cause {
                    concatcp!(VERTICAL_BAR, "     ")
                } else {
                    concatcp!(VERTICAL_BAR, "   ")
                },
                Style::new().bold().fg(printer.level.color()),
            );

            message_printer.indent(&prefix.sections, false);
            printer.append(message_printer);
        }

        // Traces
        let trace_prefix = TextBlock::new().add_styled_text(
            concatcp!(VERTICAL_BAR, "  "),
            Style::new().bold().fg(printer.level.color()),
        );
        let full_trace_prefix = trace_prefix.clone().add_styled_text(
            build_space_string(max_trace_digits + 2),
            Style::new().bold().fg(printer.level.color()),
        );

        let mut trace_printer = printer.derive();
        for (next_trace_number, trace) in self.traces.iter().enumerate() {
            printer.push_plain_text(Cow::Borrowed("\n"));
            trace_prefix.print(printer);

            if self.show_stack_numbers {
                let number = self.traces.len() - next_trace_number + initial_trace_number;
                printer.push_styled_text(
                    format!("[{:>width$}] ", number, width = max_trace_digits),
                    Style::new().bold().fg(printer.level.color()),
                );
            } else {
                printer.push_styled_text(" at ", Style::new().bold().fg(printer.level.color()));
            }

            trace.print(&mut trace_printer);
            trace_printer.indent(&full_trace_prefix.sections, false);
            printer.append(mem::replace(&mut trace_printer, printer.derive()));
        }

        // Final line
        if is_root {
            printer.push_styled_text(
                concatcp!('\n', TOP_RIGHT_CORNER, HORIZONTAL_BAR),
                Style::new().bold().fg(printer.level.color()),
            );
        }
    }

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> StackBlock<'static> {
        StackBlock {
            message: self.message.make_owned(),
            traces: self.traces.into_iter().map(|v| v.make_owned()).collect(),
            cause: self.cause.map(|v| Box::new(v.make_owned())),
            show_stack_numbers: self.show_stack_numbers,
            wrapped_by_format: self.wrapped_by_format,
        }
    }
}

impl<'a> Printable<'a> for StackBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        let max_trace_digits = format!("{}", self.count_traces()).len();

        if self.wrapped_by_format {
            self.print_as_wrapped_by(printer, 0, max_trace_digits, true)
        } else {
            self.print_as_caused_by(printer, 0, max_trace_digits, false)
        }
    }
}

impl<'a> Display for StackBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = Printer::new(LogLevel::trace(), PrinterFormat::Plain);
        self.print(&mut printer);
        printer.fmt(f, PrinterFormat::Plain)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain() {
        // Empty
        let log = StackBlock::new();
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─ \n╰─");

        // Message
        let log = StackBlock::new().message(TextBlock::new_plain("This is\na message"));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─▶ This is\n│   a message\n╰─");

        // Traces without numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            );
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─ \n│   at /a/b/c(crate::x) - This is a \n│      message\n│   at /a/b/c/2(crate::x::2) - This is a \n│      message2\n╰─");

        // Traces with numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true);
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─ \n│  [2] /a/b/c(crate::x) - This is a \n│      message\n│  [1] /a/b/c/2(crate::x::2) - This is a \n│      message2\n╰─");

        // All
        let log = StackBlock::new()
            .message(TextBlock::new_plain("This is\na message"))
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true);
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─▶ This is\n│   a message\n│  [2] /a/b/c(crate::x) - This is a \n│      message\n│  [1] /a/b/c/2(crate::x::2) - This is a \n│      message2\n╰─");
    }

    #[test]
    fn test_styled() {
        // Empty
        let log = StackBlock::new();
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─ \n╰─\u{1b}[0m");

        // Message
        let log = StackBlock::new().message(TextBlock::new_plain("This is\na message"));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mThis is\n\u{1b}[1;31m│   \u{1b}[0ma message\n\u{1b}[1;31m╰─\u{1b}[0m");

        // Traces without numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            );
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─ \n│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");

        // Traces with numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true);
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─ \n│  [2] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [1] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");

        // All
        let log = StackBlock::new()
            .message(TextBlock::new_plain("This is\na message"))
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true);
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mThis is\n\u{1b}[1;31m│   \u{1b}[0ma message\n\u{1b}[1;31m│  [2] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [1] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");
    }

    #[test]
    fn test_plain_as_caused_by_format() {
        let cause = StackBlock::new()
            .message(TextBlock::new_plain("Cause\nnumber2"))
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true);
        let cause = cause
            .clone()
            .message("Cause\nnumber1")
            .show_stack_numbers(false)
            .cause(cause.clone());

        // Empty
        let log = StackBlock::new().cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─ \n├───▶ Caused by: Cause\n│     number1\n│   at /a/b/c(crate::x) - This is a \n│      message\n│   at /a/b/c/2(crate::x::2) - This is a \n│      message2\n├───▶ Caused by: Cause\n│     number2\n│  [4] /a/b/c(crate::x) - This is a \n│      message\n│  [3] /a/b/c/2(crate::x::2) - This is a \n│      message2\n╰─");

        // Message
        let log = StackBlock::new()
            .message(TextBlock::new_plain("This is\na message"))
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─▶ This is\n│   a message\n├───▶ Caused by: Cause\n│     number1\n│   at /a/b/c(crate::x) - This is a \n│      message\n│   at /a/b/c/2(crate::x::2) - This is a \n│      message2\n├───▶ Caused by: Cause\n│     number2\n│  [4] /a/b/c(crate::x) - This is a \n│      message\n│  [3] /a/b/c/2(crate::x::2) - This is a \n│      message2\n╰─");

        // Traces without numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─ \n│   at /a/b/c(crate::x) - This is a \n│      message\n│   at /a/b/c/2(crate::x::2) - This is a \n│      message2\n├───▶ Caused by: Cause\n│     number1\n│   at /a/b/c(crate::x) - This is a \n│      message\n│   at /a/b/c/2(crate::x::2) - This is a \n│      message2\n├───▶ Caused by: Cause\n│     number2\n│  [6] /a/b/c(crate::x) - This is a \n│      message\n│  [5] /a/b/c/2(crate::x::2) - This is a \n│      message2\n╰─");

        // Traces with numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─ \n│  [2] /a/b/c(crate::x) - This is a \n│      message\n│  [1] /a/b/c/2(crate::x::2) - This is a \n│      message2\n├───▶ Caused by: Cause\n│     number1\n│   at /a/b/c(crate::x) - This is a \n│      message\n│   at /a/b/c/2(crate::x::2) - This is a \n│      message2\n├───▶ Caused by: Cause\n│     number2\n│  [6] /a/b/c(crate::x) - This is a \n│      message\n│  [5] /a/b/c/2(crate::x::2) - This is a \n│      message2\n╰─");

        // All
        let log = StackBlock::new()
            .message(TextBlock::new_plain("This is\na message"))
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "╭─▶ This is\n│   a message\n│  [2] /a/b/c(crate::x) - This is a \n│      message\n│  [1] /a/b/c/2(crate::x::2) - This is a \n│      message2\n├───▶ Caused by: Cause\n│     number1\n│   at /a/b/c(crate::x) - This is a \n│      message\n│   at /a/b/c/2(crate::x::2) - This is a \n│      message2\n├───▶ Caused by: Cause\n│     number2\n│  [6] /a/b/c(crate::x) - This is a \n│      message\n│  [5] /a/b/c/2(crate::x::2) - This is a \n│      message2\n╰─");
    }

    #[test]
    fn test_styled_as_caused_by_format() {
        let cause = StackBlock::new()
            .message(TextBlock::new_plain("Cause\nnumber2"))
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true);
        let cause = cause
            .clone()
            .message("Cause\nnumber1")
            .show_stack_numbers(false)
            .cause(cause.clone());

        // Empty
        let log = StackBlock::new().cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─ \n├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber2\n\u{1b}[1;31m│  [4] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [3] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");

        // Message
        let log = StackBlock::new()
            .message(TextBlock::new_plain("This is\na message"))
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mThis is\n\u{1b}[1;31m│   \u{1b}[0ma message\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber2\n\u{1b}[1;31m│  [4] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [3] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");

        // Traces without numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─ \n│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber2\n\u{1b}[1;31m│  [6] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [5] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");

        // Traces with numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─ \n│  [2] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [1] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber2\n\u{1b}[1;31m│  [6] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [5] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");

        // All
        let log = StackBlock::new()
            .message(TextBlock::new_plain("This is\na message"))
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mThis is\n\u{1b}[1;31m│   \u{1b}[0ma message\n\u{1b}[1;31m│  [2] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [1] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Caused by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber2\n\u{1b}[1;31m│  [6] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [5] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");
    }

    #[test]
    fn test_styled_as_wrapped_by_format() {
        let cause = StackBlock::new()
            .message(TextBlock::new_plain("Cause\nnumber2"))
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true);
        let cause = cause
            .clone()
            .message("Cause\nnumber1")
            .show_stack_numbers(false)
            .cause(cause.clone());

        // Empty
        let log = StackBlock::new()
            .wrapped_by_format(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mCause\n\u{1b}[1;31m│   \u{1b}[0mnumber2\n\u{1b}[1;31m│  [4] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [3] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \n╰─\u{1b}[0m");

        // Message
        let log = StackBlock::new()
            .message(TextBlock::new_plain("This is\na message"))
            .wrapped_by_format(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mCause\n\u{1b}[1;31m│   \u{1b}[0mnumber2\n\u{1b}[1;31m│  [4] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [3] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \u{1b}[0mThis is\n\u{1b}[1;31m│     \u{1b}[0ma message\n\u{1b}[1;31m╰─\u{1b}[0m");

        // Traces without numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .wrapped_by_format(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mCause\n\u{1b}[1;31m│   \u{1b}[0mnumber2\n\u{1b}[1;31m│  [6] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [5] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \n│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");

        // Traces with numbers
        let log = StackBlock::new()
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true)
            .wrapped_by_format(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mCause\n\u{1b}[1;31m│   \u{1b}[0mnumber2\n\u{1b}[1;31m│  [6] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [5] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \n│  [2] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [1] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");

        // All
        let log = StackBlock::new()
            .message(TextBlock::new_plain("This is\na message"))
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message"))
                    .file_location(TextBlock::new_plain("/a/b/c"))
                    .code_path(TextBlock::new_plain("crate::x")),
            )
            .add_stack_trace(
                StackTraceBlock::new()
                    .message(TextBlock::new_plain("This is a \n message2"))
                    .file_location(TextBlock::new_plain("/a/b/c/2"))
                    .code_path(TextBlock::new_plain("crate::x::2")),
            )
            .show_stack_numbers(true)
            .wrapped_by_format(true)
            .cause(cause.clone());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m╭─▶ \u{1b}[0mCause\n\u{1b}[1;31m│   \u{1b}[0mnumber2\n\u{1b}[1;31m│  [6] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [5] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \u{1b}[0mCause\n\u{1b}[1;31m│     \u{1b}[0mnumber1\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│   at \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m├───▶ Wrapped by: \u{1b}[0mThis is\n\u{1b}[1;31m│     \u{1b}[0ma message\n\u{1b}[1;31m│  [2] \u{1b}[0m/a/b/c\u{1b}[1;31m(\u{1b}[0mcrate::x\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message\n\u{1b}[1;31m│  [1] \u{1b}[0m/a/b/c/2\u{1b}[1;31m(\u{1b}[0mcrate::x::2\u{1b}[1;31m) - \u{1b}[0mThis is a \n\u{1b}[1;31m│     \u{1b}[0m message2\n\u{1b}[1;31m╰─\u{1b}[0m");
    }
}
