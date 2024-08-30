use crate::printer::{Printable, Printer, PrinterFormat};
use crate::LogLevel;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt::Display;
use yansi::Style;

pub use crate::printer::PaintedElement;

/// A block that prints a formated text to the terminal.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct TextBlock<'a> {
    sections: SmallVec<[PaintedElement<'a>; 3]>,
}

impl<'a> TextBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new empty [TextBlock].
    pub fn new() -> Self {
        Self::default()
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns whether the text block is empty.
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Returns the sections of the text block.
    pub fn get_sections(&self) -> &[PaintedElement<'a>] {
        &self.sections
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a plain text to the block.
    pub fn add_plain_text(self, text: impl Into<Cow<'a, str>>) -> Self {
        self.add_styled_text(text, Style::new())
    }

    /// Adds a styled text to the block.
    pub fn add_styled_text(mut self, text: impl Into<Cow<'a, str>>, style: Style) -> Self {
        let text = text.into();

        if text.is_empty() {
            return self;
        }

        self.sections.push(PaintedElement { text, style });
        self
    }

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> TextBlock<'static> {
        TextBlock {
            sections: self
                .sections
                .into_iter()
                .map(|painted| PaintedElement {
                    text: painted.text.into_owned().into(),
                    style: painted.style,
                })
                .collect(),
        }
    }
}

impl<'a> Printable for TextBlock<'a> {
    fn print<'b>(&'b self, printer: &mut Printer<'b>) {
        for painted in &self.sections {
            printer.push_painted_element(painted.clone());
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
            .add_styled_text("This is\na test", Style::new().bold().yellow())
            .add_plain_text("- plain")
            .add_styled_text(" - styled", Style::new().bold().red());
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "This is\na test- plain - styled");
    }

    #[test]
    fn test_styled() {
        yansi::disable();
        let log = TextBlock::new()
            .add_styled_text("This is\na test", Style::new().bold().yellow())
            .add_plain_text("- plain")
            .add_styled_text(" - styled", Style::new().bold().red());
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
