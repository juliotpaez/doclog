use crate::printer::{Printable, Printer, PrinterFormat};
use crate::LogLevel;
use smallvec::{smallvec, SmallVec};
use std::borrow::Cow;
use std::fmt::Display;
use yansi::Style;

/// A block that prints a formated text to the terminal.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct TextBlock<'a> {
    sections: SmallVec<[TextSection<'a>; 3]>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextSection<'a> {
    pub text: Cow<'a, str>,
    pub style: Style,
}

impl<'a> TextBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new empty [TextBlock].
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [TextBlock] with a plain text.
    #[inline(always)]
    pub fn new_plain(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            sections: smallvec![TextSection {
                text: text.into(),
                style: Style::new(),
            }],
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns whether the text block is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Returns the sections of the text block.
    #[inline(always)]
    pub fn get_sections(&self) -> &[TextSection<'a>] {
        &self.sections
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a plain text to the block.
    #[inline(always)]
    pub fn add_plain_text(self, text: impl Into<Cow<'a, str>>) -> Self {
        self.add_section(TextSection {
            text: text.into(),
            style: Style::new(),
        })
    }

    /// Adds a styled text to the block.
    #[inline(always)]
    pub fn add_styled_text(self, text: impl Into<Cow<'a, str>>, style: Style) -> Self {
        self.add_section(TextSection {
            text: text.into(),
            style,
        })
    }

    /// Adds a section to the block.
    #[inline]
    pub fn add_section(mut self, section: TextSection<'a>) -> Self {
        if section.text.is_empty() {
            return self;
        }

        self.sections.push(section);
        self
    }

    /// Makes this [TextBlock] to be single-lined.
    #[inline]
    pub fn single_lined(&self) -> Self {
        Self {
            sections: self
                .sections
                .iter()
                .map(|section| TextSection {
                    text: match &section.text {
                        Cow::Borrowed(v) => {
                            if memchr::memchr(b'\n', v.as_bytes()).is_some() {
                                Cow::Owned(section.text.replace('\n', " "))
                            } else {
                                Cow::Borrowed(*v)
                            }
                        }
                        Cow::Owned(v) => v.replace('\n', " ").into(),
                    },
                    style: section.style,
                })
                .collect(),
        }
    }

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> TextBlock<'static> {
        TextBlock {
            sections: self
                .sections
                .into_iter()
                .map(|painted| TextSection {
                    text: painted.text.into_owned().into(),
                    style: painted.style,
                })
                .collect(),
        }
    }
}

impl<'a> Printable<'a> for TextBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        for painted in &self.sections {
            printer.push_text_section(painted.clone());
        }
    }
}

impl<'a> Display for TextBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = Printer::new(LogLevel::trace(), PrinterFormat::Plain);
        self.print(&mut printer);
        printer.fmt(f, PrinterFormat::Plain)
    }
}

impl<'a> From<&'a str> for TextBlock<'a> {
    fn from(text: &'a str) -> Self {
        TextBlock::new_plain(text)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::blocks::TextBlock;
    use crate::printer::{Printable, PrinterFormat};
    use crate::LogLevel;
    use yansi::Style;

    #[test]
    fn test_plain() {
        let log = TextBlock::new()
            .add_styled_text("This is\na test", Style::new().bold().yellow())
            .add_plain_text("- plain")
            .add_styled_text(" - styled", Style::new().bold().red());
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Plain)
            .to_string();

        assert_eq!(text, "This is\na test- plain - styled");
    }

    #[test]
    fn test_styled() {
        yansi::disable();
        let log = TextBlock::new()
            .add_styled_text("This is\na test", Style::new().bold().yellow())
            .add_plain_text("- plain")
            .add_styled_text(" - styled", Style::new().bold().red());
        let text = log
            .print_to_string(LogLevel::error(), PrinterFormat::Styled)
            .to_string();

        println!("{}", text);
        assert_eq!(
            text,
            "\u{1b}[1;33mThis is\na test\u{1b}[0m- plain\u{1b}[1;31m - styled\u{1b}[0m"
        );
    }
}
