use std::borrow::Cow;

const N_NEWLINES: usize = 32;
const N_SPACES: usize = 128;
const WS: &str =
    "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n                                                                                                                                ";
const _: () = {
    assert!(WS.len() == N_NEWLINES + N_SPACES);
    assert!(WS.as_bytes()[N_NEWLINES - 1] == b'\n');
    assert!(WS.as_bytes()[N_NEWLINES] == b' ');
};

/// Builds a string with `count` spaces.
pub fn build_space_string(count: usize) -> Cow<'static, str> {
    if count < N_SPACES {
        Cow::Borrowed(&WS[N_NEWLINES..N_NEWLINES + count])
    } else {
        Cow::Owned(" ".repeat(count))
    }
}

/// Builds a string with `newline_count` new lines followed by `space_count` spaces.
pub fn build_whitespace_string(newline_count: usize, space_count: usize) -> Cow<'static, str> {
    if newline_count < N_NEWLINES && space_count < N_SPACES {
        let start = N_NEWLINES - newline_count;
        Cow::Borrowed(&WS[start..start + newline_count + space_count])
    } else {
        Cow::Owned(format!(
            "{}{}",
            "\n".repeat(newline_count),
            " ".repeat(space_count)
        ))
    }
}
