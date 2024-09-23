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
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct NoteBlock<'a> {
    pub text: TextBlock<'a>,
}

impl<'a> NoteBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new empty [NoteBlock].
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    // BUILDERS ---------------------------------------------------------------

    /// Sets the text.
    #[inline(always)]
    pub fn text(mut self, text: impl Into<TextBlock<'a>>) -> Self {
        self.text = text.into();
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

impl<'a> Printable<'a> for NoteBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
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
    use yansi::Style;

    use crate::LogLevel;

    use super::*;

    #[test]
    fn test_plain() {
        let log = NoteBlock::new()
            .text(TextBlock::new().add_styled_text("NOTE", Style::new().underline().yellow()));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "= NOTE");
    }

    #[test]
    fn test_styled() {
        let log = NoteBlock::new()
            .text(TextBlock::new().add_styled_text("NOTE", Style::new().underline().yellow()));
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m= \u{1b}[0m\u{1b}[4;33mNOTE\u{1b}[0m");
    }
}
