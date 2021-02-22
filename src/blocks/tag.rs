use std::sync::Arc;

use yansi::{Color, Style};

use crate::utils::text::{color_bold_if, remove_jump_lines};
use crate::Log;

/// A block that prints a tag.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TagBlock {
    tag: Arc<String>,
}

impl TagBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(tag: Arc<String>) -> TagBlock {
        TagBlock { tag }
    }

    // GETTERS ----------------------------------------------------------------

    /// The tag of the block.
    pub fn get_tag(&self) -> &Arc<String> {
        &self.tag
    }

    // SETTERS ----------------------------------------------------------------

    pub fn tag(mut self, tag: Arc<String>) -> Self {
        self.tag = tag;
        self
    }

    pub fn tag_str(mut self, tag: &str) -> Self {
        self.tag = Arc::new(tag.to_string());
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let tag = remove_jump_lines(&self.tag);

        buffer.push_str(&color_bold_if(
            "=".to_string(),
            log.level().color(),
            in_ansi,
        ));
        buffer.push(' ');
        buffer.push_str(&color_bold_if(tag.to_string(), Color::Unset, in_ansi));
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
