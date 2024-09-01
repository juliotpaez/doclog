use crate::blocks::TextSection;
use crate::LogLevel;
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Display, Formatter};
use yansi::Style;

#[derive(Debug, Clone)]
pub struct Printer<'a> {
    pub level: LogLevel,
    pub format: PrinterFormat,
    pub lines: Vec<Vec<TextSection<'a>>>,
}

impl<'a> Printer<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new [Printer] for the given initial configuration.
    pub fn new(level: LogLevel, format: PrinterFormat) -> Self {
        Self {
            level,
            format,
            lines: Vec::new(),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Derives a new [Printer] from this one.
    pub fn derive<'b>(&self) -> Printer<'b> {
        Printer {
            level: self.level,
            format: self.format,
            lines: Vec::new(),
        }
    }

    /// Appends another [Printer] to this one.
    pub fn append(&mut self, other: Printer<'a>) {
        if other.lines.is_empty() {
            return;
        }

        if self.lines.is_empty() {
            self.lines = other.lines;
            return;
        }

        let mut iter = other.lines.into_iter();

        self.lines.last_mut().unwrap().extend(iter.next().unwrap());
        self.lines.extend(iter);
    }

    /// Appends another [Printer] to this one.
    pub fn append_lines(&mut self, other: Printer<'a>) {
        self.lines.extend(other.lines);
    }

    /// Pushes a text section to the printer.
    pub fn push_text_section(&mut self, element: TextSection<'a>) {
        if element.text.is_empty() {
            return;
        }

        match element.text {
            Cow::Borrowed(text) => {
                for (i, line) in text.lines().enumerate() {
                    // Push to the last if first.
                    if let (0, Some(last)) = (i, self.lines.last_mut()) {
                        if !line.is_empty() {
                            last.push(TextSection {
                                text: Cow::Borrowed(line),
                                style: element.style,
                            });
                        }
                        continue;
                    }

                    if line.is_empty() {
                        self.lines.push(vec![]);
                    } else {
                        self.lines.push(vec![TextSection {
                            text: Cow::Borrowed(line),
                            style: element.style,
                        }]);
                    }
                }

                if text.ends_with('\n') {
                    self.lines.push(vec![]);
                }
            }
            Cow::Owned(text) => {
                for (i, line) in text.lines().enumerate() {
                    // Push to the last if first.
                    if let (0, Some(last)) = (i, self.lines.last_mut()) {
                        if !line.is_empty() {
                            last.push(TextSection {
                                text: Cow::Owned(line.to_string()),
                                style: element.style,
                            });
                        }
                        continue;
                    }

                    if line.is_empty() {
                        self.lines.push(vec![]);
                    } else {
                        self.lines.push(vec![TextSection {
                            text: Cow::Owned(line.to_string()),
                            style: element.style,
                        }]);
                    }
                }

                if text.ends_with('\n') {
                    self.lines.push(vec![]);
                }
            }
        }
    }

    /// Pushes a styled string to the printer.
    pub fn push_plain_text(&mut self, text: impl Into<Cow<'a, str>>) {
        self.push_text_section(TextSection {
            text: text.into(),
            style: Style::new(),
        });
    }

    /// Pushes a styled string to the printer.
    pub fn push_styled_text(&mut self, text: impl Into<Cow<'a, str>>, style: Style) {
        self.push_text_section(TextSection {
            text: text.into(),
            style,
        });
    }

    /// Indents the content of this [Printer] with a list of text sections.
    pub fn indent(&mut self, sections: &[TextSection<'a>], indent_first_line: bool) {
        if sections.is_empty() {
            return;
        }

        for line in self
            .lines
            .iter_mut()
            .skip(if indent_first_line { 0 } else { 1 })
        {
            line.splice(0..0, sections.iter().cloned());
        }
    }

    /// Implement this to provide custom formatting for this type.
    pub fn fmt(&self, fmt: &mut Formatter<'_>, format: PrinterFormat) -> fmt::Result {
        let styled = match format {
            PrinterFormat::Default => yansi::is_enabled(),
            PrinterFormat::Plain => false,
            PrinterFormat::Styled => true,
        };

        if styled {
            let mut prev_style: Option<&Style> = None;

            for (i, line) in self.lines.iter().enumerate() {
                if i != 0 {
                    writeln!(fmt)?;
                }

                for section in line {
                    if section.style.enabled() {
                        let all_whitespace =
                            section.text.chars().all(|c| char::is_ascii_whitespace(&c));

                        // Print previous suffix and current prefix only if the style is different.
                        if !all_whitespace {
                            if let Some(prev_style) = prev_style.take() {
                                if prev_style != &section.style {
                                    prev_style.fmt_suffix(fmt)?;
                                    section.style.fmt_prefix(fmt)?;
                                }
                            } else {
                                section.style.fmt_prefix(fmt)?;
                            }
                        }

                        write!(fmt, "{}", section.text)?;

                        if !all_whitespace {
                            prev_style = Some(&section.style);
                        }
                    } else {
                        // Print previous suffix.
                        if let Some(prev_style) = prev_style.take() {
                            prev_style.fmt_suffix(fmt)?;
                        }

                        write!(fmt, "{}", section.text)?;
                    }
                }
            }

            if let Some(prev_style) = prev_style.take() {
                prev_style.fmt_suffix(fmt)?;
            }
        } else {
            for (i, line) in self.lines.iter().enumerate() {
                if i != 0 {
                    writeln!(fmt)?;
                }

                for section in line {
                    write!(fmt, "{}", section.text)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> Display for Printer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt(f, self.format)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrinterFormat {
    /// Format depends on system settings.
    Default,

    /// Plain text format.
    Plain,

    /// Styled text format.
    Styled,
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub trait Printable<'a> {
    /// Maps the content of this type to a [Printer].
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's;

    /// Converts the content of this type to a string.
    fn print_to_string(&self, level: LogLevel, format: PrinterFormat) -> String {
        let mut printer = Printer::new(level, format);
        self.print(&mut printer);
        format!("{}", printer)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_plain() {
        let mut base = Printer::new(LogLevel::error(), PrinterFormat::Plain);
        base.push_styled_text("this\nis\n\na\ntest\n", Style::new().bold().yellow());
        base.push_plain_text("::a\nplain\ntest\n");

        let indent = vec![
            TextSection {
                text: Cow::Borrowed("--"),
                style: Style::new().bold().blue(),
            },
            TextSection {
                text: Cow::Borrowed(">>"),
                style: Style::new().bold().green(),
            },
        ];

        // Generate result.
        base.indent(&indent, true);
        let result = format!("{}", base);

        println!("{}", result);
        assert_eq!(
            result,
            "-->>this\n-->>is\n-->>\n-->>a\n-->>test\n-->>::a\n-->>plain\n-->>test\n-->>"
        );
    }

    #[test]
    fn test_indent_plain_skip_first() {
        let mut base = Printer::new(LogLevel::error(), PrinterFormat::Plain);
        base.push_styled_text("this\nis\n\na\ntest", Style::new().bold().yellow());
        base.push_plain_text("::a\nplain\ntest\n");

        let indent = vec![
            TextSection {
                text: Cow::Borrowed("--"),
                style: Style::new().bold().blue(),
            },
            TextSection {
                text: Cow::Borrowed(">>"),
                style: Style::new().bold().green(),
            },
        ];

        // Generate result.
        base.indent(&indent, false);
        let result = format!("{}", base);

        println!("{}", result);
        assert_eq!(
            result,
            "this\n-->>is\n-->>\n-->>a\n-->>test::a\n-->>plain\n-->>test\n-->>"
        );
    }

    #[test]
    fn test_indent_styled() {
        let mut base = Printer::new(LogLevel::error(), PrinterFormat::Styled);
        base.push_styled_text("this\nis\n\na\ntest\n", Style::new().bold().yellow());
        base.push_plain_text("::a\nplain\ntest\n");

        let indent = vec![
            TextSection {
                text: Cow::Borrowed("--"),
                style: Style::new().bold().blue(),
            },
            TextSection {
                text: Cow::Borrowed(">>"),
                style: Style::new().bold().green(),
            },
        ];

        // Generate result.
        base.indent(&indent, true);
        let result = format!("{}", base);

        println!("{}", result);
        assert_eq!(
            result,
            "\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33mthis\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33mis\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33ma\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33mtest\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m::a\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0mplain\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0mtest\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m"
        );
    }

    #[test]
    fn test_indent_styled_skip_first() {
        let mut base = Printer::new(LogLevel::error(), PrinterFormat::Styled);
        base.push_styled_text("this\nis\n\na\ntest", Style::new().bold().yellow());
        base.push_plain_text("::a\nplain\ntest\n");

        let indent = vec![
            TextSection {
                text: Cow::Borrowed("--"),
                style: Style::new().bold().blue(),
            },
            TextSection {
                text: Cow::Borrowed(">>"),
                style: Style::new().bold().green(),
            },
        ];

        // Generate result.
        base.indent(&indent, false);
        let result = format!("{}", base);

        println!("{}", result);
        assert_eq!(
            result,
            "\u{1b}[1;33mthis\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33mis\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33ma\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33mtest\u{1b}[0m::a\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0mplain\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0mtest\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m"
        );
    }
}
