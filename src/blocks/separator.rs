use yansi::Style;

use crate::constants::HORIZONTAL_BAR;
use crate::Log;

/// A block that prints a line separator.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SeparatorBlock {
    width: usize,
}

impl SeparatorBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(width: usize) -> SeparatorBlock {
        SeparatorBlock { width }
    }

    // GETTERS ----------------------------------------------------------------

    /// The width of the line separator.
    pub fn width(&self) -> usize {
        self.width
    }

    // SETTERS ----------------------------------------------------------------

    pub fn set_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let separator = HORIZONTAL_BAR.repeat(self.width);
        if in_ansi {
            let style = Style::new(log.level().color()).bold();
            buffer.push_str(style.paint(separator).to_string().as_str());
        } else {
            buffer.push_str(separator.as_str());
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
        let log = Log::info().separator(0);
        let text = log.to_plain_text();

        assert_eq!(text, format!(""));

        let log = Log::error().separator(10);
        let text = log.to_plain_text();

        assert_eq!(text, format!("──────────"));
    }

    #[test]
    fn test_ansi() {
        let log = Log::info().separator(0);
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!("{}", Style::new(LogLevel::info().color()).bold().paint(""))
        );

        let log = Log::error().separator(10);
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{}",
                Style::new(LogLevel::error().color())
                    .bold()
                    .paint("──────────")
            )
        );
    }
}
