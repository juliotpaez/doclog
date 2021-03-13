use arcstr::ArcStr;

/// A block that prints a plain text.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlainTextBlock {
    message: ArcStr,
}

impl PlainTextBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new<M: Into<ArcStr>>(message: M) -> PlainTextBlock {
        PlainTextBlock {
            message: message.into(),
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The message to print.
    pub fn get_message(&self) -> &ArcStr {
        &self.message
    }

    // SETTERS ----------------------------------------------------------------

    pub fn message<M: Into<ArcStr>>(mut self, message: M) -> Self {
        self.message = message.into();
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, buffer: &mut String) {
        buffer.push_str(self.get_message());
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::Log;

    #[test]
    fn test_plain() {
        let log = Log::error().plain_text("This is\na test");
        let text = log.to_plain_text();

        assert_eq!(text, format!("This is\na test"));
    }

    #[test]
    fn test_ansi() {
        let log = Log::info().plain_text("This is\na test");
        let text = log.to_ansi_text();

        assert_eq!(text, format!("This is\na test"));
    }
}
