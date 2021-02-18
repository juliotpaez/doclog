/// Indents `text` prefixing each line with `indent`.
/// If `indent_first` is `true`, the first line is also prefixed.
pub fn indent_text(text: &str, buffer: &mut String, indent: &str, indent_first: bool) {
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            buffer.push('\n');
        }

        if indent_first || i > 0 {
            buffer.push_str(indent);
        }
        buffer.push_str(line);
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_text() {
        let mut buffer = String::new();
        indent_text("this\nis\na\ntest", &mut buffer, "---", false);

        assert_eq!(buffer, format!("this\n---is\n---a\n---test"));

        let mut buffer = String::new();
        indent_text("this\nis\na\ntest", &mut buffer, "---", true);

        assert_eq!(buffer, format!("---this\n---is\n---a\n---test"));
    }
}
