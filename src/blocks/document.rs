use std::cmp::max;
use std::ops::Range;
use std::option::Option::Some;
use std::sync::Arc;

use yansi::Color;

use crate::constants::{
    BOTTOM_RIGHT_CORNER, HORIZONTAL_BAR, HORIZONTAL_BOTTOM_BAR, HORIZONTAL_TOP_BAR, MIDDLE_DOT,
    NEW_LINE, RIGHT_POINTER, TOP_LEFT_CORNER, TOP_RIGHT_CORNER, UP_POINTER, VERTICAL_BAR,
    VERTICAL_LEFT_BAR, VERTICAL_RIGHT_BAR,
};
use crate::utils::cursor::Cursor;
use crate::utils::text::{indent_text, remove_jump_lines};
use crate::utils::RangeMap;
use crate::Log;

/// A block that prints a section of a document.
#[derive(Debug, Clone)]
pub struct DocumentBlock {
    content: Arc<String>,
    sections: RangeMap<HighlightedSection>,
    related_block: Option<Box<DocumentBlock>>,
    title: Option<Arc<String>>,
    file_path: Option<Arc<String>>,
    end_message: Option<Arc<String>>,
    show_new_line_chars: bool,
    secondary_color: Color,
}

impl DocumentBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(content: Arc<String>) -> DocumentBlock {
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
    pub fn get_content(&self) -> &Arc<String> {
        &self.content
    }

    /// The related block of this one.
    pub fn get_related_block(&self) -> &Option<Box<DocumentBlock>> {
        &self.related_block
    }

    /// The title message to show at the top of the block.
    /// Ignored if the block belongs to another one as a related block.
    pub fn get_title(&self) -> &Option<Arc<String>> {
        &self.title
    }

    /// The file path of the document.
    /// Ignored if the block belongs to another one as a related block.
    pub fn get_file_path(&self) -> &Option<Arc<String>> {
        &self.file_path
    }

    /// The final message to show at the bottom of the block.
    /// Ignored if the block belongs to another one as a related block.
    pub fn get_end_message(&self) -> &Option<Arc<String>> {
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

    pub fn title(mut self, title: Arc<String>) -> Self {
        self.title = Some(title);
        self
    }

    pub fn title_str(self, title: &str) -> Self {
        self.title(Arc::new(title.to_string()))
    }

    pub fn clear_title(mut self) -> Self {
        self.title = None;
        self
    }

    pub fn file_path(mut self, file_path: Arc<String>) -> Self {
        self.file_path = Some(file_path);
        self
    }

    pub fn file_path_str(mut self, file_path: &str) -> Self {
        self.file_path = Some(Arc::new(file_path.to_string()));
        self
    }

    pub fn clear_file_path(mut self) -> Self {
        self.file_path = None;
        self
    }

    pub fn end_message(mut self, end_message: Arc<String>) -> Self {
        self.end_message = Some(end_message);
        self
    }

    pub fn end_message_str(mut self, end_message: &str) -> Self {
        self.end_message = Some(Arc::new(end_message.to_string()));
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
    pub fn highlight_cursor(
        self,
        position: usize,
        message: Option<Arc<String>>,
        color: Option<Color>,
    ) -> Self {
        self.highlight_section(position..position, message, color)
    }

    /// Highlights a cursor position.
    /// Equals to `highlight_section_str(position..position, ..)`.
    pub fn highlight_cursor_str(
        self,
        position: usize,
        message: Option<&str>,
        color: Option<Color>,
    ) -> Self {
        self.highlight_cursor(position, message.map(|v| Arc::new(v.to_string())), color)
    }

    /// Highlights a section.
    pub fn highlight_section(
        mut self,
        range: Range<usize>,
        message: Option<Arc<String>>,
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

    /// Highlights a section.
    pub fn highlight_section_str(
        self,
        range: Range<usize>,
        message: Option<&str>,
        color: Option<Color>,
    ) -> Self {
        self.highlight_section(range, message.map(|v| Arc::new(v.to_string())), color)
    }

    /// Sets a related block to the block.
    pub fn related_block<F>(mut self, builder: F) -> Self
    where
        F: FnOnce(DocumentBlock) -> DocumentBlock,
    {
        let document = DocumentBlock::new(self.content.clone());
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
        log: &Log,
        in_ansi: bool,
        buffer: &mut String,
        is_related_block: bool,
    ) {
        if in_ansi {
            // TODO
        } else {
            // FIRST LINE + TITLE
            if is_related_block {
                buffer.push_str(VERTICAL_RIGHT_BAR);
                buffer.push_str(HORIZONTAL_BAR);

                if let Some(title) = &self.title {
                    let title = indent_text(&title, format!("{}  ", VERTICAL_BAR).as_str(), false);
                    buffer.push_str(" ");
                    buffer.push_str(title.as_str());
                }
            } else {
                buffer.push_str(BOTTOM_RIGHT_CORNER);

                if self.file_path.is_some() {
                    buffer.push_str(HORIZONTAL_BOTTOM_BAR);
                    buffer.push_str(HORIZONTAL_BAR);
                } else {
                    buffer.push_str(HORIZONTAL_BAR);
                }

                if let Some(title) = &self.title {
                    let title = if self.file_path.is_some() {
                        indent_text(
                            &title,
                            format!("{}{}  ", VERTICAL_BAR, VERTICAL_BAR).as_str(),
                            false,
                        )
                    } else {
                        indent_text(&title, format!("{}  ", VERTICAL_BAR).as_str(), false)
                    };
                    buffer.push_str(" ");
                    buffer.push_str(title.as_str());
                }

                // FILE
                if let Some(file_path) = &self.file_path {
                    let file_path = remove_jump_lines(file_path.as_str());

                    buffer.push('\n');
                    buffer.push_str(VERTICAL_BAR);
                    buffer.push_str(TOP_RIGHT_CORNER);
                    buffer.push_str(RIGHT_POINTER);
                    buffer.push_str(" at ");
                    buffer.push_str(file_path.as_str());
                }
            }

            // SECTIONS
            if !self.sections.is_empty() {
                // Normalize sections.
                let mut normalized_sections = self.normalize_sections();
                let last_line = normalized_sections.last().unwrap().to.line;
                let last_content_line = Cursor::from_byte_offset_and_cursor(
                    &self.content,
                    self.content.len(),
                    &normalized_sections.last().unwrap().to,
                )
                .line;
                let max_line_digits = last_line.to_string().len();

                // EMPTY LINE
                // Only if file path is present or the title is multiline.
                if self.file_path.is_some()
                    || self
                        .title
                        .as_ref()
                        .map_or(false, |v| memchr::memchr(b'\n', v.as_bytes()).is_some())
                {
                    buffer.push('\n');
                    buffer.push_str(VERTICAL_BAR);
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
                    buffer.push_str(VERTICAL_BAR);
                    buffer.push_str("   ");
                    buffer.push_str(
                        format!(
                            "{:>width$}",
                            last_section.from.line,
                            width = max_line_digits
                        )
                        .as_str(),
                    );
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
                        buffer.push_str(NEW_LINE);
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
                    let digits_as_whitespace = " ".repeat(max_line_digits);

                    for arrow_line in 0..arrow_lines_height {
                        let mut arrow_lines_height = arrow_lines_height;

                        buffer.push('\n');
                        buffer.push_str(VERTICAL_BAR);
                        buffer.push_str("   ");
                        buffer.push_str(&digits_as_whitespace);
                        buffer.push_str("  ");

                        // SECTIONS
                        let mut prev_cursor = start_line_cursor.clone();
                        let mut previous_cursors = 0;
                        for section in &sections_in_same_line {
                            if section.is_cursor {
                                previous_cursors += 1;
                            }

                            // PREVIOUS CONTENT
                            buffer.push_str(
                                &" ".repeat(section.from.char_offset - prev_cursor.char_offset),
                            );

                            // CONTENT
                            section.print(
                                buffer,
                                self,
                                in_ansi,
                                arrow_line,
                                arrow_lines_height,
                                message_column,
                                previous_cursors,
                            );

                            // Exit on last.
                            if arrow_line + 1 == arrow_lines_height {
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
                buffer.push_str(TOP_RIGHT_CORNER);
                buffer.push_str(HORIZONTAL_BAR);

                if let Some(end_message) = &self.end_message {
                    buffer.push_str(" ");
                    buffer.push_str(indent_text(end_message.as_str(), "   ", false).as_str());
                }
            }
        }
    }

    fn get_other_color(&self, log: &Log, current: &Color) -> Color {
        if current == &self.secondary_color {
            log.level().color()
        } else {
            self.secondary_color
        }
    }

    fn normalize_sections(&self) -> Vec<HighlightedSection> {
        self.sections
            .iter()
            .flat_map(|(_, s)| {
                // To normalizing multiline sections we must split them into two different sections.
                if s.is_multiline_start {
                    if s.content(&self.content).ends_with("\n") {
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
                                color: s.color.clone(),
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
                                    color: s.color.clone(),
                                    is_multiline_start: true,
                                    is_multiline_end: false,
                                    is_cursor: false,
                                },
                                HighlightedSection {
                                    from: to.start_line_cursor(&self.content),
                                    to,
                                    message: s.message.clone(),
                                    color: s.color.clone(),
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
                                color: s.color.clone(),
                                is_multiline_start: true,
                                is_multiline_end: false,
                                is_cursor: false,
                            },
                            HighlightedSection {
                                from: s.to.start_line_cursor(&self.content),
                                to: s.to.clone(),
                                message: s.message.clone(),
                                color: s.color.clone(),
                                is_multiline_start: false,
                                is_multiline_end: true,
                                is_cursor: false,
                            },
                        ]
                    }
                } else {
                    // Single line sections are kept the same.
                    vec![s.clone()]
                }
            })
            .collect()
    }

    // STATIC METHODS ---------------------------------------------------------

    fn get_sections_in_same_line(
        &self,
        sections: &mut Vec<HighlightedSection>,
        sections_in_same_line: &mut Vec<HighlightedSection>,
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
struct HighlightedSection {
    // Inclusive range from..=to.
    from: Cursor,
    to: Cursor,
    message: Option<Arc<String>>,
    color: Option<Color>,
    is_multiline_start: bool,
    is_multiline_end: bool,
    is_cursor: bool,
}

impl HighlightedSection {
    // GETTERS ----------------------------------------------------------------

    pub fn char_len(&self) -> usize {
        self.to.char_offset - self.from.char_offset
    }

    pub fn content<'a>(&self, text: &'a str) -> &'a str {
        self.from.slice(text, &self.to)
    }

    // METHODS ----------------------------------------------------------------

    fn print_content_section(&self, document: &DocumentBlock, in_ansi: bool, buffer: &mut String) {
        if in_ansi {
            // TODO
        } else {
            if self.is_cursor {
                buffer.push_str(MIDDLE_DOT);
            } else {
                let content = self.content(&document.content).trim();
                buffer.push_str(content);
            }
        }
    }

    fn print(
        &self,
        buffer: &mut String,
        document: &DocumentBlock,
        in_ansi: bool,
        arrow_line: usize,
        arrow_lines_height: usize,
        message_column: usize,
        previous_cursors: usize,
    ) {
        if arrow_line + 1 > arrow_lines_height {
            return;
        }

        if arrow_line == 0 {
            self.print_first_arrow_line(
                buffer,
                document,
                in_ansi,
                message_column,
                arrow_lines_height == 1,
                previous_cursors,
            )
        } else if arrow_line + 1 == arrow_lines_height {
            self.print_last_arrow_line(buffer, document, in_ansi, message_column, previous_cursors)
        } else {
            self.print_middle_arrow_line(buffer, document, in_ansi);
        }
    }

    fn print_first_arrow_line(
        &self,
        buffer: &mut String,
        document: &DocumentBlock,
        in_ansi: bool,
        message_column: usize,
        print_message: bool,
        previous_cursors: usize,
    ) {
        if in_ansi {
            // TODO
        } else {
            let char_length = self.char_len();

            if self.is_multiline_start {
                buffer.push_str(TOP_RIGHT_CORNER);
                buffer.push_str(&HORIZONTAL_BAR.repeat(char_length));
                buffer.push_str(RIGHT_POINTER);
            } else if self.is_multiline_end {
                buffer.pop().unwrap();
                buffer.push_str(RIGHT_POINTER);

                if let Some(message) = &self.message {
                    if print_message {
                        // With message at first line.
                        if char_length > 0 {
                            buffer.push_str(&HORIZONTAL_BAR.repeat(char_length - 1));
                        }

                        buffer.push_str(HORIZONTAL_TOP_BAR);
                        buffer.push_str(
                            &HORIZONTAL_BAR
                                .repeat(message_column - self.to.column - previous_cursors),
                        );
                        buffer.push(' ');
                        buffer.push_str(&remove_jump_lines(message));
                    } else {
                        // With message at other line.
                        if char_length <= 1 {
                            buffer.push_str(VERTICAL_LEFT_BAR);
                        } else {
                            buffer.push_str(HORIZONTAL_BOTTOM_BAR);
                            buffer.push_str(&HORIZONTAL_BAR.repeat(char_length - 2));
                            buffer.push_str(TOP_LEFT_CORNER);
                        }
                    }
                } else {
                    // No message.
                    if char_length > 0 {
                        buffer.push_str(&HORIZONTAL_BAR.repeat(char_length - 1));
                    }

                    buffer.push_str(TOP_LEFT_CORNER);
                }
            } else {
                match char_length {
                    0 | 1 => {
                        if let Some(message) = &self.message {
                            if print_message {
                                // With message at first line.
                                buffer.push_str(TOP_RIGHT_CORNER);
                                buffer.push_str(&HORIZONTAL_BAR.repeat(
                                    message_column - self.to.column + 1 - previous_cursors,
                                ));
                                buffer.push(' ');
                                buffer.push_str(&remove_jump_lines(message));
                            } else {
                                // With message at other line.
                                buffer.push_str(VERTICAL_BAR);
                            }
                        } else {
                            // No message.
                            buffer.push_str(UP_POINTER);
                        }
                    }
                    _ => {
                        if let Some(message) = &self.message {
                            if print_message {
                                // With message at first line.
                                buffer.push_str(TOP_RIGHT_CORNER);
                                buffer.push_str(&HORIZONTAL_BAR.repeat(char_length - 2));
                                buffer.push_str(HORIZONTAL_TOP_BAR);
                                buffer.push_str(&HORIZONTAL_BAR.repeat(
                                    message_column - self.to.column + 1 - previous_cursors,
                                ));
                                buffer.push(' ');
                                buffer.push_str(&remove_jump_lines(message));
                            } else {
                                // With message at other line.
                                buffer.push_str(VERTICAL_RIGHT_BAR);
                                buffer.push_str(&HORIZONTAL_BAR.repeat(char_length - 2));
                                buffer.push_str(TOP_LEFT_CORNER);
                            }
                        } else {
                            // No message.
                            buffer.push_str(TOP_RIGHT_CORNER);
                            buffer.push_str(&HORIZONTAL_BAR.repeat(char_length - 2));
                            buffer.push_str(TOP_LEFT_CORNER);
                        }
                    }
                }
            }
        }
    }

    fn print_middle_arrow_line(
        &self,
        buffer: &mut String,
        document: &DocumentBlock,
        in_ansi: bool,
    ) {
        if in_ansi {
            // TODO
        } else {
            let char_length = self.char_len();

            match char_length {
                0 | 1 => {
                    buffer.push_str(VERTICAL_BAR);
                }
                _ => {
                    buffer.push_str(VERTICAL_BAR);
                    buffer.push_str(&" ".repeat(char_length - 1));
                }
            }
        }
    }

    fn print_last_arrow_line(
        &self,
        buffer: &mut String,
        document: &DocumentBlock,
        in_ansi: bool,
        message_column: usize,
        previous_cursors: usize,
    ) {
        if in_ansi {
            // TODO
        } else {
            let char_length = self.char_len();

            match char_length {
                0 | 1 => {
                    let message = self
                        .message
                        .as_ref()
                        .expect("Cannot call print_last_arrow_line without a message");
                    buffer.push_str(TOP_RIGHT_CORNER);
                    buffer.push_str(
                        &HORIZONTAL_BAR
                            .repeat(message_column - self.to.column + 1 - previous_cursors),
                    );
                    buffer.push(' ');
                    buffer.push_str(&remove_jump_lines(message));
                }
                _ => {
                    let message = self
                        .message
                        .as_ref()
                        .expect("Cannot call print_last_arrow_line without a message");
                    buffer.push_str(TOP_RIGHT_CORNER);
                    buffer.push_str(
                        &HORIZONTAL_BAR.repeat(
                            message_column - self.to.column + char_length - previous_cursors,
                        ),
                    );
                    buffer.push(' ');
                    buffer.push_str(&remove_jump_lines(message));
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // This test covers every branch in the code.
    #[test]
    fn test_plain() {
        let text = "First\ntest\nthird\nline";
        let log = Log::info().document_str(text, |document| {
            document
                .show_new_line_chars(true)
                .highlight_section_str(1..3, Some("Comment\nmultiline"), None)
                .highlight_cursor_str(3, Some("Comment cursor"), None)
                .highlight_section_str(3..4, Some("Comment"), Some(Color::Red))
                .highlight_section_str(5..7, Some("Comment jump line"), Some(Color::Red))
                .highlight_section_str(7..8, Some("A"), Some(Color::Red))
                .highlight_section_str(8..20, Some("B"), Some(Color::Red))
                .highlight_section_str(20..21, Some("C"), Some(Color::Red))
        });
        let text = log.to_plain_text();

        assert_eq!(
            &text,
            "┌─\n\
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
             └─"
        );

        let text = "Second\ntest";
        let log = Log::info().document_str(text, |document| {
            document
                .show_new_line_chars(true)
                .highlight_section_str(1..2, None, None)
                .highlight_section_str(3..5, Some("1+ chars with message"), None)
                .highlight_section_str(8..10, None, None)
        });
        let text = log.to_plain_text();

        assert_eq!(
            &text,
            "┌─\n\
             │   1  Second↩\n\
             │       ^\n\
             │   2  test\n\
             │       └┘\n\
             └─"
        );

        let text = "This\na\ndocument";
        let log = Log::info().document_str(text, |document| {
            document.highlight_section_str(0..text.len(), None, None)
        });
        let text = log.to_plain_text();

        assert_eq!(
            &text,
            "┌─\n\
             │   1  This\n\
             │      └────>\n\
             │   3  document\n\
             │     >───────┘\n\
             └─"
        );

        let text = "This\r\na\ndocument";
        let log = Log::info().document_str(text, |document| {
            document
                .highlight_section_str(0..5, Some("a"), None)
                .highlight_section_str(5..6, Some("b"), None)
                .highlight_section_str(6..7, Some("c"), None)
        });
        let text = log.to_plain_text();

        assert_eq!(
            &text,
            "┌─\n\
             │   1  This\n\
             │      ├───┘└── b\n\
             │      └────── a\n\
             │   2  a\n\
             │      └── c\n\
             └─"
        );
    }

    #[test]
    fn test_ansi() {
        let text = "First\ntest\nthird\nline";
        let log = Log::info().document_str(text, |document| {
            document
                .show_new_line_chars(true)
                .highlight_section_str(1..3, Some("Comment\nmultiline"), None)
                .highlight_cursor_str(3, Some("Comment cursor"), None)
                .highlight_section_str(3..4, Some("Comment"), Some(Color::Red))
                .highlight_section_str(5..7, Some("Comment jump line"), Some(Color::Red))
                .highlight_section_str(7..8, Some("A"), Some(Color::Red))
                .highlight_section_str(8..20, Some("B"), Some(Color::Red))
                .highlight_section_str(20..21, Some("C"), Some(Color::Red))
        });
        let text = log.to_ansi_text();

        println!("{}", text)
    }
}
