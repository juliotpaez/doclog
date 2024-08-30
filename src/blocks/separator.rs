use crate::constants::HORIZONTAL_BAR;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::LogLevel;
use std::fmt::Display;
use yansi::Style;

/// A block that prints a line separator repeating a character.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SeparatorBlock {
    width: usize,
    character: char,
}

impl SeparatorBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new [SeparatorBlock].
    #[inline(always)]
    pub fn new(width: usize, character: char) -> Self {
        Self { width, character }
    }

    /// Creates a new [SeparatorBlock] with a width of `width` using the [HORIZONTAL_BAR] character.
    #[inline(always)]
    pub fn with_width(width: usize) -> Self {
        Self {
            width,
            character: HORIZONTAL_BAR,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The width of the separator.
    #[inline(always)]
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// The character used to repeat the separator.
    #[inline(always)]
    pub fn get_character(&self) -> char {
        self.character
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the width of the separator.
    #[inline(always)]
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Sets the character used to repeat the separator.
    #[inline(always)]
    pub fn character(mut self, character: char) -> Self {
        self.character = character;
        self
    }
}

impl<'a> Printable<'a> for SeparatorBlock {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        if self.width == 0 {
            return;
        }

        let separator = format!("{}", self.character).repeat(self.width);
        printer.push_styled_text(separator, Style::new().bold().fg(printer.level.color()));
    }
}

impl Display for SeparatorBlock {
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
    use crate::blocks::SeparatorBlock;
    use crate::printer::{Printable, PrinterFormat};
    use crate::LogLevel;

    #[test]
    fn test_plain() {
        let log = SeparatorBlock::new(0, '/');
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "");

        let log = SeparatorBlock::new(10, '/');
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "//////////");

        let log = SeparatorBlock::with_width(10);
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "──────────");
    }

    #[test]
    fn test_styled() {
        let log = SeparatorBlock::new(0, '/');
        let text = log
            .print_to_string(LogLevel::info(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(text, "");

        let log = SeparatorBlock::new(10, '/');
        let text = log
            .print_to_string(LogLevel::info(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;34m//////////\u{1b}[0m");

        let log = SeparatorBlock::with_width(10);
        let text = log
            .print_to_string(LogLevel::info(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;34m──────────\u{1b}[0m");
    }
}
