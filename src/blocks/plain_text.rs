use std::borrow::Cow;

/// A block that prints a plain text.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlainTextBlock<'a> {
    message: Cow<'a, str>,
}

impl<'a> PlainTextBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(message: Cow<'a, str>) -> PlainTextBlock {
        PlainTextBlock { message }
    }

    // GETTERS ----------------------------------------------------------------

    /// The message to print.
    pub fn get_message(&self) -> &Cow<'a, str> {
        &self.message
    }

    // SETTERS ----------------------------------------------------------------

    pub fn message(mut self, message: Cow<'a, str>) -> Self {
        self.message = message;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, buffer: &mut String) {
        buffer.push_str(self.get_message().as_ref());
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
