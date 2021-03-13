use arcstr::ArcStr;
use yansi::Color;

use crate::utils::text::{color_bold_if, remove_jump_lines};
use crate::Log;

/// A block that prints a tag.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TagBlock {
    tag: ArcStr,
}

impl TagBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(tag: ArcStr) -> TagBlock {
        TagBlock { tag }
    }

    // GETTERS ----------------------------------------------------------------

    /// The tag of the block.
    pub fn get_tag(&self) -> &ArcStr {
        &self.tag
    }

    // SETTERS ----------------------------------------------------------------

    pub fn tag(mut self, tag: ArcStr) -> Self {
        self.tag = tag;
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
    use yansi::Style;

    use crate::{Log, LogLevel};

    use super::*;

    #[test]
    fn test_plain() {
        let log = Log::info().tag("TAG".into());
        let text = log.to_plain_text();

        assert_eq!(text, format!("= TAG"));
    }

    #[test]
    fn test_ansi() {
        let log = Log::info().tag("TAG".into());
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
