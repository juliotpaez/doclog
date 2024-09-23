use crate::blocks::TextBlock;
use crate::constants::NEW_LINE_RIGHT;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::utils::text::remove_jump_lines;
use crate::utils::whitespaces::build_space_string;
use crate::LogLevel;
use chrono::{SecondsFormat, Utc};
use const_format::concatcp;
use std::borrow::Cow;
use std::fmt::Display;
use yansi::Style;

/// A block that prints a title, showing the type of log and the message.
/// It optionally shows the current date and thread.
///
/// When printed, code will get all newline characters `\n`
/// replaced by whitespaces to only occupy one line.
///
/// # Examples
/// ```text
/// info[code] in /path/to/file.rs
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct HeaderBlock<'a> {
    pub title: TextBlock<'a>,
    pub code: Cow<'a, str>,
    pub location: TextBlock<'a>,
    pub show_date: bool,
    pub show_thread: bool,
    pub extra_messages: Vec<TextBlock<'a>>,
}

impl<'a> HeaderBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new empty [HeaderBlock].
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    // BUILDERS ---------------------------------------------------------------

    /// Sets the title.
    #[inline(always)]
    pub fn title(mut self, title: impl Into<TextBlock<'a>>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the code.
    #[inline(always)]
    pub fn code(mut self, code: impl Into<Cow<'a, str>>) -> Self {
        self.code = code.into();
        self
    }

    /// Sets the location.
    #[inline(always)]
    pub fn location(mut self, location: impl Into<TextBlock<'a>>) -> Self {
        self.location = location.into();
        self
    }

    /// Sets whether the date should be shown.
    #[inline(always)]
    pub fn show_date(mut self, show_date: bool) -> Self {
        self.show_date = show_date;
        self
    }

    /// Sets whether the thread should be shown.
    #[inline(always)]
    pub fn show_thread(mut self, show_thread: bool) -> Self {
        self.show_thread = show_thread;
        self
    }

    /// Adds an extra message.
    #[inline(always)]
    pub fn add_extra_message(mut self, message: impl Into<TextBlock<'a>>) -> Self {
        self.extra_messages.push(message.into());
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> HeaderBlock<'static> {
        HeaderBlock {
            title: self.title.make_owned(),
            code: Cow::Owned(self.code.into_owned()),
            location: self.location.make_owned(),
            show_date: self.show_date,
            show_thread: self.show_thread,
            extra_messages: self
                .extra_messages
                .into_iter()
                .map(|v| v.make_owned())
                .collect(),
        }
    }
}

impl<'a> Printable<'a> for HeaderBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        // Add tag.
        printer.push_styled_text(
            printer.level.tag().to_ascii_uppercase(),
            Style::new().bold().fg(printer.level.color()),
        );

        // Add code.
        if !self.code.is_empty() {
            printer.push_styled_text(
                format!("[{}]", remove_jump_lines(self.code.as_ref())),
                Style::new().bold(),
            );
        }

        // Add title.
        if !self.title.is_empty() {
            printer.push_plain_text(Cow::Borrowed(" "));

            let prefix = TextBlock::new_plain(build_space_string(printer.level.tag().len() + 1));
            let mut location_printer = printer.derive();

            self.title.print(&mut location_printer);
            location_printer.indent(&prefix.sections, false);
            printer.append(location_printer);
        }

        // Add location.
        if !self.location.is_empty() {
            printer.push_styled_text(
                Cow::Borrowed(concatcp!("\n ", NEW_LINE_RIGHT, " in ")),
                Style::new().bold().fg(printer.level.color()),
            );

            let prefix = TextBlock::new_plain(Cow::Borrowed("      "));
            let mut location_printer = printer.derive();

            self.location.print(&mut location_printer);
            location_printer.indent(&prefix.sections, false);
            printer.append(location_printer);
        }

        // Add date.
        if self.show_date {
            let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

            printer.push_styled_text(
                Cow::Borrowed(concatcp!("\n ", NEW_LINE_RIGHT, " at ")),
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
                Cow::Borrowed(concatcp!("\n ", NEW_LINE_RIGHT, " in thread ")),
                Style::new().bold().fg(printer.level.color()),
            );

            printer.push_styled_text(Cow::Owned(thread), Style::new().bold());
        }

        // Add extra messages.
        let prefix = TextBlock::new_plain(Cow::Borrowed("   "));
        for message in &self.extra_messages {
            printer.push_styled_text(
                Cow::Borrowed(concatcp!("\n ", NEW_LINE_RIGHT, " ")),
                Style::new().bold().fg(printer.level.color()),
            );

            let mut location_printer = printer.derive();

            message.print(&mut location_printer);
            location_printer.indent(&prefix.sections, false);
            printer.append(location_printer);
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
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "ERROR");

        // Code
        let log = HeaderBlock::new().code("c-xxxxx");
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "ERROR[c-xxxxx]");

        // Title
        let log = HeaderBlock::new().title("This is\na title");
        let text = log.print_to_string(LogLevel::debug(), PrinterFormat::Plain);

        println!("{}", text);
        assert_eq!(text, "DEBUG This is\n      a title");

        // Location
        let log = HeaderBlock::new().location(TextBlock::new_plain("src/blocks/\n/header.rs:3:26"));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "ERROR\n ↪ in src/blocks/\n      /header.rs:3:26");

        // Date
        let log = HeaderBlock::new().show_date(true);
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);
        let date = &text[14..];

        assert_eq!(text, format!("ERROR\n ↪ at {date}"));

        // Thread
        let thread = std::thread::current()
            .name()
            .unwrap_or("undefined")
            .to_string();
        let log = HeaderBlock::new().show_thread(true);
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, format!("ERROR\n ↪ in thread {thread}"));

        // Extra messages
        let log = HeaderBlock::new().add_extra_message("Line1\nLine2");
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, format!("ERROR\n ↪ Line1\n   Line2"));

        // All
        let log = HeaderBlock::new()
            .title("This is\na title")
            .code("c-xxxxx")
            .location(TextBlock::new_plain("src/blocks/header.rs:3:26"))
            .show_date(true)
            .show_thread(true)
            .add_extra_message("Line1\nLine2");
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);
        let date = &text[79..][..24];

        assert_eq!(
            text,
            format!(
                "ERROR[c-xxxxx] This is\n      a title\n ↪ in src/blocks/header.rs:3:26\n ↪ at {date}\n ↪ in thread {thread}\n ↪ Line1\n   Line2"
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

        // Title
        let log = HeaderBlock::new().title("This is\na title");
        let text = log
            .print_to_string(LogLevel::debug(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;32mDEBUG \u{1b}[0mThis is\n      a title");

        // Location
        let log = HeaderBlock::new().location(TextBlock::new_plain("src/blocks/\n/header.rs:3:26"));
        let text = log
            .print_to_string(LogLevel::info(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(
            text,
            "\u{1b}[1;34mINFO\n ↪ in \u{1b}[0msrc/blocks/\n      /header.rs:3:26"
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
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(
            text,
            format!("\u{1b}[1;31mERROR\n ↪ in thread \u{1b}[0m\u{1b}[1m{thread}\u{1b}[0m")
        );

        // Extra messages
        let log = HeaderBlock::new().add_extra_message("Line1\nLine2");
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(
            text,
            format!("\u{1b}[1;31mERROR\n ↪ \u{1b}[0mLine1\n   Line2")
        );

        // All
        let log = HeaderBlock::new()
            .title("This is\na title")
            .code("c-xxxxx")
            .location(TextBlock::new_plain("src/blocks/header.rs:3:26"))
            .show_date(true)
            .show_thread(true)
            .add_extra_message("Line1\nLine2");
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);
        let date = &text[124..][..24];

        println!("{}", text);
        assert_eq!(
            text,
            format!(
                "\u{1b}[1;31mERROR\u{1b}[0m\u{1b}[1m[c-xxxxx] \u{1b}[0mThis is\n      a title\n\u{1b}[1;31m ↪ in \u{1b}[0msrc/blocks/header.rs:3:26\n\u{1b}[1;31m ↪ at \u{1b}[0m\u{1b}[1m{date}\n\u{1b}[0m\u{1b}[1;31m ↪ in thread \u{1b}[0m\u{1b}[1m{thread}\n\u{1b}[0m\u{1b}[1;31m ↪ \u{1b}[0mLine1\n   Line2"
            )
        );
    }
}
