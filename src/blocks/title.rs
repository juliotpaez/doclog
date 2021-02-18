use chrono::{SecondsFormat, Utc};
use yansi::{Color, Style};

use crate::utils::text::indent_text;
use crate::Log;

/// A block that prints a title.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TitleBlock {
    message: String,
    show_date: bool,
    show_thread: bool,
}

impl TitleBlock {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(message: String, show_date: bool, show_thread: bool) -> TitleBlock {
        TitleBlock {
            message,
            show_date,
            show_thread,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The message of the block.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Whether to show the current date or not.
    pub fn show_date(&self) -> bool {
        self.show_date
    }

    /// Whether to show the current thread or not.
    pub fn show_thread(&self) -> bool {
        self.show_thread
    }

    // SETTERS ----------------------------------------------------------------

    pub fn set_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn set_show_date(&mut self, show_date: bool) {
        self.show_date = show_date;
    }

    pub fn set_show_thread(&mut self, show_thread: bool) {
        self.show_thread = show_thread;
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        let tag = log.level().tag();
        let header_size = tag.len() + 3;

        let date = if self.show_date {
            Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true))
        } else {
            None
        };

        let thread = if self.show_thread {
            Some(
                std::thread::current()
                    .name()
                    .unwrap_or("undefined")
                    .to_string(),
            )
        } else {
            None
        };

        let mut message = String::new();
        indent_text(
            self.message.as_str(),
            &mut message,
            " ".repeat(header_size).as_str(),
            false,
        );

        if in_ansi {
            buffer.push_str(
                Style::new(log.level().color())
                    .bold()
                    .paint(tag)
                    .to_string()
                    .as_str(),
            );
            buffer.push_str(" ");

            if let Some(date) = date {
                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint("at")
                        .to_string()
                        .as_str(),
                );
                buffer.push_str(" ");
                buffer.push_str(
                    Style::new(Color::Unset)
                        .bold()
                        .paint(date.as_str())
                        .to_string()
                        .as_str(),
                );
                buffer.push_str(" ");
            }

            if let Some(thread) = thread {
                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint("in")
                        .to_string()
                        .as_str(),
                );
                buffer.push_str(" ");
                buffer.push_str(
                    Style::new(log.level().color())
                        .bold()
                        .paint("thread")
                        .to_string()
                        .as_str(),
                );
                buffer.push_str(" ");
                buffer.push_str(
                    Style::new(Color::Unset)
                        .bold()
                        .paint(format!("\"{}\"", thread))
                        .to_string()
                        .as_str(),
                );
                buffer.push_str(" ");
            }

            buffer.push_str(
                Style::new(log.level().color())
                    .bold()
                    .paint("-")
                    .to_string()
                    .as_str(),
            );
            buffer.push_str(" ");
            buffer.push_str(message.as_str());
        } else {
            buffer.push_str(tag);
            buffer.push_str(" ");

            if let Some(date) = date {
                buffer.push_str("at ");
                buffer.push_str(date.as_str());
                buffer.push_str(" ");
            }

            if let Some(thread) = thread {
                buffer.push_str("in thread \"");
                buffer.push_str(thread.as_str());
                buffer.push_str("\" ");
            }

            buffer.push_str("- ");
            buffer.push_str(message.as_str());
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use crate::{Log, LogLevel};

    use super::*;

    #[test]
    fn test_plain() {
        // MESSAGE
        let log = Log::info().title_str("This is a\nmultiline\nmessage", false, false);
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!("info - This is a\n       multiline\n       message")
        );

        // MESSAGE + THREAD
        let thread = std::thread::current()
            .name()
            .unwrap_or("undefined")
            .to_string();
        let log = Log::info().title_str("This is a\nmultiline\nmessage", false, true);
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!(
                "info in thread \"{}\" - This is a\n       multiline\n       message",
                thread
            )
        );

        // MESSAGE + DATE
        let year_first_digit = Utc::today().to_string().chars().next().unwrap();
        let log = Log::info().title_str("This is a\nmultiline\nmessage", true, false);
        let text = log.to_plain_text();

        assert_eq!(
            text.split(year_first_digit).next().unwrap(),
            format!("info at ")
        );

        assert_eq!(
            text.split("Z").last().unwrap(),
            format!(" - This is a\n       multiline\n       message")
        );

        // MESSAGE + DATE + THREAD
        let log = Log::info().title_str("This is a\nmultiline\nmessage", true, true);
        let text = log.to_plain_text();

        assert_eq!(
            text.split(year_first_digit).next().unwrap(),
            format!("info at ")
        );

        assert_eq!(
            text.split("Z").last().unwrap(),
            format!(
                " in thread \"{}\" - This is a\n       multiline\n       message",
                thread
            )
        );
    }

    #[test]
    fn test_ansi() {
        // MESSAGE
        let log = Log::info().title_str("This is a\nmultiline\nmessage", false, false);
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} {} This is a\n       multiline\n       message",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(LogLevel::info().tag()),
                Style::new(LogLevel::info().color()).bold().paint("-")
            )
        );

        // MESSAGE + THREAD
        let thread = std::thread::current()
            .name()
            .unwrap_or("undefined")
            .to_string();
        let log = Log::info().title_str("This is a\nmultiline\nmessage", false, true);
        let text = log.to_ansi_text();

        assert_eq!(
            text,
            format!(
                "{} {} {} {} {} This is a\n       multiline\n       message",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(LogLevel::info().tag()),
                Style::new(LogLevel::info().color()).bold().paint("in"),
                Style::new(LogLevel::info().color()).bold().paint("thread"),
                Style::new(Color::Unset)
                    .bold()
                    .paint(format!("\"{}\"", thread)),
                Style::new(LogLevel::info().color()).bold().paint("-"),
            )
        );

        // MESSAGE + DATE
        let year = Utc::today().year().to_string();
        let log = Log::info().title_str("This is a\nmultiline\nmessage", true, false);
        let text = log.to_ansi_text();

        assert_eq!(
            text.split(&year).next().unwrap(),
            format!(
                "{} {} {}",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(LogLevel::info().tag()),
                Style::new(LogLevel::info().color()).bold().paint("at"),
                Style::new(Color::Unset)
                    .bold()
                    .paint("xxxx")
                    .to_string()
                    .split("xxxx")
                    .next()
                    .unwrap(),
            )
        );

        assert_eq!(
            text.split("Z").last().unwrap(),
            format!(
                "{} {} This is a\n       multiline\n       message",
                Style::new(Color::Unset)
                    .bold()
                    .paint("xxxx")
                    .to_string()
                    .split("xxxx")
                    .last()
                    .unwrap(),
                Style::new(LogLevel::info().color()).bold().paint("-"),
            )
        );

        // MESSAGE + DATE + THREAD
        let thread = std::thread::current()
            .name()
            .unwrap_or("undefined")
            .to_string();
        let log = Log::info().title_str("This is a\nmultiline\nmessage", true, true);
        let text = log.to_ansi_text();

        assert_eq!(
            text.split(&year).next().unwrap(),
            format!(
                "{} {} {}",
                Style::new(LogLevel::info().color())
                    .bold()
                    .paint(LogLevel::info().tag()),
                Style::new(LogLevel::info().color()).bold().paint("at"),
                Style::new(Color::Unset)
                    .bold()
                    .paint("xxxx")
                    .to_string()
                    .split("xxxx")
                    .next()
                    .unwrap(),
            )
        );

        assert_eq!(
            text.split("Z").last().unwrap(),
            format!(
                "{} {} {} {} {} This is a\n       multiline\n       message",
                Style::new(Color::Unset)
                    .bold()
                    .paint("xxxx")
                    .to_string()
                    .split("xxxx")
                    .last()
                    .unwrap(),
                Style::new(LogLevel::info().color()).bold().paint("in"),
                Style::new(LogLevel::info().color()).bold().paint("thread"),
                Style::new(Color::Unset)
                    .bold()
                    .paint(format!("\"{}\"", thread)),
                Style::new(LogLevel::info().color()).bold().paint("-"),
            )
        );
    }
}
