/// Removes the jump lines of `text`, changing them to spaces.
pub fn remove_jump_lines(text: &str) -> String {
    text.replace('\n', " ")
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_jump_lines() {
        let result = remove_jump_lines("this\nis\na\ntest");
        assert_eq!(result, "this is a test");
    }
}
