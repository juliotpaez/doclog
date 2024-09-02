use crate::blocks::TextBlock;
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::{LogContent, LogLevel};
use std::fmt::Display;

/// Prints any content prefixed with a text block.
///
/// When printed, prefix will get all newline characters `\n`
/// replaced by whitespaces to only occupy one line.
#[derive(Default, Debug, Clone)]
pub struct PrefixBlock<'a> {
    prefix: TextBlock<'a>,
    content: Box<LogContent<'a>>,
}

impl<'a> PrefixBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new empty [PrefixBlock].
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the prefix.
    #[inline(always)]
    pub fn get_prefix(&self) -> &TextBlock<'a> {
        &self.prefix
    }

    /// Returns a mutable reference to the prefix.
    #[inline(always)]
    pub fn get_prefix_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.prefix
    }

    /// Returns the inner content.
    #[inline(always)]
    pub fn get_content(&self) -> &LogContent<'a> {
        &self.content
    }

    /// Returns a mutable reference to the inner content.
    #[inline(always)]
    pub fn get_content_mut(&mut self) -> &mut LogContent<'a> {
        &mut self.content
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the prefix.
    #[inline(always)]
    pub fn prefix(mut self, prefix: impl Into<TextBlock<'a>>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Sets the inner content.
    #[inline(always)]
    pub fn content(mut self, content: LogContent<'a>) -> Self {
        self.content = Box::new(content);
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> PrefixBlock<'static> {
        PrefixBlock {
            prefix: self.prefix.make_owned(),
            content: Box::new(self.content.make_owned()),
        }
    }
}

impl<'a> Printable<'a> for PrefixBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        let mut content_printer = printer.derive();
        self.content.print(&mut content_printer);

        let prefix = self.prefix.single_lined();
        content_printer.indent(prefix.get_sections(), true);
        printer.append(content_printer);
    }
}

impl<'a> Display for PrefixBlock<'a> {
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
    use crate::blocks::{PrefixBlock, TextBlock};
    use crate::printer::{Printable, PrinterFormat};
    use crate::{LogContent, LogLevel};
    use yansi::Style;

    #[test]
    fn test_plain() {
        let log = PrefixBlock::new()
            .prefix(TextBlock::new().add_styled_text(" |\n-> ", Style::new().bold().blue()))
            .content(
                LogContent::new().add_block(TextBlock::new().add_styled_text(
                    "The message\nin\nmultiple\nlines",
                    Style::new().bold().red(),
                )),
            );
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(
            text,
            " | -> The message\n | -> in\n | -> multiple\n | -> lines"
        );
    }

    #[test]
    fn test_styled() {
        let log = PrefixBlock::new()
            .prefix(TextBlock::new().add_styled_text(" |\n-> ", Style::new().bold().blue()))
            .content(
                LogContent::new().add_block(TextBlock::new().add_styled_text(
                    "The message\nin\nmultiple\nlines",
                    Style::new().bold().red(),
                )),
            );
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;34m | -> \u{1b}[0m\u{1b}[1;31mThe message\n\u{1b}[0m\u{1b}[1;34m | -> \u{1b}[0m\u{1b}[1;31min\n\u{1b}[0m\u{1b}[1;34m | -> \u{1b}[0m\u{1b}[1;31mmultiple\n\u{1b}[0m\u{1b}[1;34m | -> \u{1b}[0m\u{1b}[1;31mlines\u{1b}[0m");
    }
}
