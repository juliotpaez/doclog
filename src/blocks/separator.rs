use crate::constants::HORIZONTAL_BAR;
use crate::utils::text::color_bold_if;
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
    pub fn get_width(&self) -> usize {
        self.width
    }

    // SETTERS ----------------------------------------------------------------

    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let separator = HORIZONTAL_BAR.repeat(self.width);
        buffer.push_str(&color_bold_if(separator, log.level().color(), in_ansi));
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use yansi::Style;

    use crate::{Log, LogLevel};

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
