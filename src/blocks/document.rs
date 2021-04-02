use std::borrow::Cow;
use std::cmp::max;
use std::ops::Range;
use std::option::Option::Some;

use yansi::Color;

use crate::constants::{
    BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, HORIZONTAL_BOTTOM_BAR, HORIZONTAL_TOP_BAR, MIDDLE_DOT,
    NEW_LINE, RIGHT_POINTER, TOP_LEFT_CORNER, TOP_RIGHT_CORNER, UP_POINTER, VERTICAL_BAR,
    VERTICAL_LEFT_BAR, VERTICAL_RIGHT_BAR,
};
use crate::utils::cursor::Cursor;
use crate::utils::text::{color_bold_if, indent_text, remove_ansi_escapes, remove_jump_lines};
use crate::utils::RangeMap;
use crate::Log;

/// A block that prints a section of a document.
#[derive(Debug, Clone)]
pub struct DocumentBlock<'a> {
    content: Cow<'a, str>,
    sections: RangeMap<HighlightedSection<'a>>,
    related_block: Option<Box<DocumentBlock<'a>>>,
    title: Option<Cow<'a, str>>,
    file_path: Option<Cow<'a, str>>,
    end_message: Option<Cow<'a, str>>,
    show_new_line_chars: bool,
    secondary_color: Color,
}

impl<'a> DocumentBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(content: Cow<'a, str>) -> DocumentBlock {
        DocumentBlock {
            content,
            sections: RangeMap::new(),
            related_block: None,
            title: None,
            file_path: None,
            end_message: None,
            show_new_line_chars: false,
            secondary_color: Color::Magenta,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The document content of the block.
    pub fn get_content(&self) -> &Cow<'a, str> {
        &self.content
    }

    /// The related block of this one.
    pub fn get_related_block(&self) -> &Option<Box<DocumentBlock>> {
        &self.related_block
    }

    /// The title message to show at the top of the block.
    pub fn get_title(&self) -> &Option<Cow<'a, str>> {
        &self.title
    }

    /// The file path of the document.
    /// Ignored if the block belongs to another one as a related block.
    pub fn get_file_path(&self) -> &Option<Cow<'a, str>> {
        &self.file_path
    }

    /// The final message to show at the bottom of the block.
    /// Ignored if the block belongs to another one as a related block.
    pub fn get_end_message(&self) -> &Option<Cow<'a, str>> {
        &self.end_message
    }

    /// Whether to show new line chars '\n' as as '↩' or not.
    pub fn get_show_new_line_chars(&self) -> bool {
        self.show_new_line_chars
    }

    /// The secondary color to highlight blocks.
    pub fn get_secondary_color(&self) -> Color {
        self.secondary_color
    }

    // SETTERS ----------------------------------------------------------------

    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn clear_title(mut self) -> Self {
        self.title = None;
        self
    }

    pub fn file_path(mut self, file_path: impl Into<Cow<'a, str>>) -> Self {
        self.file_path = Some(file_path.into());
        self
    }

    pub fn clear_file_path(mut self) -> Self {
        self.file_path = None;
        self
    }

    pub fn end_message(mut self, end_message: impl Into<Cow<'a, str>>) -> Self {
        self.end_message = Some(end_message.into());
        self
    }

    pub fn clear_end_message(mut self) -> Self {
        self.end_message = None;
        self
    }

    pub fn show_new_line_chars(mut self, show_new_line_chars: bool) -> Self {
        self.show_new_line_chars = show_new_line_chars;
        self
    }

    pub fn secondary_color(mut self, secondary_color: Color) -> Self {
        self.secondary_color = secondary_color;
        self
    }

    // METHODS ----------------------------------------------------------------

    /// Highlights a cursor position.
    /// Equals to `highlight_section(position..position, ..)`.
    pub fn highlight_cursor(self, position: usize, color: Option<Color>) -> Self {
        self.highlight_section_inner(position..position, None, color)
    }

    /// Highlights a cursor position.
    /// Equals to `highlight_section(position..position, ..)`.
    pub fn highlight_cursor_message(
        self,
        position: usize,
        message: impl Into<Cow<'a, str>>,
        color: Option<Color>,
    ) -> Self {
        self.highlight_section_inner(position..position, Some(message.into()), color)
    }

    /// Highlights a section.
    pub fn highlight_section(self, range: Range<usize>, color: Option<Color>) -> Self {
        self.highlight_section_inner(range, None, color)
    }

    /// Highlights a section with a message.
    pub fn highlight_section_message(
        self,
        range: Range<usize>,
        message: impl Into<Cow<'a, str>>,
        color: Option<Color>,
    ) -> Self {
        self.highlight_section_inner(range, Some(message.into()), color)
    }

    /// Highlights a section.
    fn highlight_section_inner(
        mut self,
        range: Range<usize>,
        message: Option<Cow<'a, str>>,
        color: Option<Color>,
    ) -> Self {
        assert!(
            self.sections.collides_with(range.clone()),
            "Sections cannot collide with others"
        );

        let from = Cursor::from_byte_offset(&self.content, range.start);

        let section = if range.is_empty() {
            // Cursor
            HighlightedSection {
                to: from.clone(),
                from,
                message,
                color,
                is_multiline_start: false,
                is_multiline_end: false,
                is_cursor: true,
            }
        } else {
            let to = Cursor::from_byte_offset_and_cursor(&self.content, range.end, &from);
            let is_multiline = from.line != to.line;

            HighlightedSection {
                from,
                to,
                message,
                color,
                is_multiline_start: is_multiline,
                is_multiline_end: is_multiline,
                is_cursor: false,
            }
        };

        self.sections.insert(range, section);
        self
    }

    /// Sets a related block to the block.
    pub fn related_document<F>(mut self, builder: F) -> Self
    where
        F: FnOnce(DocumentBlock) -> DocumentBlock,
    {
        let mut document = DocumentBlock::new(self.content.clone());
        document.show_new_line_chars = self.show_new_line_chars;

        let document = builder(document);
        self.related_block = Some(Box::new(document));
        self
    }

    /// Clears the related block of the block.
    pub fn clear_related_block(mut self) -> Self {
        self.related_block = None;
        self
    }

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        self.to_text_with_options(log, in_ansi, buffer, false)
    }

    fn to_text_with_options(
        &self,
        log: &Log<'a>,
        in_ansi: bool,
        buffer: &mut String,
        is_related_block: bool,
    ) {
        // FIRST LINE + TITLE
        if is_related_block {
            buffer.push_str(&color_bold_if(
                format!("{}{}", VERTICAL_RIGHT_BAR, HORIZONTAL_BAR),
                log.level().color(),
                in_ansi,
            ));

            if let Some(title) = &self.title {
                let indent = format!(
                    "{}  ",
                    &color_bold_if(VERTICAL_BAR.to_string(), log.level().color(), in_ansi)
                );
                let title = indent_text(&title, &indent, false);
                buffer.push(' ');
                buffer.push_str(title.as_str());
            }
        } else {
            if self.file_path.is_some() {
                buffer.push_str(&color_bold_if(
                    format!(
                        "{}{}{}",
                        BOTTOM_RIGHT_CORNER, HORIZONTAL_BOTTOM_BAR, HORIZONTAL_BAR
                    ),
                    log.level().color(),
                    in_ansi,
                ));
            } else {
                buffer.push_str(&color_bold_if(
                    format!("{}{}", BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR),
                    log.level().color(),
                    in_ansi,
                ));
            }

            if let Some(title) = &self.title {
                let title = if self.file_path.is_some() {
                    indent_text(
                        &title,
                        &color_bold_if(
                            format!("{}{}  ", VERTICAL_BAR, VERTICAL_BAR),
                            log.level().color(),
                            in_ansi,
                        ),
                        false,
                    )
                } else {
                    indent_text(
                        &title,
                        &color_bold_if(format!("{}  ", VERTICAL_BAR), log.level().color(), in_ansi),
                        false,
                    )
                };
                buffer.push(' ');
                buffer.push_str(title.as_str());
            }

            // FILE
            if let Some(file_path) = &self.file_path {
                let file_path = remove_jump_lines(file_path.as_ref());

                buffer.push('\n');
                buffer.push_str(&color_bold_if(
                    format!("{}{}{}", VERTICAL_BAR, TOP_RIGHT_CORNER, RIGHT_POINTER),
                    log.level().color(),
                    in_ansi,
                ));
                buffer.push(' ');
                buffer.push_str(&color_bold_if(
                    "at".to_string(),
                    log.level().color(),
                    in_ansi,
                ));
                buffer.push(' ');
                buffer.push_str(file_path.as_str());
            }
        }

        // SECTIONS
        if !self.sections.is_empty() {
            // Normalize sections.
            let mut normalized_sections = self.normalize_sections(self, log);
            let last_line = normalized_sections.last().unwrap().to.line;
            let last_content_line = Cursor::from_byte_offset_and_cursor(
                &self.content,
                self.content.len(),
                &normalized_sections.last().unwrap().to,
            )
            .line;
            let max_line_num_digits = last_line.to_string().len();

            // EMPTY LINE
            // Only if file path is present or the title is multiline.
            if self.file_path.is_some()
                || self
                    .title
                    .as_ref()
                    .map_or(false, |v| memchr::memchr(b'\n', v.as_bytes()).is_some())
            {
                buffer.push('\n');
                buffer.push_str(&color_bold_if(
                    VERTICAL_BAR.to_string(),
                    log.level().color(),
                    in_ansi,
                ));
            }

            // CONTENT
            let mut sections_in_same_line = Vec::new();
            while !normalized_sections.is_empty() {
                // Filter only those sections that are in the same content line.
                sections_in_same_line.clear();
                self.get_sections_in_same_line(
                    &mut normalized_sections,
                    &mut sections_in_same_line,
                );

                // Get column to align messages.
                let first_section = sections_in_same_line.first().unwrap();
                let last_section = sections_in_same_line.last().unwrap();
                let number_of_cursors =
                    sections_in_same_line.iter().filter(|s| s.is_cursor).count();
                let message_column = last_section.to.column + number_of_cursors + 1;

                // CONTENT LINE
                buffer.push('\n');
                buffer.push_str(&color_bold_if(
                    VERTICAL_BAR.to_string(),
                    log.level().color(),
                    in_ansi,
                ));
                buffer.push_str("   ");
                buffer.push_str(&color_bold_if(
                    format!(
                        "{:>width$}",
                        last_section.from.line,
                        width = max_line_num_digits
                    ),
                    log.level().color(),
                    in_ansi,
                ));
                buffer.push_str("  ");

                // SECTIONS
                let start_line_cursor = first_section.from.start_line_cursor(&self.content);
                let mut prev_cursor = start_line_cursor.clone();
                for section in &sections_in_same_line {
                    // PREVIOUS CONTENT
                    buffer.push_str(prev_cursor.slice(&self.content, &section.from));

                    // CONTENT
                    section.print_content_section(self, in_ansi, buffer);

                    prev_cursor = section.to.clone();
                }

                // FINAL CONTENT
                buffer.push_str(prev_cursor.slice_to_line_end(&self.content));

                if self.show_new_line_chars && last_section.to.line != last_content_line {
                    buffer.push_str(&color_bold_if(
                        NEW_LINE.to_string(),
                        last_section.color.unwrap(),
                        in_ansi
                            && (last_section.is_multiline_start
                                || last_section.is_ended_by_new_line(&self.content)),
                    ));
                }

                // ARROW LINES
                // Count lines with messages.
                let arrow_lines_height = max(
                    sections_in_same_line
                        .iter()
                        .filter(|v| v.message.is_some() || v.is_multiline_start)
                        .count(),
                    1, /* This is because there is always one line */
                );

                // Print lines.
                let digits_as_whitespace = " ".repeat(max_line_num_digits);

                for arrow_line in 0..arrow_lines_height {
                    let mut arrow_lines_height = arrow_lines_height;

                    buffer.push('\n');
                    buffer.push_str(&color_bold_if(
                        VERTICAL_BAR.to_string(),
                        log.level().color(),
                        in_ansi,
                    ));
                    buffer.push_str("   ");
                    buffer.push_str(&digits_as_whitespace);
                    buffer.push_str("  ");

                    // SECTIONS
                    let mut prev_cursor = start_line_cursor.clone();
                    for section in &sections_in_same_line {
                        // PREVIOUS CONTENT
                        buffer.push_str(
                            &" ".repeat(section.from.char_offset - prev_cursor.char_offset),
                        );

                        // CONTENT
                        let has_printed_message = section.print(
                            buffer,
                            in_ansi,
                            arrow_line,
                            arrow_lines_height,
                            message_column,
                            max_line_num_digits,
                        );

                        if has_printed_message {
                            break;
                        }

                        if section.message.is_some() {
                            arrow_lines_height -= 1;
                        }

                        prev_cursor = section.to.clone();
                    }
                }
            }
        }

        // RELATED BLOCK
        if let Some(related_block) = &self.related_block {
            buffer.push('\n');
            related_block.to_text_with_options(log, in_ansi, buffer, true);
        }

        // FINAL LINE + END MESSAGE
        if !is_related_block {
            buffer.push('\n');
            buffer.push_str(&color_bold_if(
                format!("{}{}", TOP_RIGHT_CORNER, HORIZONTAL_BAR),
                log.level().color(),
                in_ansi,
            ));

            if let Some(end_message) = &self.end_message {
                buffer.push(' ');
                buffer.push_str(indent_text(end_message, "   ", false).as_str());
            }
        }
    }

    fn normalize_sections(
        &self,
        document: &DocumentBlock<'a>,
        log: &Log<'a>,
    ) -> Vec<HighlightedSection> {
        let mut color = log.level().color();
        self.sections
            .iter()
            .map(|(_, s)| {
                let mut s = s.clone();

                // Normalize colors.
                let log_color = log.level().color();

                if let Some(s_color) = &s.color {
                    if s_color == &log_color {
                        color = document.secondary_color;
                    } else {
                        color = log_color;
                    }
                } else {
                    s.color = Some(color);

                    if color == log_color {
                        color = document.secondary_color;
                    } else {
                        color = log_color;
                    }
                }

                s
            })
            .flat_map(|s| {
                // To normalizing multiline sections we must split them into two different sections.
                if s.is_multiline_start {
                    if s.content(&self.content).ends_with('\n') {
                        let from = s.from.clone();
                        let to = Cursor::from_byte_offset_and_cursor(
                            &self.content,
                            s.to.byte_offset - 1,
                            &s.to,
                        );

                        if to.line == from.line {
                            vec![HighlightedSection {
                                from,
                                to,
                                message: s.message.clone(),
                                color: s.color,
                                is_multiline_start: false,
                                is_multiline_end: false,
                                is_cursor: false,
                            }]
                        } else {
                            vec![
                                HighlightedSection {
                                    to: from.end_line_cursor(&self.content),
                                    from,
                                    message: None,
                                    color: s.color,
                                    is_multiline_start: true,
                                    is_multiline_end: false,
                                    is_cursor: false,
                                },
                                HighlightedSection {
                                    from: to.start_line_cursor(&self.content),
                                    to,
                                    message: s.message.clone(),
                                    color: s.color,
                                    is_multiline_start: false,
                                    is_multiline_end: true,
                                    is_cursor: false,
                                },
                            ]
                        }
                    } else {
                        vec![
                            HighlightedSection {
                                from: s.from.clone(),
                                to: s.from.end_line_cursor(&self.content),
                                message: None,
                                color: s.color,
                                is_multiline_start: true,
                                is_multiline_end: false,
                                is_cursor: false,
                            },
                            HighlightedSection {
                                from: s.to.start_line_cursor(&self.content),
                                to: s.to.clone(),
                                message: s.message.clone(),
                                color: s.color,
                                is_multiline_start: false,
                                is_multiline_end: true,
                                is_cursor: false,
                            },
                        ]
                    }
                } else {
                    // Single line sections are kept the same.
                    vec![s]
                }
            })
            .collect()
    }

    // STATIC METHODS ---------------------------------------------------------

    fn get_sections_in_same_line(
        &self,
        sections: &mut Vec<HighlightedSection<'a>>,
        sections_in_same_line: &mut Vec<HighlightedSection<'a>>,
    ) {
        let section = sections.remove(0);
        let line = section.from.line;

        sections_in_same_line.push(section);

        while !sections.is_empty() {
            let section = &sections[0];

            if section.from.line != line {
                break;
            }

            let section = sections.remove(0);
            sections_in_same_line.push(section);
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Eq, PartialEq)]
struct HighlightedSection<'a> {
    // Inclusive range from..=to.
    from: Cursor,
    to: Cursor,
    message: Option<Cow<'a, str>>,
    color: Option<Color>,
    is_multiline_start: bool,
    is_multiline_end: bool,
    is_cursor: bool,
}

impl<'a> HighlightedSection<'a> {
    // GETTERS ----------------------------------------------------------------

    pub fn char_len(&self) -> usize {
        self.to.char_offset - self.from.char_offset
    }

    pub fn content(&self, text: &'a str) -> &'a str {
        self.from.slice(text, &self.to)
    }

    pub fn is_ended_by_new_line(&self, text: &str) -> bool {
        let text = &text[self.to.byte_offset..];

        match text.chars().next() {
            Some(v) => v == '\n',
            None => false,
        }
    }

    // METHODS ----------------------------------------------------------------

    fn print_content_section(&self, document: &DocumentBlock, in_ansi: bool, buffer: &mut String) {
        if self.is_cursor {
            buffer.push_str(&color_bold_if(
                MIDDLE_DOT.to_string(),
                self.color.unwrap(),
                in_ansi,
            ));
        } else {
            let content = self.content(&document.content);

            buffer.push_str(&color_bold_if(
                content.replace("\t", " ").replace("\r", " "),
                self.color.unwrap(),
                in_ansi,
            ));
        }
    }

    fn print(
        &self,
        buffer: &mut String,
        in_ansi: bool,
        arrow_line: usize,
        arrow_lines_height: usize,
        message_column: usize,
        max_line_num_digits: usize,
    ) -> bool {
        if arrow_line == 0 {
            self.print_first_arrow_line(
                buffer,
                in_ansi,
                message_column,
                arrow_lines_height == 1,
                max_line_num_digits,
            );
            arrow_lines_height == 1 && self.message.is_some()
        } else if self.message.is_some() && arrow_line + 1 == arrow_lines_height {
            self.print_last_arrow_line(buffer, in_ansi, message_column, max_line_num_digits);
            true
        } else {
            self.print_middle_arrow_line(buffer, in_ansi);
            false
        }
    }

    fn print_first_arrow_line(
        &self,
        buffer: &mut String,
        in_ansi: bool,
        message_column: usize,
        print_message: bool,
        max_line_num_digits: usize,
    ) {
        let char_length = self.char_len();

        if self.is_multiline_start {
            buffer.push_str(&color_bold_if(
                format!(
                    "{}{}{}",
                    TOP_RIGHT_CORNER,
                    HORIZONTAL_BAR.repeat(char_length),
                    RIGHT_POINTER
                ),
                self.color.unwrap(),
                in_ansi,
            ));
        } else if self.is_multiline_end {
            buffer.pop().unwrap();

            if self.message.is_some() {
                if print_message {
                    // With message at first line.
                    if char_length > 0 {
                        buffer.push_str(&color_bold_if(
                            format!(
                                "{}{}{}",
                                RIGHT_POINTER,
                                HORIZONTAL_BAR.repeat(char_length - 1),
                                HORIZONTAL_TOP_BAR
                            ),
                            self.color.unwrap(),
                            in_ansi,
                        ));
                    } else {
                        buffer.push_str(&color_bold_if(
                            format!("{}{}", RIGHT_POINTER, HORIZONTAL_TOP_BAR),
                            self.color.unwrap(),
                            in_ansi,
                        ));
                    }

                    self.print_message(buffer, in_ansi, message_column, max_line_num_digits);
                } else {
                    // With message at other line.
                    if char_length <= 1 {
                        buffer.push_str(&color_bold_if(
                            format!("{}{}", RIGHT_POINTER, VERTICAL_LEFT_BAR),
                            self.color.unwrap(),
                            in_ansi,
                        ));
                    } else {
                        buffer.push_str(&color_bold_if(
                            format!(
                                "{}{}{}{}",
                                RIGHT_POINTER,
                                HORIZONTAL_BOTTOM_BAR,
                                HORIZONTAL_BAR.repeat(char_length - 2),
                                TOP_LEFT_CORNER
                            ),
                            self.color.unwrap(),
                            in_ansi,
                        ));
                    }
                }
            } else {
                // No message.
                if char_length > 0 {
                    buffer.push_str(&color_bold_if(
                        format!(
                            "{}{}{}",
                            RIGHT_POINTER,
                            HORIZONTAL_BAR.repeat(char_length - 1),
                            TOP_LEFT_CORNER
                        ),
                        self.color.unwrap(),
                        in_ansi,
                    ));
                } else {
                    buffer.push_str(&color_bold_if(
                        format!("{}{}", RIGHT_POINTER, TOP_LEFT_CORNER),
                        self.color.unwrap(),
                        in_ansi,
                    ));
                }
            }
        } else {
            match char_length {
                0 | 1 => {
                    if self.message.is_some() {
                        if print_message {
                            // With message at first line.
                            buffer.push_str(&color_bold_if(
                                TOP_RIGHT_CORNER.to_string(),
                                self.color.unwrap(),
                                in_ansi,
                            ));
                            self.print_message(
                                buffer,
                                in_ansi,
                                message_column,
                                max_line_num_digits,
                            );
                        } else {
                            // With message at other line.
                            buffer.push_str(&color_bold_if(
                                VERTICAL_BAR.to_string(),
                                self.color.unwrap(),
                                in_ansi,
                            ));
                        }
                    } else {
                        // No message.
                        buffer.push_str(&color_bold_if(
                            UP_POINTER.to_string(),
                            self.color.unwrap(),
                            in_ansi,
                        ));
                    }
                }
                _ => {
                    if self.message.is_some() {
                        if print_message {
                            // With message at first line.
                            buffer.push_str(&color_bold_if(
                                format!(
                                    "{}{}{}",
                                    TOP_RIGHT_CORNER,
                                    HORIZONTAL_BAR.repeat(char_length - 2),
                                    HORIZONTAL_TOP_BAR
                                ),
                                self.color.unwrap(),
                                in_ansi,
                            ));
                            self.print_message(
                                buffer,
                                in_ansi,
                                message_column,
                                max_line_num_digits,
                            );
                        } else {
                            // With message at other line.
                            buffer.push_str(&color_bold_if(
                                format!(
                                    "{}{}{}",
                                    VERTICAL_RIGHT_BAR,
                                    HORIZONTAL_BAR.repeat(char_length - 2),
                                    TOP_LEFT_CORNER
                                ),
                                self.color.unwrap(),
                                in_ansi,
                            ));
                        }
                    } else {
                        // No message.
                        buffer.push_str(&color_bold_if(
                            format!(
                                "{}{}{}",
                                TOP_RIGHT_CORNER,
                                HORIZONTAL_BAR.repeat(char_length - 2),
                                TOP_LEFT_CORNER
                            ),
                            self.color.unwrap(),
                            in_ansi,
                        ));
                    }
                }
            }
        }
    }

    fn print_middle_arrow_line(&self, buffer: &mut String, in_ansi: bool) {
        let char_length = self.char_len();

        match char_length {
            0 | 1 => {
                if self.message.is_some() {
                    buffer.push_str(&color_bold_if(
                        VERTICAL_BAR.to_string(),
                        self.color.unwrap(),
                        in_ansi,
                    ));
                } else {
                    buffer.push(' ');
                }
            }
            _ => {
                if self.message.is_some() {
                    buffer.push_str(&color_bold_if(
                        VERTICAL_BAR.to_string(),
                        self.color.unwrap(),
                        in_ansi,
                    ));
                    buffer.push_str(&" ".repeat(char_length - 1));
                } else {
                    buffer.push_str(&" ".repeat(char_length));
                }
            }
        }
    }

    fn print_last_arrow_line(
        &self,
        buffer: &mut String,
        in_ansi: bool,
        message_column: usize,
        max_line_num_digits: usize,
    ) {
        buffer.push_str(&color_bold_if(
            TOP_RIGHT_CORNER.to_string(),
            self.color.unwrap(),
            in_ansi,
        ));
        self.print_message(buffer, in_ansi, message_column, max_line_num_digits);
    }

    fn print_message(
        &self,
        buffer: &mut String,
        in_ansi: bool,
        message_column: usize,
        max_line_num_digits: usize,
    ) {
        let message = self
            .message
            .as_ref()
            .expect("Cannot call print_last_arrow_line without a message");

        let line_start_offset = match memchr::memrchr(b'\n', buffer.as_bytes()) {
            Some(v) => v + 1,
            None => 0,
        };
        let line_content = &buffer[line_start_offset..];
        let mut num_chars = if in_ansi {
            bytecount::num_chars(remove_ansi_escapes(line_content).as_bytes())
        } else {
            bytecount::num_chars(line_content.as_bytes())
        };

        // Remove constant part.
        num_chars -= 6 + max_line_num_digits;

        let bars = HORIZONTAL_BAR.repeat(message_column - num_chars);

        buffer.push_str(&color_bold_if(bars, self.color.unwrap(), in_ansi));
        buffer.push(' ');
        buffer.push_str(&remove_jump_lines(message));
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

    // This test covers every branch in the code.
    #[test]
    fn test_plain() {
        let text = "First\ntest\nthird\nline";
        let log = Log::info().document(text, |document| {
            document
                .show_new_line_chars(true)
                .title("This\nis a\ntitle")
                .file_path("/path/t\no/file.test")
                .end_message("This\nis an\nend message")
                .highlight_section_message(1..3, "Comment\nmultiline", None)
                .highlight_cursor_message(3, "Comment cursor", None)
                .highlight_section_message(3..4, "Comment", Some(Color::Red))
                .highlight_section_message(5..7, "Comment jump line", Some(Color::Red))
                .highlight_section_message(7..8, "A", Some(Color::Red))
                .highlight_section_message(8..20, "B", Some(Color::Red))
                .highlight_section_message(20..21, "C", Some(Color::Red))
        });
        let text = log.to_plain_text();

        assert_eq!(
            &text,
            "┌┬─ This\n\
             ││  is a\n\
             ││  title\n\
             │└> at /path/t o/file.test\n\
             │\n\
             │   1  Fir·st↩\n\
             │       ├┘││ └>\n\
             │       │ │└─── Comment\n\
             │       │ └──── Comment cursor\n\
             │       └────── Comment multiline\n\
             │   2  test↩\n\
             │     >┤│└──>\n\
             │      │└──── A\n\
             │      └───── Comment jump line\n\
             │   4  line\n\
             │     >┬─┘└── C\n\
             │      └───── B\n\
             └─ This\n   is an\n   end message",
            "Test 0"
        );

        let text = "Second\ntest";
        let log = Log::info().document(text, |document| {
            document
                .show_new_line_chars(true)
                .highlight_section(1..2, None)
                .highlight_section_message(3..5, "1+ chars with message", None)
                .highlight_section(8..10, None)
        });
        let text = log.to_plain_text();

        assert_eq!(
            &text,
            "┌─\n\
             │   1  Second↩\n\
             │       ^ └┴── 1+ chars with message\n\
             │   2  test\n\
             │       └┘\n\
             └─",
            "Test 1"
        );

        let text = "This\r\nis a\ndocument";
        let log = Log::info().document(text, |document| {
            document
                .file_path("/path/t\no/file.test")
                .highlight_section(0..text.len(), None)
                .related_document(|document| {
                    document
                        .highlight_section_message(0..5, "a", None)
                        .highlight_section_message(5..6, "b", None)
                        .highlight_section_message(6..7, "c", None)
                })
        });
        let text = log.to_plain_text();

        assert_eq!(
            &text,
            "┌┬─\n\
             │└> at /path/t o/file.test\n\
             │\n\
             │   1  This \n\
             │      └─────>\n\
             │   3  document\n\
             │     >───────┘\n\
             ├─\n\
             │   1  This \n\
             │      ├───┘└─ b\n\
             │      └────── a\n\
             │   2  is a\n\
             │      └── c\n\
             └─",
            "Test 2"
        );

        let text = "This\nis a\ndocument";
        let log = Log::info().document(text, |document| {
            document
                .file_path("/path/to/file.test")
                .highlight_cursor_message(0, "x0", None)
                .highlight_cursor(1, None)
                .highlight_cursor_message(2, "x2", None)
                .highlight_cursor(3, None)
                .highlight_cursor_message(4, "x4", None)
                .highlight_cursor_message(5, "x5", None)
                .highlight_cursor(6, None)
                .highlight_cursor_message(7, "x7", None)
        });
        let text = log.to_plain_text();

        assert_eq!(
            &text,
            "┌┬─\n\
             │└> at /path/to/file.test\n\
             │\n\
             │   1  ·T·h·i·s·\n\
             │      │ ^ │ ^ └── x4\n\
             │      │   └────── x2\n\
             │      └────────── x0\n\
             │   2  ·i·s· a\n\
             │      │ ^ └── x7\n\
             │      └────── x5\n\
             └─",
            "Test 3"
        );
    }

    #[test]
    fn test_ansi() {
        let text = "First\ntest\nthird\nline";
        let log = Log::info().document(text, |document| {
            document
                .show_new_line_chars(true)
                .title("This\nis a\ntitle")
                .file_path("/path/t\no/file.test")
                .end_message("This\nis an\nend message")
                .highlight_section_message(1..3, "Comment\nmultiline", None)
                .highlight_cursor_message(3, "Comment cursor", None)
                .highlight_section_message(3..4, "Comment", Some(Color::Red))
                .highlight_section_message(5..7, "Comment jump line", Some(Color::Red))
                .highlight_section_message(7..8, "A", Some(Color::Red))
                .highlight_section_message(8..20, "B", None)
                .highlight_section_message(20..21, "C", None)
        });
        let text = log.to_ansi_text();

        let start_path_title = Style::new(LogLevel::info().color()).bold().paint("┌┬─");
        let title_prefix = Style::new(LogLevel::info().color()).bold().paint("││  ");
        let end = Style::new(LogLevel::info().color()).bold().paint("└─");
        let path = Style::new(LogLevel::info().color()).bold().paint("│└>");
        let path_at = Style::new(LogLevel::info().color()).bold().paint("at");
        let single_bar = Style::new(LogLevel::info().color())
            .bold()
            .paint(VERTICAL_BAR);
        assert_eq!(
            text,
            format!(
                "{} This\n\
                 {}is a\n\
                 {}title\n\
                 {} {} /path/t o/file.test\n\
                 {}\n\
                 {}   {}  F{}{}{}t{}{}\n\
                 {}       {}{}{} {}\n\
                 {}       {} {}{}{} Comment\n\
                 {}       {} {}{} Comment cursor\n\
                 {}       {}{} Comment multiline\n\
                 {}   {}  {}{}{}{}\n\
                 {}     {}{}{}\n\
                 {}      {}{}{} A\n\
                 {}      {}{} Comment jump line\n\
                 {}   {}  {}{}\n\
                 {}     {}{}{} C\n\
                 {}      {}{} B\n\
                 {} This\n   is an\n   end message",
                start_path_title,
                title_prefix,
                title_prefix,
                path,
                path_at,
                single_bar,
                single_bar,
                Style::new(Color::Blue).bold().paint("1"),
                Style::new(Color::Blue).bold().paint("ir"),
                Style::new(Color::Magenta).bold().paint(MIDDLE_DOT),
                Style::new(Color::Red).bold().paint("s"),
                Style::new(Color::Red).bold().paint(""),
                Style::new(Color::Red).bold().paint(NEW_LINE),
                single_bar,
                Style::new(Color::Blue).bold().paint("├┘"),
                Style::new(Color::Magenta).bold().paint("│"),
                Style::new(Color::Red).bold().paint("│"),
                Style::new(Color::Red).bold().paint("└>"),
                single_bar,
                Style::new(Color::Blue).bold().paint("│"),
                Style::new(Color::Magenta).bold().paint("│"),
                Style::new(Color::Red).bold().paint("└"),
                Style::new(Color::Red).bold().paint("───"),
                single_bar,
                Style::new(Color::Blue).bold().paint("│"),
                Style::new(Color::Magenta).bold().paint("└"),
                Style::new(Color::Magenta).bold().paint("────"),
                single_bar,
                Style::new(Color::Blue).bold().paint("└"),
                Style::new(Color::Blue).bold().paint("──────"),
                single_bar,
                Style::new(Color::Blue).bold().paint("2"),
                Style::new(Color::Red).bold().paint("t"),
                Style::new(Color::Red).bold().paint("e"),
                Style::new(Color::Blue).bold().paint("st"),
                Style::new(Color::Blue).bold().paint(NEW_LINE),
                single_bar,
                Style::new(Color::Red).bold().paint(">┤"),
                Style::new(Color::Red).bold().paint("│"),
                Style::new(Color::Blue).bold().paint("└──>"),
                single_bar,
                Style::new(Color::Red).bold().paint("│"),
                Style::new(Color::Red).bold().paint("└"),
                Style::new(Color::Red).bold().paint("────"),
                single_bar,
                Style::new(Color::Red).bold().paint("└"),
                Style::new(Color::Red).bold().paint("─────"),
                single_bar,
                Style::new(Color::Blue).bold().paint("4"),
                Style::new(Color::Blue).bold().paint("lin"),
                Style::new(Color::Magenta).bold().paint("e"),
                single_bar,
                Style::new(Color::Blue).bold().paint(">┬─┘"),
                Style::new(Color::Magenta).bold().paint("└"),
                Style::new(Color::Magenta).bold().paint("──"),
                single_bar,
                Style::new(Color::Blue).bold().paint("└"),
                Style::new(Color::Blue).bold().paint("─────"),
                end
            ),
            "Test 0"
        );

        let text = "Second\ntest";
        let log = Log::info().document(text, |document| {
            document
                .show_new_line_chars(true)
                .highlight_section(1..2, None)
                .highlight_section_message(3..5, "1+ chars with message", None)
                .highlight_section(8..10, None)
        });
        let text = log.to_ansi_text();

        let start = Style::new(LogLevel::info().color()).bold().paint("┌─");
        assert_eq!(
            text,
            format!(
                "{}\n\
                 {}   {}  S{}c{}d↩\n\
                 {}       {} {}{} 1+ chars with message\n\
                 {}   {}  t{}t\n\
                 {}       {}\n\
                 {}",
                start,
                single_bar,
                Style::new(Color::Blue).bold().paint("1"),
                Style::new(Color::Blue).bold().paint("e"),
                Style::new(Color::Magenta).bold().paint("on"),
                single_bar,
                Style::new(Color::Blue).bold().paint("^"),
                Style::new(Color::Magenta).bold().paint("└┴"),
                Style::new(Color::Magenta).bold().paint("──"),
                single_bar,
                Style::new(Color::Blue).bold().paint("2"),
                Style::new(Color::Blue).bold().paint("es"),
                single_bar,
                Style::new(Color::Blue).bold().paint("└┘"),
                end
            ),
            "Test 1"
        );

        let text = "This\r\nis a\ndocument";
        let log = Log::info().document(text, |document| {
            document
                .file_path("/path/t\no/file.test")
                .highlight_section(0..text.len(), None)
                .related_document(|document| {
                    document
                        .highlight_section_message(0..5, "a", None)
                        .highlight_section_message(5..6, "b", None)
                        .highlight_section_message(6..7, "c", None)
                })
        });
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{}\n\
                 {} {} /path/t o/file.test\n\
                 {}\n\
                 {}   {}  {}\n\
                 {}      {}\n\
                 {}   {}  {}\n\
                 {}     {}\n\
                 {}\n\
                 {}   {}  {}{}\n\
                 {}      {}{}{} b\n\
                 {}      {}{} a\n\
                 {}   {}  {}s a\n\
                 {}      {}{} c\n\
                 {}",
                start_path_title,
                path,
                path_at,
                single_bar,
                single_bar,
                Style::new(Color::Blue).bold().paint("1"),
                Style::new(Color::Blue).bold().paint("This "),
                single_bar,
                Style::new(Color::Blue).bold().paint("└─────>"),
                single_bar,
                Style::new(Color::Blue).bold().paint("3"),
                Style::new(Color::Blue).bold().paint("document"),
                single_bar,
                Style::new(Color::Blue).bold().paint(">───────┘"),
                Style::new(Color::Blue).bold().paint("├─"),
                single_bar,
                Style::new(Color::Blue).bold().paint("1"),
                Style::new(Color::Blue).bold().paint("This "),
                Style::new(Color::Magenta).bold().paint(""),
                single_bar,
                Style::new(Color::Blue).bold().paint("├───┘"),
                Style::new(Color::Magenta).bold().paint("└"),
                Style::new(Color::Magenta).bold().paint("─"),
                single_bar,
                Style::new(Color::Blue).bold().paint("└"),
                Style::new(Color::Blue).bold().paint("──────"),
                single_bar,
                Style::new(Color::Blue).bold().paint("2"),
                Style::new(Color::Blue).bold().paint("i"),
                single_bar,
                Style::new(Color::Blue).bold().paint("└"),
                Style::new(Color::Blue).bold().paint("──"),
                end
            ),
            "Test 2"
        );

        let text = "This\nis a\ndocument";
        let log = Log::info().document(text, |document| {
            document
                .file_path("/path/to/file.test")
                .highlight_cursor_message(0, "x0", None)
                .highlight_cursor(1, None)
                .highlight_cursor_message(2, "x2", None)
                .highlight_cursor(3, None)
                .highlight_cursor_message(4, "x4", None)
                .highlight_cursor_message(5, "x5", None)
                .highlight_cursor(6, None)
                .highlight_cursor_message(7, "x7", None)
        });
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{}\n\
                 {} {} /path/to/file.test\n\
                 {}\n\
                 {}   {}  {}T{}h{}i{}s{}\n\
                 {}      {} {} {} {} {}{} x4\n\
                 {}      {}   {}{} x2\n\
                 {}      {}{} x0\n\
                 {}   {}  {}i{}s{} a\n\
                 {}      {} {} {}{} x7\n\
                 {}      {}{} x5\n\
                 {}",
                start_path_title,
                path,
                path_at,
                single_bar,
                single_bar,
                Style::new(Color::Blue).bold().paint("1"),
                Style::new(Color::Blue).bold().paint("·"),
                Style::new(Color::Magenta).bold().paint("·"),
                Style::new(Color::Blue).bold().paint("·"),
                Style::new(Color::Magenta).bold().paint("·"),
                Style::new(Color::Blue).bold().paint("·"),
                single_bar,
                Style::new(Color::Blue).bold().paint("│"),
                Style::new(Color::Magenta).bold().paint("^"),
                Style::new(Color::Blue).bold().paint("│"),
                Style::new(Color::Magenta).bold().paint("^"),
                Style::new(Color::Blue).bold().paint("└"),
                Style::new(Color::Blue).bold().paint("──"),
                single_bar,
                Style::new(Color::Blue).bold().paint("│"),
                Style::new(Color::Blue).bold().paint("└"),
                Style::new(Color::Blue).bold().paint("──────"),
                single_bar,
                Style::new(Color::Blue).bold().paint("└"),
                Style::new(Color::Blue).bold().paint("──────────"),
                single_bar,
                Style::new(Color::Blue).bold().paint("2"),
                Style::new(Color::Magenta).bold().paint("·"),
                Style::new(Color::Blue).bold().paint("·"),
                Style::new(Color::Magenta).bold().paint("·"),
                single_bar,
                Style::new(Color::Magenta).bold().paint("│"),
                Style::new(Color::Blue).bold().paint("^"),
                Style::new(Color::Magenta).bold().paint("└"),
                Style::new(Color::Magenta).bold().paint("──"),
                single_bar,
                Style::new(Color::Magenta).bold().paint("└"),
                Style::new(Color::Magenta).bold().paint("──────"),
                end,
            ),
            "Test 3"
        );
    }
}
