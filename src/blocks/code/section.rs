use crate::blocks::code::CodeBlock;
use crate::blocks::TextBlock;
use crate::constants::{
    HORIZONTAL_BAR, HORIZONTAL_BOTTOM_BAR, HORIZONTAL_TOP_BAR, MIDDLE_DOT, NEW_LINE_LEFT,
    RIGHT_ARROW, TOP_LEFT_CORNER, TOP_RIGHT_CORNER, UP_POINTER, VERTICAL_BAR, VERTICAL_RIGHT_BAR,
};
use crate::printer::Printer;
use crate::utils::cursor::Cursor;
use const_format::concatcp;
use std::borrow::Cow;
use yansi::{Color, Style};

/// A highlighted code section in a code block.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CodeSection<'a> {
    pub(crate) start: Cursor,
    // Exclusive
    pub(crate) end: Cursor,
    pub(crate) message: TextBlock<'a>,
    pub(crate) color: Option<Color>,
    pub(crate) is_multiline_start: bool,
    pub(crate) is_multiline_end: bool,
}

impl<'a> CodeSection<'a> {
    // GETTERS ----------------------------------------------------------------

    /// Returns the size of the section in characters.
    pub fn char_len(&self) -> usize {
        if self.is_cursor() {
            1
        } else {
            self.end.char_offset - self.start.char_offset
        }
    }

    /// Returns whether this section is a cursor.
    pub fn is_cursor(&self) -> bool {
        self.start == self.end
    }

    // METHODS ----------------------------------------------------------------

    /// Prints the actual code of the section.
    pub(crate) fn print_content(
        &self,
        printer: &mut Printer<'a>,
        block: &CodeBlock<'a>,
        next_color: Color,
    ) {
        if self.is_cursor() {
            printer.push_styled_text(concatcp!(MIDDLE_DOT), Style::new().bold().fg(next_color))
        } else {
            let content = match &block.code {
                Cow::Borrowed(code) => {
                    if !block.show_new_line_chars {
                        Cow::Borrowed(self.start.slice(code, &self.end).trim_end_matches('\n'))
                    } else {
                        Cow::Owned(
                            self.start
                                .slice(code, &self.end)
                                .replace('\n', concatcp!(NEW_LINE_LEFT)),
                        )
                    }
                }
                Cow::Owned(code) => {
                    if !block.show_new_line_chars {
                        Cow::Owned(
                            self.start
                                .slice(code, &self.end)
                                .trim_end_matches('\n')
                                .to_string(),
                        )
                    } else {
                        Cow::Owned(
                            self.start
                                .slice(code, &self.end)
                                .replace('\n', concatcp!(NEW_LINE_LEFT)),
                        )
                    }
                }
            };

            printer.push_styled_text(content, Style::new().bold().fg(next_color))
        }
    }

    /// Prints the actual code of the section.
    pub(crate) fn print_underline(&self, printer: &mut Printer<'a>, next_color: Color) {
        // Print start multiline connection.
        if self.is_multiline_start {
            printer.push_styled_text(
                format!(
                    "{TOP_RIGHT_CORNER}{}{RIGHT_ARROW}",
                    concatcp!(HORIZONTAL_BAR).repeat(self.char_len())
                ),
                Style::new().bold().fg(next_color),
            );
            return;
        }

        // Print end multiline connection.
        if self.is_multiline_end {
            if self.message.is_empty() {
                printer.push_styled_text(
                    format!(
                        "{RIGHT_ARROW}{}{TOP_LEFT_CORNER}",
                        concatcp!(HORIZONTAL_BAR).repeat(self.char_len())
                    ),
                    Style::new().bold().fg(next_color),
                );
            } else {
                printer.push_styled_text(
                    format!(
                        "{RIGHT_ARROW}{HORIZONTAL_BAR}{HORIZONTAL_BOTTOM_BAR}{}{TOP_LEFT_CORNER}",
                        concatcp!(HORIZONTAL_BAR).repeat(self.char_len().saturating_sub(2))
                    ),
                    Style::new().bold().fg(next_color),
                );
            }
            return;
        }

        // Print single character.
        if self.char_len() == 1 {
            if self.message.is_empty() {
                printer.push_styled_text(concatcp!(UP_POINTER), Style::new().bold().fg(next_color));
            } else {
                printer
                    .push_styled_text(concatcp!(VERTICAL_BAR), Style::new().bold().fg(next_color));
            }

            return;
        }

        // Print multiple characters.
        printer.push_styled_text(
            format!(
                "{}{}{TOP_LEFT_CORNER}",
                if self.message.is_empty() {
                    TOP_RIGHT_CORNER
                } else {
                    VERTICAL_RIGHT_BAR
                },
                concatcp!(HORIZONTAL_BAR).repeat(self.char_len() - 2)
            ),
            Style::new().bold().fg(next_color),
        );
    }

    /// Prints the actual code of the section.
    pub(crate) fn print_underline_with_message(
        &self,
        printer: &mut Printer<'a>,
        next_color: Color,
    ) {
        // Print start multiline connection.
        if self.is_multiline_start {
            panic!("Multiline start not supported with message.");
        }

        // Print end multiline connection.
        if self.is_multiline_end {
            printer.push_styled_text(
                format!(
                    "{RIGHT_ARROW}{}{HORIZONTAL_TOP_BAR}{HORIZONTAL_BAR}{HORIZONTAL_BAR} ",
                    concatcp!(HORIZONTAL_BAR).repeat(self.char_len())
                ),
                Style::new().bold().fg(next_color),
            );
            return;
        }

        // Print single character.
        if self.char_len() == 1 {
            printer.push_styled_text(
                concatcp!(TOP_RIGHT_CORNER, HORIZONTAL_BAR, HORIZONTAL_BAR, ' '),
                Style::new().bold().fg(next_color),
            );
            return;
        }

        // Print multiple characters.
        printer.push_styled_text(
            format!(
                "{TOP_RIGHT_CORNER}{}{HORIZONTAL_TOP_BAR}{HORIZONTAL_BAR}{HORIZONTAL_BAR} ",
                concatcp!(HORIZONTAL_BAR).repeat(self.char_len() - 2)
            ),
            Style::new().bold().fg(next_color),
        );
    }

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> CodeSection<'static> {
        CodeSection {
            start: self.start,
            end: self.end,
            message: self.message.make_owned(),
            color: self.color,
            is_multiline_start: self.is_multiline_start,
            is_multiline_end: self.is_multiline_end,
        }
    }
}
