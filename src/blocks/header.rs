use crate::blocks::TextBlock;
use crate::constants::NEW_LINE_RIGH;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::utils::text::remove_jump_lines;
use crate::LogLevel;
use chrono::{SecondsFormat, Utc};
use const_format::concatcp;
use std::borrow::Cow;
use std::fmt::Display;
use yansi::Style;

/// A block that prints a title, showing the type of log and the message.
/// It optionally shows the current date and thread.
///
/// # Examples
/// ```text
/// info in /path/to/file.rs
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct HeaderBlock<'a> {
    code: Option<Cow<'a, str>>,
    location: Option<TextBlock<'a>>,
    show_date: bool,
    show_thread: bool,
}

impl<'a> HeaderBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> Self {
        Self::default()
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the code.
    pub fn get_code(&self) -> &Option<Cow<'a, str>> {
        &self.code
    }

    /// Returns a mutable reference to the code.
    pub fn get_code_mut(&mut self) -> &mut Option<Cow<'a, str>> {
        &mut self.code
    }

    /// Returns the location.
    pub fn get_location(&self) -> &Option<TextBlock<'a>> {
        &self.location
    }

    /// Returns a mutable reference to the location.
    pub fn get_location_mut(&mut self) -> &mut Option<TextBlock<'a>> {
        &mut self.location
    }

    /// Returns whether the date should be shown.
    pub fn get_show_date(&self) -> bool {
        self.show_date
    }

    /// Returns whether the thread should be shown.
    pub fn get_show_thread(&self) -> bool {
        self.show_thread
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the code.
    pub fn code(mut self, code: impl Into<Cow<'a, str>>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Removes the code.
    pub fn remove_code(mut self) -> Self {
        self.code = None;
        self
    }

    /// Sets the location.
    pub fn location(mut self, location: TextBlock<'a>) -> Self {
        self.location = Some(location);
        self
    }

    /// Sets the location as plain text.
    pub fn plain_location(mut self, location: impl Into<Cow<'a, str>>) -> Self {
        self.location = Some(TextBlock::new().add_plain_text(location));
        self
    }

    /// Removes the location.
    pub fn remove_location(mut self) -> Self {
        self.location = None;
        self
    }

    /// Sets whether the date should be shown.
    pub fn show_date(mut self, show_date: bool) -> Self {
        self.show_date = show_date;
        self
    }

    /// Sets whether the thread should be shown.
    pub fn show_thread(mut self, show_thread: bool) -> Self {
        self.show_thread = show_thread;
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> HeaderBlock<'static> {
        HeaderBlock {
            code: self.code.map(|v| Cow::Owned(v.into_owned())),
            location: self.location.map(|v| v.make_owned()),
            show_date: self.show_date,
            show_thread: self.show_thread,
        }
    }
}

impl<'a> Printable for HeaderBlock<'a> {
    fn print<'b>(&'b self, printer: &mut Printer<'b>) {
        // Add tag.
        printer.push_styled_text(
            printer.level.tag().to_ascii_uppercase(),
            Style::new().bold().fg(printer.level.color()),
        );

        // Add code.
        if let Some(code) = &self.code {
            printer.push_styled_text(format!("[{}]", code), Style::new().bold());
        }

        // Add location.
        if let Some(location) = &self.location {
            printer.push_plain_text(Cow::Borrowed(" in "));
            location.print(printer);
        }

        // Add date.
        if self.show_date {
            let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

            printer.push_styled_text(
                Cow::Borrowed(concatcp!("\n ", NEW_LINE_RIGH, " at ")),
                Style::new().bold().fg(printer.level.color()),
            );

            printer.push_styled_text(Cow::Owned(date), Style::new().bold());
        }

        // Add thread.
        if self.show_thread {
            let thread = std::thread::current()
                .name()
                .unwrap_or("undefined")
                .to_string();

            printer.push_styled_text(
                Cow::Borrowed(concatcp!("\n ", NEW_LINE_RIGH, " in thread ")),
                Style::new().bold().fg(printer.level.color()),
            );

            printer.push_styled_text(Cow::Owned(thread), Style::new().bold());
        }
    }
}

impl<'a> Display for HeaderBlock<'a> {
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
    use crate::LogLevel;

    use super::*;

    #[test]
    fn test_plain() {
        // Empty
        let log = HeaderBlock::new();
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "ERROR");

        // Code
        let log = HeaderBlock::new().code("c-xxxxx");
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "ERROR[c-xxxxx]");

        // Location
        let log = HeaderBlock::new().plain_location("src/blocks/header.rs:3:26");
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "ERROR in src/blocks/header.rs:3:26");

        // Date
        let log = HeaderBlock::new().show_date(true);
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();
        let date = &text[14..];

        assert_eq!(text, format!("ERROR\n ↪ at {date}"));

        // Thread
        let thread = std::thread::current()
            .name()
            .unwrap_or("undefined")
            .to_string();
        let log = HeaderBlock::new().show_thread(true);
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, format!("ERROR\n ↪ in thread {thread}"));

        // All
        let log = HeaderBlock::new()
            .code("c-xxxxx")
            .plain_location("src/blocks/header.rs:3:26")
            .show_date(true)
            .show_thread(true);
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();
        let date = &text[52..][..24];

        assert_eq!(
            text,
            format!(
                "ERROR[c-xxxxx] in src/blocks/header.rs:3:26\n ↪ at {date}\n ↪ in thread {thread}"
            )
        );
    }

    #[test]
    fn test_styled() {
        // Empty
        let log = HeaderBlock::new();
        let text = log
            .print_to_string(LogLevel::trace(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;38;5;102mTRACE\u{1b}[0m");

        // Code
        let log = HeaderBlock::new().code("c-xxxxx");
        let text = log
            .print_to_string(LogLevel::debug(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(
            text,
            "\u{1b}[1;32mDEBUG\u{1b}[0m\u{1b}[1m[c-xxxxx]\u{1b}[0m"
        );

        // Location
        let log = HeaderBlock::new().plain_location("src/blocks/header.rs:3:26");
        let text = log
            .print_to_string(LogLevel::info(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(
            text,
            "\u{1b}[1;34mINFO\u{1b}[0m in src/blocks/header.rs:3:26"
        );

        // Date
        let log = HeaderBlock::new().show_date(true);
        let text = log
            .print_to_string(LogLevel::warn(), PrinterFormat::Styled)
            .to_string();
        let date = &text[28..][..24];

        println!("{}", text);
        assert_eq!(
            text,
            format!("\u{1b}[1;33mWARN\n ↪ at \u{1b}[0m\u{1b}[1m{date}\u{1b}[0m")
        );

        // Thread
        let thread = std::thread::current()
            .name()
            .unwrap_or("undefined")
            .to_string();
        let log = HeaderBlock::new().show_thread(true);
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(
            text,
            format!("\u{1b}[1;31mERROR\n ↪ in thread \u{1b}[0m\u{1b}[1m{thread}\u{1b}[0m")
        );

        // All
        let log = HeaderBlock::new()
            .code("c-xxxxx")
            .plain_location("src/blocks/header.rs:3:26")
            .show_date(true)
            .show_thread(true);
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Styled)
            .to_string();
        let date = &text[86..][..24];

        println!("{}", text);
        assert_eq!(
            text,
            format!(
                "\u{1b}[1;31mERROR\u{1b}[0m\u{1b}[1m[c-xxxxx]\u{1b}[0m in src/blocks/header.rs:3:26\u{1b}[1;31m\n ↪ at \u{1b}[0m\u{1b}[1m{date}\u{1b}[0m\u{1b}[1;31m\n ↪ in thread \u{1b}[0m\u{1b}[1m{thread}\u{1b}[0m"
            )
        );
    }
}
