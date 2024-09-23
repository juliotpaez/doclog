use crate::constants::HORIZONTAL_BAR;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::LogLevel;
use const_format::{concatcp, formatcp};
use std::borrow::Cow;
use std::fmt::Display;
use yansi::Style;

const N_HORIZONTAL_BARS: usize = 100;
const HORIZONTAL_BARS: &str = formatcp!("{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}{0}", HORIZONTAL_BAR);
const _: () = {
    assert!(HORIZONTAL_BARS.len() == HORIZONTAL_BAR.len_utf8() * N_HORIZONTAL_BARS);
};

/// A block that prints a line separator repeating a character.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SeparatorBlock {
    pub width: usize,
    character: char,
}

impl SeparatorBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new [SeparatorBlock].
    ///
    /// # Panic
    /// This method panics if `character` is a newline character.
    #[inline(always)]
    pub fn new(width: usize, character: char) -> Self {
        assert_ne!(
            character, '\n',
            "The character cannot be a newline character."
        );
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

    /// Creates a new [SeparatorBlock] representing a white line.
    #[inline(always)]
    pub fn with_white() -> Self {
        Self {
            width: 0,
            character: ' ',
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The character used to repeat the separator.
    #[inline(always)]
    pub fn get_character(&self) -> char {
        self.character
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the character used to repeat the separator.
    ///
    /// # Panic
    /// This method panics if `character` is a newline character.
    #[inline(always)]
    pub fn set_character(&mut self, character: char) {
        assert_ne!(
            character, '\n',
            "The character cannot be a newline character."
        );
        self.character = character;
    }

    // BUILDERS ---------------------------------------------------------------

    /// Sets the width of the separator.
    #[inline(always)]
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Sets the character used to repeat the separator.
    ///
    /// # Panic
    /// This method panics if `character` is a newline character.
    #[inline(always)]
    pub fn character(mut self, character: char) -> Self {
        assert_ne!(
            character, '\n',
            "The character cannot be a newline character."
        );
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

        let separator = match self.character {
            // Whitespaces are not seen in the terminal, so we use an empty string to skip it.
            c if c.is_whitespace() => Cow::Borrowed(""),
            HORIZONTAL_BAR => {
                if self.width < N_HORIZONTAL_BARS {
                    Cow::Borrowed(&HORIZONTAL_BARS[0..(self.width * HORIZONTAL_BAR.len_utf8())])
                } else {
                    Cow::Owned(concatcp!(HORIZONTAL_BAR).repeat(self.width))
                }
            }
            _ => Cow::Owned(format!("{}", self.character).repeat(self.width)),
        };
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
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "");

        let log = SeparatorBlock::new(10, '/');
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "//////////");

        let log = SeparatorBlock::with_width(10);
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

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
