use yansi::{Color, Style};

use crate::utils::text::remove_jump_lines;
use crate::Log;

/// A block that prints a tag.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TagBlock {
    tag: String,
}

impl TagBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(tag: String) -> TagBlock {
        TagBlock { tag }
    }

    // GETTERS ----------------------------------------------------------------

    /// The tag of the block.
    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    // SETTERS ----------------------------------------------------------------

    pub fn tag(mut self, tag: String) -> Self {
        self.tag = tag;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let tag = remove_jump_lines(&self.tag);

        if in_ansi {
            buffer.push_str(
                format!(
                    "{} {}",
                    Style::new(log.level().color()).bold().paint("="),
                    Style::new(Color::Unset).bold().paint(tag)
                )
                .as_str(),
            );
        } else {
            buffer.push_str("= ");
            buffer.push_str(tag.as_str());
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
        let log = Log::info().tag_str("TAG");
        let text = log.to_plain_text();

        assert_eq!(text, format!("= TAG"));
    }

    #[test]
    fn test_ansi() {
        let log = Log::info().tag_str("TAG");
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} {}",
                Style::new(LogLevel::info().color()).bold().paint("="),
                Style::new(Color::Unset).bold().paint("TAG")
            )
        );
    }
}