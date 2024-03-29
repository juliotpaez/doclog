use std::borrow::Cow;

use chrono::{SecondsFormat, Utc};
use yansi::Color;

use crate::utils::text::{color_bold_if, indent_text};
use crate::Log;

/// A block that prints a title.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TitleBlock<'a> {
    message: Cow<'a, str>,
    show_date: bool,
    show_thread: bool,
}

impl<'a> TitleBlock<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(message: Cow<'a, str>, show_date: bool, show_thread: bool) -> TitleBlock {
        TitleBlock {
            message,
            show_date,
            show_thread,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The message of the block.
    pub fn get_message(&self) -> &Cow<'a, str> {
        &self.message
    }

    /// Whether to show the current date or not.
    pub fn get_show_date(&self) -> bool {
        self.show_date
    }

    /// Whether to show the current thread or not.
    pub fn get_show_thread(&self) -> bool {
        self.show_thread
    }

    // SETTERS ----------------------------------------------------------------

    pub fn message(mut self, message: Cow<'a, str>) -> Self {
        self.message = message;
        self
    }

    pub fn show_date(&mut self, show_date: bool) {
        self.show_date = show_date;
    }

    pub fn show_thread(&mut self, show_thread: bool) {
        self.show_thread = show_thread;
    }

    // METHODS ----------------------------------------------------------------

    pub(crate) fn to_text(&self, log: &Log<'a>, in_ansi: bool, buffer: &mut String) {
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

        let message = indent_text(
            self.message.as_ref(),
            " ".repeat(header_size).as_str(),
            false,
        );

        buffer.push_str(&color_bold_if(
            tag.to_string(),
            log.level().color(),
            in_ansi,
        ));
        buffer.push(' ');

        if let Some(date) = date {
            buffer.push_str(&color_bold_if(
                "at".to_string(),
                log.level().color(),
                in_ansi,
            ));
            buffer.push(' ');
            buffer.push_str(&color_bold_if(date, Color::Unset, in_ansi));
            buffer.push(' ');
        }

        if let Some(thread) = thread {
            buffer.push_str(&color_bold_if(
                "in".to_string(),
                log.level().color(),
                in_ansi,
            ));
            buffer.push(' ');
            buffer.push_str(&color_bold_if(
                "thread".to_string(),
                log.level().color(),
                in_ansi,
            ));
            buffer.push(' ');
            buffer.push_str(&color_bold_if(
                format!("\"{}\"", thread),
                Color::Unset,
                in_ansi,
            ));
            buffer.push(' ');
        }

        buffer.push_str(&color_bold_if(
            "-".to_string(),
            log.level().color(),
            in_ansi,
        ));
        buffer.push(' ');
        buffer.push_str(message.as_str());
    }

    pub fn make_owned<'b>(&self) -> TitleBlock<'b> {
        TitleBlock {
            message: Cow::Owned(self.message.to_string()),
            show_date: self.show_date,
            show_thread: self.show_thread,
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use yansi::Style;

    use crate::{Log, LogLevel};

    use super::*;

    #[test]
    fn test_plain() {
        // MESSAGE
        let log = Log::info().title("This is a\nmultiline\nmessage", false, false);
        let text = log.to_plain_text();

        assert_eq!(text, "info - This is a\n       multiline\n       message");

        // MESSAGE + THREAD
        let thread = std::thread::current()
            .name()
            .unwrap_or("undefined")
            .to_string();
        let log = Log::info().title("This is a\nmultiline\nmessage", false, true);
        let text = log.to_plain_text();

        assert_eq!(
            text,
            format!(
                "info in thread \"{}\" - This is a\n       multiline\n       message",
                thread
            )
        );

        // MESSAGE + DATE
        let year_first_digit = Utc::now().date_naive().to_string().chars().next().unwrap();
        let log = Log::info().title("This is a\nmultiline\nmessage", true, false);
        let text = log.to_plain_text();

        assert_eq!(text.split(year_first_digit).next().unwrap(), "info at ");

        assert_eq!(
            text.split('Z').last().unwrap(),
            " - This is a\n       multiline\n       message"
        );

        // MESSAGE + DATE + THREAD
        let log = Log::info().title("This is a\nmultiline\nmessage", true, true);
        let text = log.to_plain_text();

        assert_eq!(text.split(year_first_digit).next().unwrap(), "info at ");

        assert_eq!(
            text.split('Z').last().unwrap(),
            format!(
                " in thread \"{}\" - This is a\n       multiline\n       message",
                thread
            )
        );
    }

    #[test]
    fn test_ansi() {
        // MESSAGE
        let log = Log::info().title("This is a\nmultiline\nmessage", false, false);
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
        let log = Log::info().title("This is a\nmultiline\nmessage", false, true);
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
        let year = Utc::now().year().to_string();
        let log = Log::info().title("This is a\nmultiline\nmessage", true, false);
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
            text.split('Z').last().unwrap(),
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
        let log = Log::info().title("This is a\nmultiline\nmessage", true, true);
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
            text.split('Z').last().unwrap(),
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
