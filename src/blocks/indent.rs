use crate::utils::text::indent_text;
use crate::Log;

/// A block that prints a line separator.
#[derive(Debug, Clone)]
pub struct IndentBlock {
    indent: usize,
    log: Box<Log>,
}

impl IndentBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(indent: usize, log: Box<Log>) -> IndentBlock {
        IndentBlock { indent, log }
    }

    // GETTERS ----------------------------------------------------------------

    /// The inner log of the indent.
    pub fn get_log(&self) -> &Log {
        &self.log
    }

    /// The indent length of the block.
    pub fn get_indent(&self) -> &usize {
        &self.indent
    }

    // SETTERS ----------------------------------------------------------------

    pub fn log(mut self, log: Box<Log>) -> Self {
        self.log = log;
        self
    }

    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, in_ansi: bool, buffer: &mut String) {
        let mut inner_buffer = String::new();
        self.log.to_text_internal(in_ansi, &mut inner_buffer);

        buffer.push_str(
            indent_text(
                inner_buffer.as_str(),
                " ".repeat(self.indent).as_str(),
                true,
            )
            .as_str(),
        );
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
        let log = Log::error().indent(4, |log| log.plain_text("Plain\ntext".into()));
        let text = log.to_plain_text();

        assert_eq!(text, format!("    Plain\n    text"));
    }

    #[test]
    fn test_ansi() {
        let log = Log::error().indent(2, |log| log.plain_text("Plain\ntext".into()));
        let text = log.to_ansi_text();

        assert_eq!(text, format!("  Plain\n  text"));
    }
}
