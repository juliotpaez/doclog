use crate::utils::text::indent_text;
use crate::Log;

/// A block that prints a line separator.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndentBlock {
    log: Box<Log>,
}

impl IndentBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(log: Box<Log>) -> IndentBlock {
        IndentBlock { log }
    }

    // GETTERS ----------------------------------------------------------------

    /// The inner log of the indent.
    pub fn get_log(&self) -> &Box<Log> {
        &self.log
    }

    // SETTERS ----------------------------------------------------------------

    pub fn log(mut self, log: Box<Log>) -> Self {
        self.log = log;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, in_ansi: bool, buffer: &mut String) {
        let mut inner_buffer = String::new();
        self.log.to_text_internal(in_ansi, &mut inner_buffer);

        buffer.push_str(indent_text(inner_buffer.as_str(), "    ", true).as_str());
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
        let log = Log::error().indent(|log| log.plain_text_str("Plain\ntext"));
        let text = log.to_plain_text();

        assert_eq!(text, format!("    Plain\n    text"));
    }

    #[test]
    fn test_ansi() {
        let log = Log::error().indent(|log| log.plain_text_str("Plain\ntext"));
        let text = log.to_ansi_text();

        assert_eq!(text, format!("    Plain\n    text"));
    }
}
