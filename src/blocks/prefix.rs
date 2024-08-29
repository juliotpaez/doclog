use crate::blocks::TextBlock;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::LogContent;
use std::fmt::Display;

/// Prints any content with a custom indentation.
#[derive(Default, Debug, Clone)]
pub struct PrefixBlock<'a> {
    prefix: TextBlock<'a>,
    content: Box<LogContent<'a>>,
}

impl<'a> PrefixBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> Self {
        Self::default()
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the prefix.
    pub fn prefix(&self) -> &TextBlock<'a> {
        &self.prefix
    }

    /// Returns a mutable reference to the prefix.
    pub fn prefix_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.prefix
    }

    /// Returns the inner content.
    pub fn content(&self) -> &LogContent<'a> {
        &self.content
    }

    /// Returns a mutable reference to the inner content.
    pub fn content_mut(&mut self) -> &mut LogContent<'a> {
        &mut self.content
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the prefix.
    pub fn set_prefix(mut self, prefix: TextBlock<'a>) -> Self {
        self.prefix = prefix;
        self
    }

    /// Sets the inner content.
    pub fn set_content(mut self, content: LogContent<'a>) -> Self {
        self.content = Box::new(content);
        self
    }

    // METHODS ----------------------------------------------------------------

    pub fn make_owned(self) -> PrefixBlock<'static> {
        PrefixBlock {
            prefix: self.prefix.make_owned(),
            content: Box::new(self.content.make_owned()),
        }
    }
}

impl<'a> Printable for PrefixBlock<'a> {
    fn print<'b>(&'b self, printer: &mut Printer<'b>) {
        let mut prefix_printer = Printer::new(printer.format);
        self.prefix.print(&mut prefix_printer);

        let mut content_printer = Printer::new(printer.format);
        self.content.print(&mut content_printer);

        content_printer.indent(&prefix_printer, true);
        printer.append(content_printer);
    }
}

impl<'a> Display for PrefixBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = Printer::new(PrinterFormat::Plain);
        self.print(&mut printer);
        printer.fmt(f, PrinterFormat::Plain)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::blocks::{PrefixBlock, TextBlock};
    use crate::printer::{Printable, PrinterFormat};
    use crate::LogContent;
    use yansi::Style;

    #[test]
    fn test_plain() {
        let log = PrefixBlock::new()
            .set_prefix(TextBlock::new().add_section(" | -> ", Style::new().bold().blue()))
            .set_content(LogContent::new().add_block(TextBlock::new().add_section(
                "The message\nin\nmultiple\nlines",
                Style::new().bold().red(),
            )));
        let text = log.print_to_string(PrinterFormat::Plain).to_string();

        assert_eq!(
            text,
            " | -> The message\n | -> in\n | -> multiple\n | -> lines"
        );
    }

    #[test]
    fn test_styled() {
        let log = PrefixBlock::new()
            .set_prefix(TextBlock::new().add_section(" | -> ", Style::new().bold().blue()))
            .set_content(LogContent::new().add_block(TextBlock::new().add_section(
                "The message\nin\nmultiple\nlines",
                Style::new().bold().red(),
            )));
        let text = log.print_to_string(PrinterFormat::Styled).to_string();

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;34m | -> \u{1b}[0m\u{1b}[1;31mThe message\n\u{1b}[0m\u{1b}[1;34m | -> \u{1b}[0m\u{1b}[1;31min\n\u{1b}[0m\u{1b}[1;34m | -> \u{1b}[0m\u{1b}[1;31mmultiple\n\u{1b}[0m\u{1b}[1;34m | -> \u{1b}[0m\u{1b}[1;31mlines\u{1b}[0m");
    }
}
