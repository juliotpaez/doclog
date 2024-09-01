use crate::blocks::code::CodeBlock;
use crate::blocks::TextBlock;
use crate::constants::{
    HORIZONTAL_BAR, HORIZONTAL_BOTTOM_BAR, MIDDLE_DOT, NEW_LINE_LEFT, RIGHT_ARROW, TOP_LEFT_CORNER,
    TOP_RIGHT_CORNER, UP_POINTER, VERTICAL_BAR, VERTICAL_RIGHT_BAR,
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

    /// Returns whether the section ends with a new line.
    pub(crate) fn is_ended_by_new_line(&self, text: &str) -> bool {
        let text = &text[self.end.byte_offset..];

        match text.chars().next() {
            Some(v) => v == '\n',
            None => false,
        }
    }

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
    pub(crate) fn print_underline(
        &self,
        printer: &mut Printer<'a>,
        block: &CodeBlock<'a>,
        next_color: Color,
    ) {
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

    // TODO
    // pub fn print(
    //     &self,
    //     buffer: &mut String,
    //     in_ansi: bool,
    //     arrow_line: usize,
    //     arrow_lines_height: usize,
    //     message_column: usize,
    //     max_line_num_digits: usize,
    // ) -> bool {
    //     if arrow_line == 0 {
    //         self.print_first_arrow_line(
    //             buffer,
    //             in_ansi,
    //             message_column,
    //             arrow_lines_height == 1,
    //             max_line_num_digits,
    //         );
    //         arrow_lines_height == 1 && self.message.is_some()
    //     } else if self.message.is_some() && arrow_line + 1 == arrow_lines_height {
    //         self.print_last_arrow_line(buffer, in_ansi, message_column, max_line_num_digits);
    //         true
    //     } else {
    //         self.print_middle_arrow_line(buffer, in_ansi);
    //         false
    //     }
    // }
    //
    // pub fn print_first_arrow_line(
    //     &self,
    //     buffer: &mut String,
    //     in_ansi: bool,
    //     message_column: usize,
    //     print_message: bool,
    //     max_line_num_digits: usize,
    // ) {
    //     let char_length = self.char_len();
    //
    //     if self.is_multiline_start {
    //         buffer.push_str(&color_bold_if(
    //             format!(
    //                 "{}{}{}",
    //                 TOP_RIGHT_CORNER,
    //                 HORIZONTAL_BAR.repeat(char_length),
    //                 RIGHT_POINTER
    //             ),
    //             self.color.unwrap(),
    //             in_ansi,
    //         ));
    //     } else if self.is_multiline_end {
    //         buffer.pop().unwrap();
    //
    //         if self.message.is_some() {
    //             if print_message {
    //                 // With message at first line.
    //                 if char_length > 0 {
    //                     buffer.push_str(&color_bold_if(
    //                         format!(
    //                             "{}{}{}",
    //                             RIGHT_POINTER,
    //                             HORIZONTAL_BAR.repeat(char_length - 1),
    //                             HORIZONTAL_TOP_BAR
    //                         ),
    //                         self.color.unwrap(),
    //                         in_ansi,
    //                     ));
    //                 } else {
    //                     buffer.push_str(&color_bold_if(
    //                         format!("{}{}", RIGHT_POINTER, HORIZONTAL_TOP_BAR),
    //                         self.color.unwrap(),
    //                         in_ansi,
    //                     ));
    //                 }
    //
    //                 self.print_message(buffer, in_ansi, message_column, max_line_num_digits);
    //             } else {
    //                 // With message at other line.
    //                 if char_length <= 1 {
    //                     buffer.push_str(&color_bold_if(
    //                         format!("{}{}", RIGHT_POINTER, VERTICAL_LEFT_BAR),
    //                         self.color.unwrap(),
    //                         in_ansi,
    //                     ));
    //                 } else {
    //                     buffer.push_str(&color_bold_if(
    //                         format!(
    //                             "{}{}{}{}",
    //                             RIGHT_POINTER,
    //                             HORIZONTAL_BOTTOM_BAR,
    //                             HORIZONTAL_BAR.repeat(char_length - 2),
    //                             TOP_LEFT_CORNER
    //                         ),
    //                         self.color.unwrap(),
    //                         in_ansi,
    //                     ));
    //                 }
    //             }
    //         } else {
    //             // No message.
    //             if char_length > 0 {
    //                 buffer.push_str(&color_bold_if(
    //                     format!(
    //                         "{}{}{}",
    //                         RIGHT_POINTER,
    //                         HORIZONTAL_BAR.repeat(char_length - 1),
    //                         TOP_LEFT_CORNER
    //                     ),
    //                     self.color.unwrap(),
    //                     in_ansi,
    //                 ));
    //             } else {
    //                 buffer.push_str(&color_bold_if(
    //                     format!("{}{}", RIGHT_POINTER, TOP_LEFT_CORNER),
    //                     self.color.unwrap(),
    //                     in_ansi,
    //                 ));
    //             }
    //         }
    //     } else {
    //         match char_length {
    //             0 | 1 => {
    //                 if self.message.is_some() {
    //                     if print_message {
    //                         // With message at first line.
    //                         buffer.push_str(&color_bold_if(
    //                             TOP_RIGHT_CORNER.to_string(),
    //                             self.color.unwrap(),
    //                             in_ansi,
    //                         ));
    //                         self.print_message(
    //                             buffer,
    //                             in_ansi,
    //                             message_column,
    //                             max_line_num_digits,
    //                         );
    //                     } else {
    //                         // With message at other line.
    //                         buffer.push_str(&color_bold_if(
    //                             VERTICAL_BAR.to_string(),
    //                             self.color.unwrap(),
    //                             in_ansi,
    //                         ));
    //                     }
    //                 } else {
    //                     // No message.
    //                     buffer.push_str(&color_bold_if(
    //                         UP_POINTER.to_string(),
    //                         self.color.unwrap(),
    //                         in_ansi,
    //                     ));
    //                 }
    //             }
    //             _ => {
    //                 if self.message.is_some() {
    //                     if print_message {
    //                         // With message at first line.
    //                         buffer.push_str(&color_bold_if(
    //                             format!(
    //                                 "{}{}{}",
    //                                 TOP_RIGHT_CORNER,
    //                                 HORIZONTAL_BAR.repeat(char_length - 2),
    //                                 HORIZONTAL_TOP_BAR
    //                             ),
    //                             self.color.unwrap(),
    //                             in_ansi,
    //                         ));
    //                         self.print_message(
    //                             buffer,
    //                             in_ansi,
    //                             message_column,
    //                             max_line_num_digits,
    //                         );
    //                     } else {
    //                         // With message at other line.
    //                         buffer.push_str(&color_bold_if(
    //                             format!(
    //                                 "{}{}{}",
    //                                 VERTICAL_RIGHT_BAR,
    //                                 HORIZONTAL_BAR.repeat(char_length - 2),
    //                                 TOP_LEFT_CORNER
    //                             ),
    //                             self.color.unwrap(),
    //                             in_ansi,
    //                         ));
    //                     }
    //                 } else {
    //                     // No message.
    //                     buffer.push_str(&color_bold_if(
    //                         format!(
    //                             "{}{}{}",
    //                             TOP_RIGHT_CORNER,
    //                             HORIZONTAL_BAR.repeat(char_length - 2),
    //                             TOP_LEFT_CORNER
    //                         ),
    //                         self.color.unwrap(),
    //                         in_ansi,
    //                     ));
    //                 }
    //             }
    //         }
    //     }
    // }
    //
    // pub fn print_middle_arrow_line(&self, buffer: &mut String, in_ansi: bool) {
    //     let char_length = self.char_len();
    //
    //     match char_length {
    //         0 | 1 => {
    //             if self.message.is_some() {
    //                 buffer.push_str(&color_bold_if(
    //                     VERTICAL_BAR.to_string(),
    //                     self.color.unwrap(),
    //                     in_ansi,
    //                 ));
    //             } else {
    //                 buffer.push(' ');
    //             }
    //         }
    //         _ => {
    //             if self.message.is_some() {
    //                 buffer.push_str(&color_bold_if(
    //                     VERTICAL_BAR.to_string(),
    //                     self.color.unwrap(),
    //                     in_ansi,
    //                 ));
    //                 buffer.push_str(&" ".repeat(char_length - 1));
    //             } else {
    //                 buffer.push_str(&" ".repeat(char_length));
    //             }
    //         }
    //     }
    // }
    //
    // pub fn print_last_arrow_line(
    //     &self,
    //     buffer: &mut String,
    //     in_ansi: bool,
    //     message_column: usize,
    //     max_line_num_digits: usize,
    // ) {
    //     buffer.push_str(&color_bold_if(
    //         TOP_RIGHT_CORNER.to_string(),
    //         self.color.unwrap(),
    //         in_ansi,
    //     ));
    //     self.print_message(buffer, in_ansi, message_column, max_line_num_digits);
    // }
    //
    // pub fn print_message(
    //     &self,
    //     buffer: &mut String,
    //     in_ansi: bool,
    //     message_column: usize,
    //     max_line_num_digits: usize,
    // ) {
    //     let message = self
    //         .message
    //         .as_ref()
    //         .expect("Cannot call print_last_arrow_line without a message");
    //
    //     let line_start_offset = match memchr::memrchr(b'\n', buffer.as_bytes()) {
    //         Some(v) => v + 1,
    //         None => 0,
    //     };
    //     let line_content = &buffer[line_start_offset..];
    //     let mut num_chars = if in_ansi {
    //         bytecount::num_chars(remove_ansi_escapes(line_content).as_bytes())
    //     } else {
    //         bytecount::num_chars(line_content.as_bytes())
    //     };
    //
    //     // Remove constant part.
    //     num_chars -= 6 + max_line_num_digits;
    //
    //     let bars = HORIZONTAL_BAR.repeat(message_column - num_chars);
    //
    //     buffer.push_str(&color_bold_if(bars, self.color.unwrap(), in_ansi));
    //     buffer.push(' ');
    //     buffer.push_str(&remove_jump_lines(message));
    // }

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
