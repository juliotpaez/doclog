use memchr::memchr_iter;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::iter::once;
use yansi::Style;

#[derive(Debug, Clone)]
pub struct Printer<'a> {
    pub format: PrinterFormat,
    pub elements: Vec<(&'a str, Style)>,
}

impl<'a> Printer<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new [Printer] for the given format.
    pub fn new(format: PrinterFormat) -> Self {
        Self {
            format,
            elements: Vec::new(),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Appends another [Printer] to this one.
    pub fn append(&mut self, other: Printer<'a>) {
        self.elements.extend(other.elements);
    }

    /// Pushes a styled string to the printer.
    pub fn push_plain_str(&mut self, text: &'a str) {
        self.elements.push((text, Style::new()));
    }

    /// Pushes a styled string to the printer.
    pub fn push_styled_str(&mut self, text: &'a str, style: Style) {
        self.elements.push((text, style));
    }

    /// Indents the content of this [Printer] with the content of another [Printer].
    pub fn indent(&mut self, other: &Printer<'a>, indent_first_line: bool) {
        if other.elements.is_empty() {
            return;
        }

        let mut self_index = if indent_first_line {
            self.elements.splice(0..0, other.elements.iter().cloned());
            other.elements.iter().len()
        } else {
            0
        };

        while self_index < self.elements.len() {
            let (self_content, self_style) = self.elements[self_index];

            let mut start_index = 0;
            let mut increase = 0;
            let iterator = memchr_iter(b'\n', self_content.as_bytes())
                .chain(once(self_content.len()))
                .flat_map(|position| {
                    let is_first = start_index == 0;
                    let current_line = if position >= self_content.len() {
                        &self_content[start_index..]
                    } else {
                        &self_content[start_index..=position]
                    };
                    start_index = position + 1;

                    (0..if is_first {
                        0
                    } else {
                        increase += other.elements.len();
                        other.elements.len()
                    })
                        .map(|v| {
                            let (prefix, style) = other.elements[v];
                            (prefix, style)
                        })
                        .chain(
                            once(if current_line.is_empty() {
                                None
                            } else {
                                increase += 1;
                                Some((
                                    current_line,
                                    // Optimization to match the style of the indentation when no visual content is present.
                                    // This way we avoid unnecessary ANSI codes.
                                    if current_line.chars().all(|c| char::is_ascii_whitespace(&c)) {
                                        other.elements.last().unwrap().1
                                    } else {
                                        self_style
                                    },
                                ))
                            })
                            .flatten(),
                        )
                });
            self.elements.splice(self_index..self_index + 1, iterator);
            self_index += increase;
        }
    }

    /// Implement this to provide custom formatting for this type.
    pub fn fmt(&self, fmt: &mut Formatter<'_>, format: PrinterFormat) -> fmt::Result {
        match format {
            PrinterFormat::Plain => {
                for (text, _) in &self.elements {
                    write!(fmt, "{}", text)?;
                }
            }
            PrinterFormat::Default | PrinterFormat::Styled => {
                let mut prev_style: Option<&Style> = None;

                for (text, style) in &self.elements {
                    // Print previos suffix and current prefix only if the style is different.
                    if let Some(prev_style) = prev_style.take() {
                        if prev_style != style {
                            prev_style.fmt_suffix(fmt)?;
                            style.fmt_prefix(fmt)?;
                        }
                    } else {
                        style.fmt_prefix(fmt)?;
                    }

                    write!(fmt, "{}", text)?;
                    prev_style = Some(style);
                }

                if let Some(prev_style) = prev_style.take() {
                    prev_style.fmt_suffix(fmt)?;
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

pub trait Printable {
    /// Maps the content of this type to a [Printer].
    fn print<'a>(&'a self, printer: &mut Printer<'a>);

    /// Converts the content of this type to a string.
    fn print_to_string(&self, format: PrinterFormat) -> String {
        let mut printer = Printer::new(format);
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
        let mut base = Printer::new(PrinterFormat::Plain);
        base.push_styled_str("this\nis\n\na\ntest\n", Style::new().bold().yellow());
        base.push_plain_str("::a\nplain\ntest\n");

        let mut other = Printer::new(PrinterFormat::Styled);
        other.push_styled_str("--", Style::new().bold().blue());
        other.push_styled_str(">>", Style::new().bold().green());

        // Generate result.
        base.indent(&other, true);
        let result = format!("{}", base);

        println!("{}", result);
        assert_eq!(
            result,
            "-->>this\n-->>is\n-->>\n-->>a\n-->>test\n-->>::a\n-->>plain\n-->>test\n-->>"
        );
    }

    #[test]
    fn test_indent_styled() {
        let mut base = Printer::new(PrinterFormat::Styled);
        base.push_styled_str("this\nis\n\na\ntest\n", Style::new().bold().yellow());
        base.push_plain_str("::a\nplain\ntest\n");

        let mut other = Printer::new(PrinterFormat::Styled);
        other.push_styled_str("--", Style::new().bold().blue());
        other.push_styled_str(">>", Style::new().bold().green());

        // Generate result.
        base.indent(&other, true);
        let result = format!("{}", base);

        println!("{}", result);
        assert_eq!(
            result,
            "\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33mthis\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33mis\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33ma\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m\u{1b}[1;33mtest\n\u{1b}[0m\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m::a\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0mplain\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0mtest\n\u{1b}[1;34m--\u{1b}[0m\u{1b}[1;32m>>\u{1b}[0m"
        );
    }
}
