use crate::blocks::TextBlock;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::LogLevel;
use std::borrow::Cow;
use std::fmt::Display;
use yansi::Style;

/// A trace message of a stack block. It can include a file location, a path inside the code
/// and a message.
///
/// When printed, location and path will get all newline characters `\n`
/// replaced by whitespaces to only occupy one line.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct StackTraceBlock<'a> {
    file_location: TextBlock<'a>,
    code_path: TextBlock<'a>,
    message: TextBlock<'a>,
}

impl<'a> StackTraceBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> Self {
        Self::default()
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the file location.
    pub fn get_file_location(&self) -> &TextBlock<'a> {
        &self.file_location
    }

    /// Returns a mutable reference to the file location.
    pub fn get_file_location_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.file_location
    }

    /// Returns the code path.
    pub fn get_code_path(&self) -> &TextBlock<'a> {
        &self.code_path
    }

    /// Returns a mutable reference to the code path.
    pub fn get_inner_code_path_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.code_path
    }

    /// Returns the message.
    pub fn get_message(&self) -> &TextBlock<'a> {
        &self.message
    }

    /// Returns a mutable reference to the message.
    pub fn get_message_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.message
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the file location.
    pub fn file_location(mut self, file_location: impl Into<TextBlock<'a>>) -> Self {
        self.file_location = file_location.into();
        self
    }

    /// Sets the inner code path.
    pub fn code_path(mut self, code_path: impl Into<TextBlock<'a>>) -> Self {
        self.code_path = code_path.into();
        self
    }

    /// Sets the message.
    pub fn message(mut self, message: impl Into<TextBlock<'a>>) -> Self {
        self.message = message.into();
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> StackTraceBlock<'static> {
        StackTraceBlock {
            file_location: self.file_location.make_owned(),
            code_path: self.code_path.make_owned(),
            message: self.message.make_owned(),
        }
    }
}

impl<'a> Printable<'a> for StackTraceBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        // Print file location.
        if !self.file_location.is_empty() {
            self.file_location.single_lined().print(printer);
        } else {
            printer.push_plain_text("<unknown location>");
        }

        // Print code path.
        if !self.code_path.is_empty() {
            printer.push_styled_text(
                Cow::Borrowed("("),
                Style::new().bold().fg(printer.level.color()),
            );
            self.code_path.single_lined().print(printer);
            printer.push_styled_text(
                Cow::Borrowed(")"),
                Style::new().bold().fg(printer.level.color()),
            );
        }

        // Print message.
        if !self.message.is_empty() {
            printer.push_styled_text(" - ", Style::new().bold().fg(printer.level.color()));
            self.message.print(printer);
        }
    }
}

impl<'a> Display for StackTraceBlock<'a> {
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
    use crate::LogLevel;

    #[test]
    fn test_plain() {
        // Empty
        let log = StackTraceBlock::new();
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "<unknown location>");

        // Location
        let log =
            StackTraceBlock::new().file_location(TextBlock::new_plain("/path/to/file.rs:15:24"));
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "/path/to/file.rs:15:24");

        // Inner path
        let log = StackTraceBlock::new().code_path(TextBlock::new_plain("crate::mod::impl"));
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "<unknown location>(crate::mod::impl)");

        // Message
        let log = StackTraceBlock::new().message(TextBlock::new_plain("this is a message"));
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "<unknown location> - this is a message");

        // All
        let log = StackTraceBlock::new()
            .file_location(TextBlock::new_plain("/path/to/\n/file.rs:15:24"))
            .code_path(TextBlock::new_plain("crate::mod::\n::impl"))
            .message(TextBlock::new_plain("this is a\nmessage"));
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(
            text,
            "/path/to/ /file.rs:15:24(crate::mod:: ::impl) - this is a\nmessage"
        );
    }

    #[test]
    fn test_styled() {
        // Empty
        let log = StackTraceBlock::new();
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "<unknown location>");

        // Location
        let log =
            StackTraceBlock::new().file_location(TextBlock::new_plain("/path/to/file.rs:15:24"));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "/path/to/file.rs:15:24");

        // Inner path
        let log = StackTraceBlock::new().code_path(TextBlock::new_plain("crate::mod::impl"));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(
            text,
            "<unknown location>\u{1b}[1;31m(\u{1b}[0mcrate::mod::impl\u{1b}[1;31m)\u{1b}[0m"
        );

        // Message
        let log = StackTraceBlock::new().message(TextBlock::new_plain("this is a message"));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(
            text,
            "<unknown location>\u{1b}[1;31m - \u{1b}[0mthis is a message"
        );

        // All
        let log = StackTraceBlock::new()
            .file_location(TextBlock::new_plain("/path/to/\n/file.rs:15:24"))
            .code_path(TextBlock::new_plain("crate::mod::\n::impl"))
            .message(TextBlock::new_plain("this is a\nmessage"));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(
            text,
            "/path/to/ /file.rs:15:24\u{1b}[1;31m(\u{1b}[0mcrate::mod:: ::impl\u{1b}[1;31m) - \u{1b}[0mthis is a\nmessage"
        );
    }
}
