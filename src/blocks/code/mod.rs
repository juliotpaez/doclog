use section::*;
mod section;

use crate::blocks::TextBlock;
use crate::constants::{
    BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, NEW_LINE_LEFT, TOP_RIGHT_CORNER, VERTICAL_BAR,
};
use crate::printer::{Printable, Printer, PrinterFormat};
use crate::utils::cursor::Cursor;
use crate::utils::whitespaces::{build_space_string, build_whitespace_string};
use crate::LogLevel;
use const_format::concatcp;
use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Range;
use std::option::Option::Some;
use yansi::{Color, Style};

/// A block that prints a section of a document.
#[derive(Debug, Clone)]
pub struct CodeBlock<'a> {
    code: Cow<'a, str>,
    sections: Vec<CodeSection<'a>>,
    title: TextBlock<'a>,
    file_path: TextBlock<'a>,
    final_message: TextBlock<'a>,
    show_new_line_chars: bool,
    secondary_color: Color,
    previous_lines: usize,
    next_lines: usize,
    middle_lines: usize,
}

impl<'a> CodeBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new [CodeBlock] with the given code.
    pub fn new(code: impl Into<Cow<'a, str>>) -> Self {
        Self {
            code: code.into(),
            sections: Vec::new(),
            title: TextBlock::new(),
            file_path: TextBlock::new(),
            final_message: TextBlock::new(),
            show_new_line_chars: false,
            secondary_color: Color::Magenta,
            previous_lines: 0,
            next_lines: 0,
            middle_lines: 0,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the actual code the block will use.
    #[inline(always)]
    fn max_line_to_print(&self) -> usize {
        self.sections
            .last()
            .map(|v| v.end.line.saturating_add(self.next_lines))
            .unwrap_or(1)
    }

    /// Returns the actual code the block will use.
    #[inline(always)]
    pub fn get_content(&self) -> &str {
        &self.code
    }

    /// Returns the sections.
    #[inline(always)]
    pub fn get_sections(&self) -> &[CodeSection<'a>] {
        &self.sections
    }

    /// Returns the title.
    #[inline(always)]
    pub fn get_title(&self) -> &TextBlock<'a> {
        &self.title
    }

    /// Returns a mutable reference to the title.
    #[inline(always)]
    pub fn get_title_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.title
    }

    /// Returns the file path.
    #[inline(always)]
    pub fn get_file_path(&self) -> &TextBlock<'a> {
        &self.file_path
    }

    /// Returns a mutable reference to the file path.
    #[inline(always)]
    pub fn get_file_path_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.file_path
    }

    /// Returns the final message.
    #[inline(always)]
    pub fn get_final_message(&self) -> &TextBlock<'a> {
        &self.final_message
    }

    /// Returns a mutable reference to the final message.
    #[inline(always)]
    pub fn get_final_message_mut(&mut self) -> &mut TextBlock<'a> {
        &mut self.final_message
    }

    /// Returns whether to show new line chars '\n' as '↩' or not.
    #[inline(always)]
    pub fn get_show_new_line_chars(&self) -> bool {
        self.show_new_line_chars
    }

    /// Returns the secondary color to highlight blocks.
    #[inline(always)]
    pub fn get_secondary_color(&self) -> Color {
        self.secondary_color
    }

    /// Returns the number of lines to show before all sections.
    #[inline(always)]
    pub fn get_previous_lines(&self) -> usize {
        self.previous_lines
    }

    /// Returns the number of lines to show after all sections.
    #[inline(always)]
    pub fn get_next_lines(&self) -> usize {
        self.next_lines
    }

    /// Returns the number of lines to show in the middle of two sections.
    #[inline(always)]
    pub fn get_middle_lines(&self) -> usize {
        self.middle_lines
    }

    // SETTERS ----------------------------------------------------------------

    /// Sets the title.
    #[inline(always)]
    pub fn title(mut self, title: impl Into<TextBlock<'a>>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the file path.
    #[inline(always)]
    pub fn file_path(mut self, file_path: impl Into<TextBlock<'a>>) -> Self {
        self.file_path = file_path.into();
        self
    }

    /// Sets the final message.
    #[inline(always)]
    pub fn final_message(mut self, final_message: impl Into<TextBlock<'a>>) -> Self {
        self.final_message = final_message.into();
        self
    }

    /// Sets whether to show new line chars '\n' as '↩' or not.
    #[inline(always)]
    pub fn show_new_line_chars(mut self, show_new_line_chars: bool) -> Self {
        self.show_new_line_chars = show_new_line_chars;
        self
    }

    /// Sets the secondary color to highlight blocks.
    #[inline(always)]
    pub fn secondary_color(mut self, secondary_color: Color) -> Self {
        self.secondary_color = secondary_color;
        self
    }

    /// Sets the number of lines to show before all sections.
    #[inline(always)]
    pub fn previous_lines(mut self, previous_lines: usize) -> Self {
        self.previous_lines = previous_lines;
        self
    }

    /// Sets the number of lines to show after all sections.
    #[inline(always)]
    pub fn next_lines(mut self, next_lines: usize) -> Self {
        self.next_lines = next_lines;
        self
    }

    /// Sets the number of lines to show in the middle of two sections.
    #[inline(always)]
    pub fn middle_lines(mut self, middle_lines: usize) -> Self {
        self.middle_lines = middle_lines;
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Highlights a cursor adding a colored dot at its position.
    ///
    /// # Panics
    /// This method panics if the section collides with another section or if the indexes are out of bounds.
    #[inline(always)]
    pub fn highlight_cursor(self, position: usize, color: Option<Color>) -> Self {
        self.highlight_section_inner(position..position, None, color)
    }

    /// Highlights a cursor adding a colored dot at its position and including a message.
    ///
    /// # Panics
    /// This method panics if the section collides with another section or if the indexes are out of bounds.
    #[inline(always)]
    pub fn highlight_cursor_message(
        self,
        position: usize,
        color: Option<Color>,
        message: impl Into<TextBlock<'a>>,
    ) -> Self {
        self.highlight_section_inner(position..position, Some(message.into()), color)
    }

    /// Highlights a code section coloring the text.
    ///
    /// # Panics
    /// This method panics if the section collides with another section or if the indexes are out of bounds.
    pub fn highlight_section(self, range: Range<usize>, color: Option<Color>) -> Self {
        assert!(
            range.start <= range.end,
            "The start index must be less or equal than the end index"
        );

        self.highlight_section_inner(range, None, color)
    }

    /// Highlights a code section coloring the text and including a message.
    ///
    /// # Panics
    /// This method panics if the section collides with another section or if the indexes are out of bounds.
    pub fn highlight_section_message(
        self,
        range: Range<usize>,
        color: Option<Color>,
        message: impl Into<TextBlock<'a>>,
    ) -> Self {
        assert!(
            range.start <= range.end,
            "The start index must be less or equal than the end index"
        );

        self.highlight_section_inner(range, Some(message.into()), color)
    }

    /// Highlights a section.
    ///
    /// # Panics
    /// This method panics if the section collides with another section or if the indexes are out of bounds.
    fn highlight_section_inner(
        mut self,
        range: Range<usize>,
        message: Option<TextBlock<'a>>,
        color: Option<Color>,
    ) -> Self {
        assert!(
            range.end <= self.code.len(),
            "The end index must be less or equal than the code length"
        );

        let index = self
            .sections
            .binary_search_by(|section| {
                // Special case to detect the addition of two equal cursors.
                assert!(
                    range.start != section.start.byte_offset
                        || range.end != section.end.byte_offset,
                    "Sections cannot collide with others"
                );

                if range.end <= section.start.byte_offset {
                    std::cmp::Ordering::Greater
                } else if section.end.byte_offset <= range.start {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .expect_err("Sections cannot collide with others");

        let start = if let Some(section) = self.sections.get(index) {
            Cursor::from_byte_offset_and_cursor(&self.code, range.start, &section.start)
        } else {
            Cursor::from_byte_offset(&self.code, range.start)
        };

        if range.is_empty() {
            // Cursor
            self.sections.insert(
                index,
                CodeSection {
                    start,
                    end: start,
                    message: message.unwrap_or_default(),
                    color,
                    is_multiline_start: false,
                    is_multiline_end: false,
                },
            );
        } else {
            let end = Cursor::from_byte_offset_and_cursor(&self.code, range.end, &start);
            let is_multiline = start.line != end.line;

            if is_multiline {
                // When the end cursor is at the start of a line, it means the section finishes at
                // a new line character, therefore we need to add only one section.
                if end.column == 1 {
                    self.sections.insert(
                        index,
                        CodeSection {
                            start,
                            end: start
                                .next_start_line_cursor(&self.code)
                                .unwrap_or_else(|| start.end_line_cursor(&self.code)),
                            message: message.unwrap_or_default(),
                            color,
                            is_multiline_start: false,
                            is_multiline_end: false,
                        },
                    );
                } else {
                    self.sections.splice(
                        index..index,
                        [
                            CodeSection {
                                start,
                                end: start
                                    .next_start_line_cursor(&self.code)
                                    .unwrap_or_else(|| start.end_line_cursor(&self.code)),
                                message: TextBlock::new(),
                                color,
                                is_multiline_start: true,
                                is_multiline_end: false,
                            },
                            CodeSection {
                                start: end.start_line_cursor(&self.code),
                                end,
                                message: message.unwrap_or_default(),
                                color,
                                is_multiline_start: false,
                                is_multiline_end: true,
                            },
                        ],
                    );
                }
            } else {
                self.sections.insert(
                    index,
                    CodeSection {
                        start,
                        end,
                        message: message.unwrap_or_default(),
                        color,
                        is_multiline_start: false,
                        is_multiline_end: false,
                    },
                );
            }
        };
        self
    }

    fn print_with_options(&self, printer: &mut Printer<'a>, max_line_digits: usize) {
        // Title
        let code_indent = TextBlock::new_plain(build_space_string(max_line_digits + 1));

        if !self.title.is_empty() {
            printer.push_styled_text(
                format!(
                    "{:>width$} ",
                    printer.level.symbol(),
                    width = max_line_digits
                ),
                Style::new().bold().fg(printer.level.color()),
            );

            let mut title_printer = printer.derive();

            self.title.print(&mut title_printer);
            title_printer.indent(code_indent.get_sections(), false);
            printer.append(title_printer);
        }

        // First line.
        {
            if self.title.is_empty() {
                printer.push_styled_text(
                    format!(
                        "{:>width$} ",
                        printer.level.symbol(),
                        width = max_line_digits
                    ),
                    Style::new().bold().fg(printer.level.color()),
                );
            } else {
                printer.push_plain_text("\n");
                code_indent.print(printer);
            }

            if self.file_path.is_empty() {
                printer.push_styled_text(
                    Cow::Borrowed(concatcp!(BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR)),
                    Style::new().bold(),
                );
            } else {
                printer.push_styled_text(
                    Cow::Borrowed(concatcp!(BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, '[')),
                    Style::new().bold(),
                );
                self.file_path.single_lined().print(printer);
                printer.push_styled_text(Cow::Borrowed(concatcp!(']')), Style::new().bold());
            }
        }

        // Sections.
        if !self.sections.is_empty() {
            // Show previous lines.
            if self.previous_lines > 0 {
                let first_section_start_cursor = self.sections.first().unwrap().start;
                let start_line = first_section_start_cursor
                    .line
                    .saturating_sub(self.previous_lines)
                    .max(1);
                let mut next_line_start_cursor = first_section_start_cursor
                    .find_line_start(&self.code, start_line)
                    .unwrap();

                for line in start_line..first_section_start_cursor.line {
                    printer.push_styled_text(
                        format!("\n{:>width$} ", line, width = max_line_digits),
                        Style::new().bold().fg(Color::BrightBlack),
                    );
                    printer.push_styled_text(
                        Cow::Borrowed(concatcp!(VERTICAL_BAR, "    ")),
                        Style::new().bold(),
                    );
                    printer.push_plain_text({
                        if self.show_new_line_chars {
                            Cow::Owned(format!(
                                "{}{NEW_LINE_LEFT}",
                                next_line_start_cursor.slice_to_line_end(&self.code)
                            ))
                        } else {
                            match &self.code {
                                Cow::Borrowed(v) => {
                                    Cow::Borrowed(next_line_start_cursor.slice_to_line_end(v))
                                }
                                Cow::Owned(v) => Cow::Owned(
                                    next_line_start_cursor.slice_to_line_end(v).to_string(),
                                ),
                            }
                        }
                    });

                    next_line_start_cursor = next_line_start_cursor
                        .next_start_line_cursor(&self.code)
                        .unwrap();
                }
            }

            // Show highlighted sections.
            {
                let last_line = self.sections.first().unwrap().start.line;
                let mut sections: &[CodeSection] = &self.sections;
                let mut current_line_sections = Vec::new();

                while !sections.is_empty() {
                    group_sections_in_same_line(&mut sections, &mut current_line_sections);

                    let line_start_cursor = current_line_sections
                        .first()
                        .unwrap()
                        .start
                        .start_line_cursor(&self.code);

                    // Print middle lines.
                    let middle_lines = (line_start_cursor.line - last_line).saturating_sub(1);
                    if middle_lines > 1 {
                        if self.middle_lines >= middle_lines {
                            // Print lines.
                            let mut next_line_start_cursor = line_start_cursor
                                .find_line_start(&self.code, last_line)
                                .unwrap();

                            for line in (last_line + 1)..line_start_cursor.line {
                                printer.push_styled_text(
                                    format!("\n{:>width$} ", line, width = max_line_digits),
                                    Style::new().bold().fg(Color::BrightBlack),
                                );
                                printer.push_styled_text(
                                    Cow::Borrowed(concatcp!(VERTICAL_BAR, "    ")),
                                    Style::new().bold(),
                                );
                                printer.push_plain_text({
                                    if self.show_new_line_chars {
                                        Cow::Owned(format!(
                                            "{}{NEW_LINE_LEFT}",
                                            next_line_start_cursor.slice_to_line_end(&self.code)
                                        ))
                                    } else {
                                        match &self.code {
                                            Cow::Borrowed(v) => Cow::Borrowed(
                                                next_line_start_cursor.slice_to_line_end(v),
                                            ),
                                            Cow::Owned(v) => Cow::Owned(
                                                next_line_start_cursor
                                                    .slice_to_line_end(v)
                                                    .to_string(),
                                            ),
                                        }
                                    }
                                });

                                next_line_start_cursor = next_line_start_cursor
                                    .next_start_line_cursor(&self.code)
                                    .unwrap();
                            }
                        } else {
                            // Skip lines.
                            printer.push_styled_text(
                                build_whitespace_string(1, max_line_digits),
                                Style::new(),
                            );
                            printer.push_styled_text(Cow::Borrowed("···    "), Style::new().bold());
                        }
                    }

                    // Print code line.
                    printer.push_styled_text(
                        format!(
                            "\n{:>width$} ",
                            line_start_cursor.line,
                            width = max_line_digits
                        ),
                        Style::new().bold().fg(Color::BrightBlack),
                    );
                    printer.push_styled_text(
                        Cow::Borrowed(concatcp!(VERTICAL_BAR, "    ")),
                        Style::new().bold(),
                    );

                    let mut next_color = self.secondary_color;
                    let mut previous_cursor = line_start_cursor;

                    for section in &current_line_sections {
                        // Print previous content.
                        printer.push_plain_text(match &self.code {
                            Cow::Borrowed(v) => {
                                Cow::Borrowed(previous_cursor.slice(v, &section.start))
                            }
                            Cow::Owned(v) => {
                                Cow::Owned(previous_cursor.slice(v, &section.start).to_string())
                            }
                        });

                        next_color =
                            section
                                .color
                                .unwrap_or(if next_color == self.secondary_color {
                                    printer.level.color()
                                } else {
                                    self.secondary_color
                                });

                        section.print_content(printer, self, next_color);
                        previous_cursor = section.end;
                    }

                    if previous_cursor.line == line_start_cursor.line {
                        let line_end_cursor = previous_cursor.end_line_cursor(&self.code);
                        printer.push_plain_text(match &self.code {
                            Cow::Borrowed(v) => {
                                Cow::Borrowed(previous_cursor.slice(v, &line_end_cursor))
                            }
                            Cow::Owned(v) => {
                                Cow::Owned(previous_cursor.slice(v, &line_end_cursor).to_string())
                            }
                        });

                        if self.show_new_line_chars {
                            printer.push_plain_text(Cow::Borrowed(concatcp!(NEW_LINE_LEFT)));
                        }
                    }

                    // Print underline.
                    printer.push_plain_text(build_whitespace_string(1, max_line_digits + 1));
                    printer.push_styled_text(
                        if current_line_sections.first().unwrap().is_multiline_end {
                            Cow::Borrowed(concatcp!(VERTICAL_BAR, "  "))
                        } else {
                            Cow::Borrowed(concatcp!(VERTICAL_BAR, "    "))
                        },
                        Style::new().bold(),
                    );

                    next_color = self.secondary_color;
                    previous_cursor = line_start_cursor;

                    for section in &current_line_sections {
                        // Print previous content.
                        printer.push_plain_text(build_space_string(
                            section.start.char_offset - previous_cursor.char_offset,
                        ));

                        next_color =
                            section
                                .color
                                .unwrap_or(if next_color == self.secondary_color {
                                    printer.level.color()
                                } else {
                                    self.secondary_color
                                });

                        section.print_underline(printer, self, next_color);
                        previous_cursor = section.end;
                    }

                    // Print message lines.
                    let number_of_messages = current_line_sections
                        .iter()
                        .filter(|v| !v.message.is_empty())
                        .count();

                    for row in 0..number_of_messages {
                        printer.push_plain_text(Cow::Borrowed("\n"));
                        let mut prefix = TextBlock::new()
                            .add_plain_text(build_space_string(max_line_digits + 1))
                            .add_styled_text(
                                Cow::Borrowed(concatcp!(VERTICAL_BAR)),
                                Style::new().bold(),
                            );

                        next_color = self.secondary_color;
                        previous_cursor = line_start_cursor;

                        // TODO: add option to align.

                        let mut space_count = 4;

                        let mut current_message_index = number_of_messages;
                        for section in &current_line_sections {
                            // Add previous content to the space count.
                            space_count += section.start.char_offset - previous_cursor.char_offset;

                            if !section.message.is_empty() {
                                prefix = prefix.add_plain_text(build_space_string(space_count));
                                space_count = 0;
                            }

                            next_color =
                                section
                                    .color
                                    .unwrap_or(if next_color == self.secondary_color {
                                        printer.level.color()
                                    } else {
                                        self.secondary_color
                                    });

                            if section.message.is_empty() {
                                space_count += section.char_len();
                            } else {
                                if row + 1 == current_message_index {
                                    prefix.print(printer);
                                    printer.push_styled_text(
                                        Cow::Borrowed(concatcp!(
                                            TOP_RIGHT_CORNER,
                                            HORIZONTAL_BAR,
                                            HORIZONTAL_BAR,
                                            ' '
                                        )),
                                        Style::new().bold().fg(next_color),
                                    );

                                    prefix = prefix.add_plain_text("    ");

                                    let mut message_printer = printer.derive();
                                    section.message.print(&mut message_printer);
                                    message_printer.indent(prefix.get_sections(), false);
                                    printer.append(message_printer);
                                    break;
                                }

                                prefix = prefix.add_styled_text(
                                    Cow::Borrowed(concatcp!(VERTICAL_BAR)),
                                    Style::new().bold().fg(next_color),
                                );

                                space_count += section.char_len() - 1;
                                current_message_index -= 1;
                            }

                            previous_cursor = section.end;
                        }
                    }
                }
            }

            // Show next lines.
            if self.next_lines > 0 {
                let mut last_section_start_cursor = self.sections.last().unwrap().start;
                let last_line = last_section_start_cursor
                    .line
                    .saturating_add(self.next_lines);

                for line in last_section_start_cursor.line..last_line {
                    let next_line_start_cursor =
                        match last_section_start_cursor.next_start_line_cursor(&self.code) {
                            Some(v) => v,
                            None => break,
                        };

                    printer.push_styled_text(
                        format!("\n{:>width$} ", line + 1, width = max_line_digits),
                        Style::new().bold().fg(Color::BrightBlack),
                    );
                    printer.push_styled_text(
                        Cow::Borrowed(concatcp!(VERTICAL_BAR, "    ")),
                        Style::new().bold(),
                    );
                    printer.push_plain_text({
                        match &self.code {
                            Cow::Borrowed(v) => {
                                if self.show_new_line_chars {
                                    let slice = next_line_start_cursor.slice_to_line_end(v);

                                    if slice.len() + next_line_start_cursor.byte_offset
                                        == self.code.len()
                                    {
                                        Cow::Borrowed(slice)
                                    } else {
                                        Cow::Owned(format!("{}{NEW_LINE_LEFT}", slice))
                                    }
                                } else {
                                    Cow::Borrowed(next_line_start_cursor.slice_to_line_end(v))
                                }
                            }
                            Cow::Owned(v) => {
                                if self.show_new_line_chars {
                                    let slice = next_line_start_cursor.slice_to_line_end(v);

                                    if slice.len() + next_line_start_cursor.byte_offset
                                        == self.code.len()
                                    {
                                        Cow::Owned(slice.to_string())
                                    } else {
                                        Cow::Owned(format!("{}{NEW_LINE_LEFT}", slice))
                                    }
                                } else {
                                    Cow::Owned(
                                        next_line_start_cursor.slice_to_line_end(v).to_string(),
                                    )
                                }
                            }
                        }
                    });

                    last_section_start_cursor = next_line_start_cursor;
                }
            }
        }

        // Final line + message.
        {
            let mut final_line_printer = printer.derive();
            if self.final_message.is_empty() {
                final_line_printer.push_styled_text(
                    Cow::Borrowed(concatcp!(TOP_RIGHT_CORNER, HORIZONTAL_BAR)),
                    Style::new().bold(),
                );
            } else {
                final_line_printer.push_styled_text(
                    Cow::Borrowed(concatcp!(TOP_RIGHT_CORNER, HORIZONTAL_BAR, ' ')),
                    Style::new().bold(),
                );

                let message_indent = TextBlock::new_plain(Cow::Borrowed("   "));
                let mut message_printer = final_line_printer.derive();

                self.final_message.print(&mut message_printer);
                message_printer.indent(message_indent.get_sections(), false);
                final_line_printer.append(message_printer);
            }

            final_line_printer.indent(code_indent.get_sections(), true);
            printer.append_lines(final_line_printer);
        }
    }

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> CodeBlock<'static> {
        CodeBlock {
            code: Cow::Owned(self.code.to_string()),
            sections: self.sections.into_iter().map(|v| v.make_owned()).collect(),
            title: self.title.make_owned(),
            file_path: self.file_path.make_owned(),
            final_message: self.final_message.make_owned(),
            show_new_line_chars: self.show_new_line_chars,
            secondary_color: self.secondary_color,
            previous_lines: self.previous_lines,
            next_lines: self.next_lines,
            middle_lines: self.middle_lines,
        }
    }
}

impl<'a> Printable<'a> for CodeBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        let max_line_digits = format!("{}", self.max_line_to_print()).len();

        self.print_with_options(printer, max_line_digits)
    }
}

impl<'a> Display for CodeBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = Printer::new(LogLevel::trace(), PrinterFormat::Plain);
        self.print(&mut printer);
        printer.fmt(f, PrinterFormat::Plain)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

// This method will panic if sections is empty.
fn group_sections_in_same_line<'s, 'a>(
    sections: &mut &'s [CodeSection<'a>],
    sections_in_same_line: &mut Vec<&'s CodeSection<'a>>,
) {
    sections_in_same_line.clear();

    let line = sections.first().unwrap().start.line;
    sections_in_same_line.extend(sections.iter().take_while(|v| v.start.line == line));
    *sections = &sections[sections_in_same_line.len()..];
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogLevel;

    #[test]
    fn test_plain() {
        let code =
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10";

        // All
        let log = CodeBlock::new(code);
        let text = log
            .title("This is\na title")
            .final_message("This is\na message")
            .file_path("a/b/c")
            // // Line 3
            // .highlight_section(14..15, None)
            // .highlight_cursor(15, None)
            // .highlight_section(15..16, None)
            // .highlight_cursor(16, None)
            // .highlight_section(16..20, None)
            // .highlight_cursor(20, None)
            // .highlight_section(20..21, None)
            // // Line 6
            // .highlight_section(36..41, None)
            // // Line 8
            // .highlight_section(52..58, None)
            // Line 3
            .highlight_section_message(14..15, None, "This is\na message")
            .highlight_cursor_message(15, None, "This is\na message")
            .highlight_section_message(15..16, None, "This is\na message")
            .highlight_cursor_message(16, None, "This is\na message")
            .highlight_section_message(16..20, None, "This is\na message")
            .highlight_cursor_message(20, None, "This is\na message")
            .highlight_section_message(20..21, None, "This is\na message")
            // Line 6
            .highlight_section_message(36..41, None, "This is\na message")
            // Line 8
            .highlight_section_message(52..58, None, "This is\na message")
            // .previous_lines(1)
            // .next_lines(1)
            // .middle_lines(50)
            .show_new_line_chars(true)
            .print_to_string(LogLevel::error(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(text, "╭─▶ This is\n│   a message\n│  [2] /a/b/c(crate::x) - This is a \n│      message\n│  [1]");
    }
}