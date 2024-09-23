use crate::blocks::{LogBlock, TextBlock};
use crate::constants::{
    HORIZONTAL_BAR, RIGHT_ARROW, TOP_RIGHT_CORNER, VERTICAL_BAR, VERTICAL_RIGHT_BAR,
};
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::{LogContent, LogLevel};
use const_format::concatcp;
use std::borrow::Cow;
use std::fmt::Display;
use std::option::Option::Some;
use yansi::Style;

/// A block that prints a section of a document.
#[derive(Default, Debug, Clone)]
pub struct StepsBlock<'a> {
    pub title: TextBlock<'a>,
    pub final_message: TextBlock<'a>,
    pub steps: Box<LogContent<'a>>,
}

impl<'a> StepsBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new empty [StepsBlock].
    pub fn new() -> Self {
        Self {
            title: TextBlock::new(),
            final_message: TextBlock::new(),
            steps: Box::new(LogContent::new()),
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the maximum line to print.
    fn max_line(&self) -> usize {
        self.steps
            .blocks
            .iter()
            .filter_map(|v| match v {
                LogBlock::Code(v) => Some(v.max_line()),
                _ => None,
            })
            .max()
            .unwrap_or(1)
    }

    // BUILDERS ---------------------------------------------------------------

    /// Sets the title.
    #[inline(always)]
    pub fn title(mut self, title: impl Into<TextBlock<'a>>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the final message.
    #[inline(always)]
    pub fn final_message(mut self, final_message: impl Into<TextBlock<'a>>) -> Self {
        self.final_message = final_message.into();
        self
    }

    /// Adds a new step.
    #[inline(always)]
    pub fn add_step(mut self, block: impl Into<LogBlock<'a>>) -> Self {
        self.steps.blocks.push(block.into());
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> StepsBlock<'static> {
        StepsBlock {
            title: self.title.make_owned(),
            final_message: self.final_message.make_owned(),
            steps: Box::new(self.steps.make_owned()),
        }
    }
}

impl<'a> Printable<'a> for StepsBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        let max_line_digits = format!("{}", self.max_line()).len();
        let block_prefix = TextBlock::new().add_styled_text(
            Cow::Borrowed(concatcp!(VERTICAL_BAR, "   ")),
            Style::new().bold().fg(printer.level.color()),
        );

        // Initial message.
        if !self.title.is_empty() {
            printer.push_styled_text(
                format!("{} ", printer.level.symbol()),
                Style::new().bold().fg(printer.level.color()),
            );

            let title_prefix = TextBlock::new().add_styled_text(
                Cow::Borrowed(concatcp!(VERTICAL_BAR, " ")),
                Style::new().bold().fg(printer.level.color()),
            );
            let mut title_printer = printer.derive();

            self.title.print(&mut title_printer);
            title_printer.indent(&title_prefix.sections, false);
            printer.append(title_printer);
        } else {
            printer.push_styled_text(
                format!("{}", printer.level.symbol()),
                Style::new().bold().fg(printer.level.color()),
            );
        }

        // Print steps.
        for block in &self.steps.blocks {
            let print_start = !matches!(block, LogBlock::Separator(_));

            if print_start {
                printer.push_styled_text(
                    Cow::Borrowed(concatcp!(
                        '\n',
                        VERTICAL_RIGHT_BAR,
                        HORIZONTAL_BAR,
                        RIGHT_ARROW,
                        ' '
                    )),
                    Style::new().bold().fg(printer.level.color()),
                );
            } else {
                printer.push_styled_text(
                    Cow::Borrowed(concatcp!('\n', VERTICAL_BAR, "   ")),
                    Style::new().bold().fg(printer.level.color()),
                );
            }

            let mut block_printer = printer.derive();

            match block {
                LogBlock::Code(block) => {
                    block.print_with_options(&mut block_printer, max_line_digits);
                }
                LogBlock::Separator(block) => {
                    block.print(&mut block_printer);
                }
                _ => {
                    block.print(&mut block_printer);
                }
            }

            block_printer.indent(&block_prefix.sections, false);
            printer.append(block_printer);
        }

        // Print last line.
        if !self.final_message.is_empty() {
            printer.push_styled_text(
                Cow::Borrowed(concatcp!(
                    '\n',
                    TOP_RIGHT_CORNER,
                    HORIZONTAL_BAR,
                    RIGHT_ARROW,
                    ' '
                )),
                Style::new().bold().fg(printer.level.color()),
            );

            let message_prefix = TextBlock::new().add_styled_text(
                Cow::Borrowed("    "),
                Style::new().bold().fg(printer.level.color()),
            );
            let mut message_printer = printer.derive();

            self.final_message.print(&mut message_printer);
            message_printer.indent(&message_prefix.sections, false);
            printer.append(message_printer);
        } else {
            printer.push_styled_text(
                Cow::Borrowed(concatcp!('\n', TOP_RIGHT_CORNER, HORIZONTAL_BAR)),
                Style::new().bold().fg(printer.level.color()),
            );
        }
    }
}

impl<'a> Display for StepsBlock<'a> {
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
    use super::*;
    use crate::blocks::{CodeBlock, SeparatorBlock};
    use crate::LogLevel;

    #[test]
    fn test_plain() {
        let code =
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10";

        // Empty
        let log = StepsBlock::new();
        let text = log.print_to_string(LogLevel::trace(), PrinterFormat::Plain);

        assert_eq!(text, "•\n╰─");

        // Title
        let log = StepsBlock::new().title("This is\na title");
        let text = log.print_to_string(LogLevel::debug(), PrinterFormat::Plain);

        assert_eq!(text, "• This is\n│ a title\n╰─");

        // Final message
        let log = StepsBlock::new().final_message("This is\na message");
        let text = log.print_to_string(LogLevel::info(), PrinterFormat::Plain);

        assert_eq!(text, "•\n╰─▶ This is\n    a message");

        // Steps
        let log = StepsBlock::new()
            .add_step(TextBlock::new().add_plain_text("Line 1\nLine 2"))
            .add_step(SeparatorBlock::with_width(20))
            .add_step(TextBlock::new().add_plain_text("Line 1\nLine 2"))
            .add_step(SeparatorBlock::with_white());
        let text = log.print_to_string(LogLevel::warn(), PrinterFormat::Plain);

        assert_eq!(
            text,
            "⚠\n├─▶ Line 1\n│   Line 2\n│   ────────────────────\n├─▶ Line 1\n│   Line 2\n│   \n╰─"
        );

        // All + match line size in code blocks
        let log = StepsBlock::new()
            .title("This is\na title")
            .final_message("This is\na message")
            .add_step(
                CodeBlock::new(code)
                    // Line 3
                    .highlight_section(14..20, None),
            )
            .add_step(SeparatorBlock::with_width(20))
            .add_step(
                CodeBlock::new(code)
                    // Line 8
                    .highlight_section(52..58, None)
                    .next_lines(50),
            )
            .add_step(SeparatorBlock::with_white());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Plain);

        assert_eq!(text, "× This is\n│ a title\n├─▶  × ╭─\n│    3 │    Line 3\n│      │    ╰────╯\n│      ╰─\n│   ────────────────────\n├─▶  × ╭─\n│    8 │    Line 8\n│      │       ╰────▶\n│    9 │    Line 9\n│      │  ▶──╯\n│   10 │    Line 10\n│      ╰─\n│   \n╰─▶ This is\n    a message");
    }

    #[test]
    fn test_styled() {
        let code =
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10";

        // Empty
        let log = StepsBlock::new();
        let text = log.print_to_string(LogLevel::trace(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;38;5;102m•\n╰─\u{1b}[0m");

        // Title
        let log = StepsBlock::new().title("This is\na title");
        let text = log.print_to_string(LogLevel::debug(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;32m• \u{1b}[0mThis is\n\u{1b}[1;32m│ \u{1b}[0ma title\n\u{1b}[1;32m╰─\u{1b}[0m");

        // Final message
        let log = StepsBlock::new().final_message("This is\na message");
        let text = log.print_to_string(LogLevel::info(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;34m•\n╰─▶ \u{1b}[0mThis is\n    a message");

        // Steps
        let log = StepsBlock::new()
            .add_step(TextBlock::new().add_plain_text("Line 1\nLine 2"))
            .add_step(SeparatorBlock::with_width(20))
            .add_step(TextBlock::new().add_plain_text("Line 1\nLine 2"))
            .add_step(SeparatorBlock::with_white());
        let text = log.print_to_string(LogLevel::warn(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;33m⚠\n├─▶ \u{1b}[0mLine 1\n\u{1b}[1;33m│   \u{1b}[0mLine 2\n\u{1b}[1;33m│   ────────────────────\n├─▶ \u{1b}[0mLine 1\n\u{1b}[1;33m│   \u{1b}[0mLine 2\n\u{1b}[1;33m│   \n╰─\u{1b}[0m");

        // All + match line size in code blocks
        let log = StepsBlock::new()
            .title("This is\na title")
            .final_message("This is\na message")
            .add_step(
                CodeBlock::new(code)
                    // Line 3
                    .highlight_section(14..20, None),
            )
            .add_step(SeparatorBlock::with_width(20))
            .add_step(
                CodeBlock::new(code)
                    // Line 8
                    .highlight_section(52..58, None)
                    .next_lines(50),
            )
            .add_step(SeparatorBlock::with_white());
        let text = log.print_to_string(LogLevel::error(), PrinterFormat::Styled);

        println!("{}", text);
        assert_eq!(text, "\u{1b}[1;31m× \u{1b}[0mThis is\n\u{1b}[1;31m│ \u{1b}[0ma title\n\u{1b}[1;31m├─▶  × \u{1b}[0m\u{1b}[1m╭─\n\u{1b}[0m\u{1b}[1;31m│   \u{1b}[0m\u{1b}[1;90m 3 \u{1b}[0m\u{1b}[1m│    \u{1b}[0m\u{1b}[1;31mLine 3\n│      \u{1b}[0m\u{1b}[1m│    \u{1b}[0m\u{1b}[1;31m╰────╯\n│      \u{1b}[0m\u{1b}[1m╰─\n\u{1b}[0m\u{1b}[1;31m│   ────────────────────\n├─▶  × \u{1b}[0m\u{1b}[1m╭─\n\u{1b}[0m\u{1b}[1;31m│   \u{1b}[0m\u{1b}[1;90m 8 \u{1b}[0m\u{1b}[1m│    \u{1b}[0mLin\u{1b}[1;31me 8\n│      \u{1b}[0m\u{1b}[1m│       \u{1b}[0m\u{1b}[1;31m╰────▶\n│   \u{1b}[0m\u{1b}[1;90m 9 \u{1b}[0m\u{1b}[1m│    \u{1b}[0m\u{1b}[1;31mLi\u{1b}[0mne 9\n\u{1b}[1;31m│      \u{1b}[0m\u{1b}[1m│  \u{1b}[0m\u{1b}[1;31m▶──╯\n│   \u{1b}[0m\u{1b}[1;90m10 \u{1b}[0m\u{1b}[1m│    \u{1b}[0mLine 10\n\u{1b}[1;31m│      \u{1b}[0m\u{1b}[1m╰─\n\u{1b}[0m\u{1b}[1;31m│   \n╰─▶ \u{1b}[0mThis is\n    a message");
    }
}
