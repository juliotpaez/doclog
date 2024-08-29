use crate::printer::{Printable, Printer, PrinterFormat};
use crate::LogLevel;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt::Display;
use yansi::Style;

/// A block that prints a formated text to the terminal.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct TextBlock<'a> {
    sections: SmallVec<[(Cow<'a, str>, Style); 3]>,
}

impl<'a> TextBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new empty [TextBlock].
    pub fn new() -> Self {
        Self::default()
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the sections of the text block.
    pub fn sections(&self) -> &[(Cow<'a, str>, Style)] {
        &self.sections
    }

    /// Returns a mutable reference to the sections of the text block.
    pub fn sections_mut(&mut self) -> &mut SmallVec<[(Cow<'a, str>, Style); 3]> {
        &mut self.sections
    }

    // METHODS ----------------------------------------------------------------

    pub fn add_section(mut self, text: impl Into<Cow<'a, str>>, style: Style) -> Self {
        self.sections.push((text.into(), style));
        self
    }

    pub fn add_plain_section(self, text: impl Into<Cow<'a, str>>) -> Self {
        self.add_section(text, Style::new())
    }

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> TextBlock<'static> {
        TextBlock {
            sections: self
                .sections
                .into_iter()
                .map(|(text, style)| (Cow::Owned(text.into_owned()), style))
                .collect(),
        }
    }
}

impl<'a> Printable for TextBlock<'a> {
    fn print<'b>(&'b self, printer: &mut Printer<'b>) {
        for (text, style) in &self.sections {
            printer.push_styled_str(text.as_ref(), *style);
        }
    }
}

impl<'a> Display for TextBlock<'a> {
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
    use crate::blocks::TextBlock;
    use crate::printer::{Printable, PrinterFormat};
    use crate::LogLevel;
    use yansi::Style;

    #[test]
    fn test_plain() {
        let log = TextBlock::new()
            .add_section("This is\na test", Style::new().bold().yellow())
            .add_plain_section("- plain")
            .add_section(" - styled", Style::new().bold().red());
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "This is\na test- plain - styled");
    }

    #[test]
    fn test_styled() {
        yansi::disable();
        let log = TextBlock::new()
            .add_section("This is\na test", Style::new().bold().yellow())
            .add_plain_section("- plain")
            .add_section(" - styled", Style::new().bold().red());
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(
            text,
            "\u{1b}[1;33mThis is\na test\u{1b}[0m- plain\u{1b}[1;31m - styled\u{1b}[0m"
        );
    }
}
