use crate::LogLevel;
use memchr::memchr_iter;
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::once;
use yansi::Style;

#[derive(Debug, Clone)]
pub struct Printer<'a> {
    pub level: LogLevel,
    pub format: PrinterFormat,
    pub elements: Vec<PaintedElement<'a>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PaintedElement<'a> {
    pub text: Cow<'a, str>,
    pub style: Style,
}

impl<'a> Printer<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new [Printer] for the given initial configuration.
    pub fn new(level: LogLevel, format: PrinterFormat) -> Self {
        Self {
            level,
            format,
            elements: Vec::new(),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Derives a new [Printer] from this one.
    pub fn derive(&self) -> Self {
        Self {
            level: self.level,
            format: self.format,
            elements: Vec::new(),
        }
    }

    /// Appends another [Printer] to this one.
    pub fn append(&mut self, other: Printer<'a>) {
        self.elements.extend(other.elements);
    }

    /// Pushes a painted element to the printer.
    pub fn push_painted_element(&mut self, element: PaintedElement<'a>) {
        self.elements.push(element);
    }

    /// Pushes a styled string to the printer.
    pub fn push_plain_text(&mut self, text: impl Into<Cow<'a, str>>) {
        self.push_painted_element(PaintedElement {
            text: text.into(),
            style: Style::new(),
        });
    }

    /// Pushes a styled string to the printer.
    pub fn push_styled_text(&mut self, text: impl Into<Cow<'a, str>>, style: Style) {
        self.push_painted_element(PaintedElement {
            text: text.into(),
            style,
        });
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
            let PaintedElement {
                text: self_text,
                style: self_style,
            } = &self.elements[self_index];

            let mut start_index = 0;
            let mut increase = 0;
            let iterator = memchr_iter(b'\n', self_text.as_bytes())
                .chain(once(self_text.len()))
                .flat_map(|position| {
                    let is_first = start_index == 0;
                    let current_line = match self_text {
                        Cow::Borrowed(self_content) => {
                            if position >= self_content.len() {
                                Cow::Borrowed(&self_content[start_index..])
                            } else {
                                Cow::Borrowed(&self_content[start_index..=position])
                            }
                        }
                        Cow::Owned(self_content) => {
                            if position >= self_content.len() {
                                Cow::Owned(self_content[start_index..].to_string())
                            } else {
                                Cow::Owned(self_content[start_index..=position].to_string())
                            }
                        }
                    };
                    start_index = position + 1;

                    (0..if is_first {
                        0
                    } else {
                        increase += other.elements.len();
                        other.elements.len()
                    })
                        .map(|v| other.elements[v].clone())
                        .chain(
                            once(if current_line.is_empty() {
                                None
                            } else {
                                increase += 1;
                                Some({
                                    let all_whitespace =
                                        current_line.chars().all(|c| char::is_ascii_whitespace(&c));

                                    PaintedElement {
                                        text: current_line,
                                        // Optimization to match the style of the indentation when no visual content is present.
                                        // This way we avoid unnecessary ANSI codes.
                                        style: if all_whitespace {
                                            other.elements.last().unwrap().style
                                        } else {
                                            *self_style
                                        },
                                    }
                                })
                            })
                            .flatten(),
                        )
                });
            let v = iterator.collect::<Vec<_>>();
            self.elements.splice(self_index..self_index + 1, v);
            self_index += increase;
        }
    }

    /// Implement this to provide custom formatting for this type.
    pub fn fmt(&self, fmt: &mut Formatter<'_>, format: PrinterFormat) -> fmt::Result {
        match format {
            PrinterFormat::Plain => {
                for painted in &self.elements {
                    write!(fmt, "{}", painted.text)?;
                }
            }
            PrinterFormat::Default | PrinterFormat::Styled => {
                let mut prev_style: Option<&Style> = None;

                for painted in &self.elements {
                    // Print previos suffix and current prefix only if the style is different.
                    if let Some(prev_style) = prev_style.take() {
                        if prev_style != &painted.style {
                            prev_style.fmt_suffix(fmt)?;
                            painted.style.fmt_prefix(fmt)?;
                        }
                    } else {
                        painted.style.fmt_prefix(fmt)?;
                    }

                    write!(fmt, "{}", painted.text)?;

                    prev_style = Some(&painted.style);
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

        let mut other = Printer::new(LogLevel::error(), PrinterFormat::Styled);
        other.push_styled_text("--", Style::new().bold().blue());
        other.push_styled_text(">>", Style::new().bold().green());

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
        let mut base = Printer::new(LogLevel::error(), PrinterFormat::Styled);
        base.push_styled_text("this\nis\n\na\ntest\n", Style::new().bold().yellow());
        base.push_plain_text("::a\nplain\ntest\n");

        let mut other = Printer::new(LogLevel::error(), PrinterFormat::Styled);
        other.push_styled_text("--", Style::new().bold().blue());
        other.push_styled_text(">>", Style::new().bold().green());

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
