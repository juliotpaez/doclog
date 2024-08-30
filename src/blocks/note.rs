use crate::blocks::TextBlock;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::LogLevel;
use std::fmt::Display;
use yansi::Style;

/// A block that prints a note, i.e. a text prefixed by an equal sign.
///
/// # Examples
/// ```text
/// = <text>
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoteBlock<'a> {
    text: TextBlock<'a>,
}

impl<'a> NoteBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> Self {
        Self {
            text: TextBlock::new(),
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the text.
    pub fn get_text(&self) -> &TextBlock<'a> {
        &self.text
    }

    /// Returns a mutable reference to the text.
    pub fn get_text_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.text
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the text.
    pub fn text(mut self, text: TextBlock<'a>) -> Self {
        self.text = text;
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> NoteBlock<'static> {
        NoteBlock {
            text: self.text.make_owned(),
        }
    }
}

impl<'a> Printable for NoteBlock<'a> {
    fn print<'b>(&'b self, printer: &mut Printer<'b>) {
        printer.push_styled_text("= ", Style::new().bold().fg(printer.level.color()));
        self.text.print(printer);
    }
}

impl<'a> Display for NoteBlock<'a> {
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
    use yansi::{Paint, Style};

    use crate::LogLevel;

    use super::*;

    #[test]
    fn test_plain() {
        let log = NoteBlock::new()
            .text(TextBlock::new().add_styled_text("NOTE", Style::new().underline().yellow()));
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "= NOTE");
    }

    #[test]
    fn test_styled() {
        let log = NoteBlock::new()
            .text(TextBlock::new().add_styled_text("NOTE", Style::new().underline().yellow()));
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m= \u{1b}[0m\u{1b}[4;33mNOTE\u{1b}[0m");
    }
}
