use std::borrow::Cow;

use yansi::Color;

use crate::utils::text::{color_bold_if, indent_text, remove_jump_lines};
use crate::Log;

/// A block that prints a note.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoteBlock<'a> {
    title: Cow<'a, str>,
    message: Cow<'a, str>,
}

impl<'a> NoteBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(title: Cow<'a, str>, message: Cow<'a, str>) -> NoteBlock<'a> {
        NoteBlock { title, message }
    }

    // GETTERS ----------------------------------------------------------------

    /// The title of the block.
    pub fn get_title(&self) -> &Cow<'a, str> {
        &self.title
    }

    /// The message of the block.
    pub fn get_message(&self) -> &Cow<'a, str> {
        &self.message
    }

    // SETTERS ----------------------------------------------------------------

    pub fn title(mut self, title: Cow<'a, str>) -> Self {
        self.title = title;
        self
    }

    pub fn message(mut self, message: Cow<'a, str>) -> Self {
        self.message = message;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let title = remove_jump_lines(&self.title);
        let message = indent_text(
            self.message.as_ref(),
            " ".repeat(4 + title.len()).as_str(),
            false,
        );

        buffer.push_str(&color_bold_if(
            "=".to_string(),
            log.level().color(),
            in_ansi,
        ));
        buffer.push(' ');
        buffer.push_str(&color_bold_if(title, Color::Unset, in_ansi));
        buffer.push_str(&color_bold_if(
            ":".to_string(),
            log.level().color(),
            in_ansi,
        ));
        buffer.push(' ');
        buffer.push_str(message.as_str());
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use yansi::Style;

    use crate::{Log, LogLevel};

    use super::*;

    #[test]
    fn test_plain() {
        let log = Log::info().note("title\nmultiline1", "message\nmultiline2");
        let text = log.to_plain_text();

        assert_eq!(
            text,
            "= title multiline1: message\n                    multiline2"
        );
    }

    #[test]
    fn test_ansi() {
        let log = Log::info().note("title\nmultiline1", "message\nmultiline2");
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} {}{} message\n                    multiline2",
                Style::new(LogLevel::info().color()).bold().paint("="),
                Style::new(Color::Unset).bold().paint("title multiline1"),
                Style::new(LogLevel::info().color()).bold().paint(":"),
            )
        );
    }
}
