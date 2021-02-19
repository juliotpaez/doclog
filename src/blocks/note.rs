use yansi::{Color, Style};

use crate::utils::text::{indent_text, remove_jump_lines};
use crate::Log;

/// A block that prints a note.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoteBlock {
    title: String,
    message: String,
}

impl NoteBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(title: String, message: String) -> NoteBlock {
        NoteBlock { title, message }
    }

    // GETTERS ----------------------------------------------------------------

    /// The title of the block.
    pub fn get_title(&self) -> &str {
        &self.title
    }

    /// The message of the block.
    pub fn get_message(&self) -> &str {
        &self.message
    }

    // SETTERS ----------------------------------------------------------------

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let title = remove_jump_lines(&self.title);
        let message = indent_text(
            self.message.as_str(),
            " ".repeat(4 + title.len()).as_str(),
            false,
        );

        if in_ansi {
            buffer.push_str(
                format!(
                    "{} {}{}",
                    Style::new(log.level().color()).bold().paint("="),
                    Style::new(Color::Unset).bold().paint(title),
                    Style::new(log.level().color()).bold().paint(":"),
                )
                .as_str(),
            );
            buffer.push_str(" ");
            buffer.push_str(message.as_str());
        } else {
            buffer.push_str("= ");
            buffer.push_str(title.as_str());
            buffer.push_str(": ");
            buffer.push_str(message.as_str());
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::{Log, LogLevel};

    use super::*;

    #[test]
    fn test_plain() {
        let log = Log::info().note_str("title\nmultiline1", "message\nmultiline2");
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!("= titlemultiline1: message\n                   multiline2")
        );
    }

    #[test]
    fn test_ansi() {
        let log = Log::info().note_str("title\nmultiline1", "message\nmultiline2");
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} {}{} message\n                   multiline2",
                Style::new(LogLevel::info().color()).bold().paint("="),
                Style::new(Color::Unset).bold().paint("titlemultiline1"),
                Style::new(LogLevel::info().color()).bold().paint(":"),
            )
        );
    }
}
