use std::borrow::Cow;

use yansi::Color;

use crate::utils::text::{color_bold_if, remove_jump_lines};
use crate::Log;

/// A block that prints a tag.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TagBlock<'a> {
    tag: Cow<'a, str>,
}

impl<'a> TagBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(tag: Cow<'a, str>) -> TagBlock {
        TagBlock { tag }
    }

    // GETTERS ----------------------------------------------------------------

    /// The tag of the block.
    pub fn get_tag(&self) -> &Cow<'a, str> {
        &self.tag
    }

    // SETTERS ----------------------------------------------------------------

    pub fn tag(mut self, tag: Cow<'a, str>) -> Self {
        self.tag = tag;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log<'a>, in_ansi: bool, buffer: &mut String) {
        let tag = remove_jump_lines(&self.tag);

        buffer.push_str(&color_bold_if(
            "=".to_string(),
            log.level().color(),
            in_ansi,
        ));
        buffer.push(' ');
        buffer.push_str(&color_bold_if(tag, Color::Unset, in_ansi));
    }

    pub fn make_owned<'b>(&self) -> TagBlock<'b> {
        TagBlock {
            tag: Cow::Owned(self.tag.to_string()),
        }
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
        let log = Log::info().tag("TAG");
        let text = log.to_plain_text();

        assert_eq!(text, "= TAG");
    }

    #[test]
    fn test_ansi() {
        let log = Log::info().tag("TAG");
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
