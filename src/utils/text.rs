/// Indents `text` prefixing each line with `indent`.
/// If `indent_first` is `true`, the first line is also prefixed.
pub fn indent_text(text: &str, indent: &str, indent_first: bool) -> String {
    let mut buffer = String::new();

    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            buffer.push('\n');
        }

        if indent_first || i > 0 {
            buffer.push_str(indent);
        }
        buffer.push_str(line);
    }

    buffer
}

/// Removes the jump lines of `text`.
pub fn remove_jump_lines(text: &str) -> String {
    let mut buffer = String::new();

    for line in text.lines() {
        buffer.push_str(line);
    }

    buffer
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_text() {
        let result = indent_text("this\nis\na\ntest", "---", false);
        assert_eq!(result, format!("this\n---is\n---a\n---test"));

        let result = indent_text("this\nis\na\ntest", "---", true);
        assert_eq!(result, format!("---this\n---is\n---a\n---test"));
    }

    #[test]
    fn test_remove_jump_lines() {
        let result = remove_jump_lines("this\nis\na\ntest");
        assert_eq!(result, format!("thisisatest"));
    }
}
