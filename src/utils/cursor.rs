use std::ops::Add;

/// A specific position in a text.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Cursor {
    pub byte_offset: usize,
    pub char_offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Cursor {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds the [Cursor] from a byte offset.
    pub fn from_byte_offset(text: &str, byte_offset: usize) -> Cursor {
        let prev_text = &text[..byte_offset];
        let start_line_offset = match memchr::memrchr(b'\n', prev_text.as_bytes()) {
            Some(v) => v + 1,
            None => 0,
        };

        Cursor {
            byte_offset,
            char_offset: bytecount::num_chars(prev_text.as_bytes()),
            line: bytecount::count(prev_text.as_bytes(), b'\n') + 1,
            column: bytecount::num_chars(prev_text[start_line_offset..].as_bytes()) + 1,
        }
    }

    /// Same as `from_byte_offset` but uses a cursor to optimize the building.
    pub fn from_byte_offset_and_cursor(text: &str, byte_offset: usize, cursor: &Cursor) -> Cursor {
        if cursor.byte_offset == byte_offset {
            return *cursor;
        }

        let prev_text = &text[..byte_offset];

        if cursor.byte_offset < byte_offset {
            let slice_from_cursor = &text[cursor.byte_offset..byte_offset];
            let start_line_offset = match memchr::memrchr(b'\n', prev_text.as_bytes()) {
                Some(v) => v + 1,
                None => 0,
            };

            Cursor {
                byte_offset,
                char_offset: cursor.char_offset
                    + bytecount::num_chars(slice_from_cursor.as_bytes()),
                line: cursor.line + bytecount::count(slice_from_cursor.as_bytes(), b'\n'),
                column: bytecount::num_chars(prev_text[start_line_offset..].as_bytes()) + 1,
            }
        } else {
            let slice_to_cursor = &text[byte_offset..cursor.byte_offset];
            let start_line_offset = match memchr::memrchr(b'\n', prev_text.as_bytes()) {
                Some(v) => v + 1,
                None => 0,
            };

            Cursor {
                byte_offset,
                char_offset: cursor.char_offset - bytecount::num_chars(slice_to_cursor.as_bytes()),
                line: cursor.line - bytecount::count(slice_to_cursor.as_bytes(), b'\n'),
                column: bytecount::num_chars(prev_text[start_line_offset..].as_bytes()) + 1,
            }
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Gets the cursor at the start of the specified line.
    pub fn find_line_start(&self, text: &str, line: usize) -> Option<Cursor> {
        let mut current_line = self.line;
        let mut start_line_offset = line_start_offset(text, self.byte_offset);

        if self.line < line {
            while current_line < line {
                let next_text = &text[start_line_offset..];
                start_line_offset += match memchr::memchr(b'\n', next_text.as_bytes()) {
                    Some(v) => v + 1,
                    None => {
                        if next_text.is_empty() {
                            return None;
                        } else {
                            next_text.len()
                        }
                    }
                };
                current_line += 1;
            }

            return Some(Cursor::from_byte_offset_and_cursor(
                text,
                start_line_offset,
                self,
            ));
        }

        while line < current_line {
            let prev_text = &text[..start_line_offset - 1];
            start_line_offset = match memchr::memrchr(b'\n', prev_text.as_bytes()) {
                Some(v) => v + 1,
                None => {
                    if prev_text.is_empty() {
                        return None;
                    } else {
                        0
                    }
                }
            };
            current_line -= 1;
        }

        Some(Cursor::from_byte_offset_and_cursor(
            text,
            start_line_offset,
            self,
        ))
    }

    /// Gets the cursor at the start of the line.
    pub fn start_line_cursor(&self, text: &str) -> Cursor {
        let line_start_offset = line_start_offset(text, self.byte_offset);
        Self::from_byte_offset_and_cursor(text, line_start_offset, self)
    }

    /// Gets the cursor at the start of the next line.
    pub fn next_start_line_cursor(&self, text: &str) -> Option<Cursor> {
        let line_start_offset = line_end_offset(text, self.byte_offset).add(1);

        if line_start_offset > text.len() {
            return None;
        }
        Some(Self::from_byte_offset_and_cursor(
            text,
            line_start_offset,
            self,
        ))
    }

    /// Gets the cursor at the end of the line.
    pub fn end_line_cursor(&self, text: &str) -> Cursor {
        let line_end_offset = line_end_offset(text, self.byte_offset);
        Self::from_byte_offset_and_cursor(text, line_end_offset, self)
    }

    /// Gets the content from the start of line.
    #[cfg(test)]
    pub fn slice_from_line_start<'a>(&self, text: &'a str) -> &'a str {
        let line_start_offset = line_start_offset(text, self.byte_offset);
        &text[line_start_offset..self.byte_offset]
    }

    /// Gets the content to the end of line.
    pub fn slice_to_line_end<'a>(&self, text: &'a str) -> &'a str {
        let line_end_offset = line_end_offset(text, self.byte_offset);
        &text[self.byte_offset..line_end_offset]
    }

    /// Gets the content between both cursors.
    pub fn slice<'a>(&self, text: &'a str, other: &Cursor) -> &'a str {
        if self.byte_offset < other.byte_offset {
            &text[self.byte_offset..other.byte_offset]
        } else {
            &text[other.byte_offset..self.byte_offset]
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Gets the byte_offset at the start of the line.
fn line_start_offset(text: &str, byte_offset: usize) -> usize {
    let prev_text = &text[..byte_offset];
    match memchr::memrchr(b'\n', prev_text.as_bytes()) {
        Some(v) => v + 1,
        None => 0,
    }
}

/// Gets the byte_offset at the end of the line.
fn line_end_offset(text: &str, byte_offset: usize) -> usize {
    let next_text = &text[byte_offset..];
    byte_offset
        + match memchr::memchr(b'\n', next_text.as_bytes()) {
            Some(v) => v,
            None => next_text.len(),
        }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_byte_offset() {
        let content = "This\nis\n- メカジキ - a\ntest";

        // First line.
        let first_line_length = "This\n".len();
        for i in 0..first_line_length {
            assert_eq!(
                Cursor::from_byte_offset(content, i),
                Cursor {
                    byte_offset: i,
                    char_offset: i,
                    line: 1,
                    column: i + 1,
                }
            )
        }

        // Second line.
        let second_line_length = "is\n".len();
        for i in 0..second_line_length {
            let j = i + first_line_length;
            assert_eq!(
                Cursor::from_byte_offset(content, j),
                Cursor {
                    byte_offset: j,
                    char_offset: j,
                    line: 2,
                    column: i + 1,
                }
            )
        }

        // Third line.
        let length = first_line_length + second_line_length + 2;
        assert_eq!(
            Cursor::from_byte_offset(content, length),
            Cursor {
                byte_offset: length,
                char_offset: length,
                line: 3,
                column: 3,
            }
        );
        assert_eq!(
            Cursor::from_byte_offset(content, length + 3),
            Cursor {
                byte_offset: length + 3,
                char_offset: length + 1,
                line: 3,
                column: 4,
            }
        );
        assert_eq!(
            Cursor::from_byte_offset(content, length + 6),
            Cursor {
                byte_offset: length + 6,
                char_offset: length + 2,
                line: 3,
                column: 5,
            }
        );
        assert_eq!(
            Cursor::from_byte_offset(content, length + 9),
            Cursor {
                byte_offset: length + 9,
                char_offset: length + 3,
                line: 3,
                column: 6,
            }
        );
    }

    #[test]
    fn test_from_byte_offset_and_cursor() {
        let content = "This\nis\n- メカジキ - a\ntest";
        let cursor = Cursor::from_byte_offset(content, 7);

        // First line.
        let first_line_length = "This\n".len();
        for i in 0..first_line_length {
            assert_eq!(
                Cursor::from_byte_offset_and_cursor(content, i, &cursor),
                Cursor {
                    byte_offset: i,
                    char_offset: i,
                    line: 1,
                    column: i + 1,
                }
            )
        }

        // Second line.
        let second_line_length = "is\n".len();
        for i in 0..second_line_length {
            let j = i + first_line_length;
            assert_eq!(
                Cursor::from_byte_offset_and_cursor(content, j, &cursor),
                Cursor {
                    byte_offset: j,
                    char_offset: j,
                    line: 2,
                    column: i + 1,
                }
            )
        }

        // Third line.
        let length = first_line_length + second_line_length + 2;
        assert_eq!(
            Cursor::from_byte_offset_and_cursor(content, length, &cursor),
            Cursor {
                byte_offset: length,
                char_offset: length,
                line: 3,
                column: 3,
            }
        );
        assert_eq!(
            Cursor::from_byte_offset_and_cursor(content, length + 3, &cursor),
            Cursor {
                byte_offset: length + 3,
                char_offset: length + 1,
                line: 3,
                column: 4,
            }
        );
        assert_eq!(
            Cursor::from_byte_offset_and_cursor(content, length + 6, &cursor),
            Cursor {
                byte_offset: length + 6,
                char_offset: length + 2,
                line: 3,
                column: 5,
            }
        );
        assert_eq!(
            Cursor::from_byte_offset_and_cursor(content, length + 9, &cursor),
            Cursor {
                byte_offset: length + 9,
                char_offset: length + 3,
                line: 3,
                column: 6,
            }
        );
    }

    #[test]
    fn test_start_line_cursor() {
        let content = "This\nis\n- メカジキ - a\ntest";

        assert_eq!(
            Cursor::from_byte_offset(content, 2).start_line_cursor(content),
            Cursor {
                byte_offset: 0,
                char_offset: 0,
                line: 1,
                column: 1,
            }
        );

        assert_eq!(
            Cursor::from_byte_offset(content, 4).start_line_cursor(content),
            Cursor {
                byte_offset: 0,
                char_offset: 0,
                line: 1,
                column: 1,
            }
        );

        assert_eq!(
            Cursor::from_byte_offset(content, 5).start_line_cursor(content),
            Cursor {
                byte_offset: 5,
                char_offset: 5,
                line: 2,
                column: 1,
            }
        )
    }

    #[test]
    fn test_end_line_cursor() {
        let content = "This\nis\n- メカジキ - a\ntest";

        assert_eq!(
            Cursor::from_byte_offset(content, 2).end_line_cursor(content),
            Cursor {
                byte_offset: 4,
                char_offset: 4,
                line: 1,
                column: 5,
            }
        );

        assert_eq!(
            Cursor::from_byte_offset(content, 4).end_line_cursor(content),
            Cursor {
                byte_offset: 4,
                char_offset: 4,
                line: 1,
                column: 5,
            }
        );

        assert_eq!(
            Cursor::from_byte_offset(content, 5).end_line_cursor(content),
            Cursor {
                byte_offset: 7,
                char_offset: 7,
                line: 2,
                column: 3,
            }
        )
    }

    #[test]
    fn test_slice() {
        let content = "This\nis\n- メカジキ - a\ntest";
        let from = Cursor::from_byte_offset(content, 2);
        let to = Cursor::from_byte_offset(content, 6);

        assert_eq!(from.slice(content, &to), "is\ni");

        assert_eq!(to.slice(content, &from), "is\ni");
    }

    #[test]
    fn test_slice_from_line_start() {
        let content = "This\nis\n- メカジキ - a\ntest";

        assert_eq!(
            Cursor::from_byte_offset(content, 4).slice_from_line_start(content),
            "This"
        );
        assert_eq!(
            Cursor::from_byte_offset(content, 5).slice_from_line_start(content),
            ""
        );
        assert_eq!(
            Cursor::from_byte_offset(content, 16).slice_from_line_start(content),
            "- メカ"
        );
    }

    #[test]
    fn test_slice_to_line_end() {
        let content = "This\nis\n- メカジキ - a\ntest";

        assert_eq!(
            Cursor::from_byte_offset(content, 4).slice_to_line_end(content),
            ""
        );
        assert_eq!(
            Cursor::from_byte_offset(content, 5).slice_to_line_end(content),
            "is"
        );
        assert_eq!(
            Cursor::from_byte_offset(content, 16).slice_to_line_end(content),
            "ジキ - a"
        );
    }

    #[test]
    fn test_find_line_start() {
        let content = "This\nis\n- a\ntest";

        for i in 0..content.len() {
            assert_eq!(
                Cursor::from_byte_offset(content, i).find_line_start(content, 1),
                Some(Cursor::from_byte_offset(content, 0))
            );
        }

        for i in 0..content.len() {
            assert_eq!(
                Cursor::from_byte_offset(content, i).find_line_start(content, 2),
                Some(Cursor::from_byte_offset(content, 5))
            );
        }

        for i in 0..content.len() {
            assert_eq!(
                Cursor::from_byte_offset(content, i).find_line_start(content, 3),
                Some(Cursor::from_byte_offset(content, 8))
            );
        }

        for i in 0..content.len() {
            assert_eq!(
                Cursor::from_byte_offset(content, i).find_line_start(content, 4),
                Some(Cursor::from_byte_offset(content, 12))
            );
        }

        for i in 0..content.len() {
            assert_eq!(
                Cursor::from_byte_offset(content, i).find_line_start(content, 5),
                Some(Cursor::from_byte_offset(content, 16))
            );
        }

        for i in 0..content.len() {
            assert_eq!(
                Cursor::from_byte_offset(content, i).find_line_start(content, 6),
                None
            );
        }
    }
}
