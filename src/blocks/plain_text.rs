/// A block that prints a plain text.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlainTextBlock {
    message: String,
}

impl PlainTextBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(message: String) -> PlainTextBlock {
        PlainTextBlock { message }
    }

    // GETTERS ----------------------------------------------------------------

    /// The message to print.
    pub fn get_message(&self) -> &str {
        &self.message
    }

    // SETTERS ----------------------------------------------------------------

    pub fn message(mut self, message: String) -> Self {
        self.message = message;
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
        let log = Log::error().plain_text_str("This is\na test");
        let text = log.to_plain_text();

        assert_eq!(text, format!("This is\na test"));
    }

    #[test]
    fn test_ansi() {
        let log = Log::info().plain_text_str("This is\na test");
        let text = log.to_ansi_text();

        assert_eq!(text, format!("This is\na test"));
    }
}
