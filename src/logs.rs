use std::borrow::Cow;
use std::fs;
use std::option::Option::Some;
use std::path::Path;

use crate::blocks::{
    DocumentBlock, IndentBlock, LogBlock, NoteBlock, PlainTextBlock, SeparatorBlock, StackBlock,
    TagBlock, TitleBlock,
};
use crate::{is_ansi_supported, LogLevel};

/// A configured log.
#[derive(Debug, Clone)]
pub struct Log<'a> {
    level: LogLevel,
    blocks: Vec<LogBlock<'a>>,
    cause: Option<Box<Log<'a>>>,
}

impl<'a> Log<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new log.
    pub fn new(level: LogLevel) -> Log<'a> {
        Log {
            level,
            blocks: Vec::new(),
            cause: None,
        }
    }

    /// Builds a new log with a trace level.
    pub fn trace() -> Log<'a> {
        Self::new(LogLevel::trace())
    }

    /// Builds a new log with a debug level.
    pub fn debug() -> Log<'a> {
        Self::new(LogLevel::debug())
    }

    /// Builds a new log with a info level.
    pub fn info() -> Log<'a> {
        Self::new(LogLevel::info())
    }

    /// Builds a new log with a warn level.
    pub fn warn() -> Log<'a> {
        Self::new(LogLevel::warn())
    }

    /// Builds a new log with a error level.
    pub fn error() -> Log<'a> {
        Self::new(LogLevel::error())
    }

    // GETTERS ----------------------------------------------------------------

    pub fn level(&self) -> &LogLevel {
        &self.level
    }

    pub fn blocks(&self) -> &Vec<LogBlock<'a>> {
        &self.blocks
    }

    pub fn cause(&self) -> &Option<Box<Log<'a>>> {
        &self.cause
    }

    // METHODS ----------------------------------------------------------------

    /// Logs in the console an ANSI or plain text depending on whether
    /// the ANSI colors are supported in the executing terminal or not.
    pub fn log(&self) {
        println!("{}", self.to_text());
    }

    /// Appends the log into the specified file as plain text.
    pub fn append_to_file(&self, file: &Path) -> std::io::Result<()> {
        let content = self.to_plain_text();
        fs::write(file, content)
    }

    /// Logs in the console detecting whether the ANSI colors are supported
    /// in the executing terminal or not and  appends the log into the specified
    /// file as plain text.
    pub fn log_and_append_to_file(&self, file: &Path) -> std::io::Result<()> {
        self.log();
        self.append_to_file(file)
    }

    /// Returns the log as a plain text.
    pub fn to_plain_text(&self) -> String {
        let mut buffer = String::new();
        self.to_text_internal(false, &mut buffer);
        buffer
    }

    /// Returns the log as an ANSI text.
    pub fn to_ansi_text(&self) -> String {
        let mut buffer = String::new();
        self.to_text_internal(true, &mut buffer);
        buffer
    }

    /// Returns the log as an ANSI or plain text depending on whether
    /// the ANSI colors are supported in the executing terminal or not.
    pub fn to_text(&self) -> String {
        let mut buffer = String::new();
        self.to_text_internal(is_ansi_supported(), &mut buffer);
        buffer
    }

    pub fn to_text_internal(&self, in_ansi: bool, buffer: &mut String) {
        // Print blocks.
        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                buffer.push('\n');
            }

            block.to_text(self, in_ansi, buffer);
        }

        // Print cause.
        if let Some(cause) = &self.cause {
            buffer.push('\n');
            cause.to_text_internal(in_ansi, buffer);
        }
    }
}

impl<'a> Log<'a> {
    // METHODS ----------------------------------------------------------------

    /// Adds a new block.
    pub fn add_block(mut self, block: LogBlock<'a>) -> Self {
        self.blocks.push(block);
        self
    }

    /// Adds a title block.
    pub fn title(
        self,
        message: impl Into<Cow<'a, str>>,
        show_date: bool,
        show_thread: bool,
    ) -> Self {
        self.add_block(LogBlock::Title(TitleBlock::new(
            message.into(),
            show_date,
            show_thread,
        )))
    }

    /// Adds a new plain text block.
    pub fn plain_text(self, text: impl Into<Cow<'a, str>>) -> Self {
        self.add_block(LogBlock::PlainText(PlainTextBlock::new(text.into())))
    }

    /// Adds a document block.
    pub fn document<F>(self, content: impl Into<Cow<'a, str>>, builder: F) -> Self
    where
        F: FnOnce(DocumentBlock<'a>) -> DocumentBlock<'a>,
    {
        let document = DocumentBlock::new(content.into());
        let document = builder(document);
        self.add_block(LogBlock::Document(document))
    }

    /// Adds a separator block.
    pub fn separator(self, width: usize) -> Self {
        self.add_block(LogBlock::Separator(SeparatorBlock::new(width)))
    }

    /// Adds an indent block.
    pub fn indent<F>(self, indent: usize, builder: F) -> Self
    where
        F: FnOnce(Log<'a>) -> Log<'a>,
    {
        let new_log = Log::new(self.level.clone());
        let new_log = builder(new_log);
        self.add_block(LogBlock::Indent(IndentBlock::new(
            indent,
            Box::new(new_log),
        )))
    }

    /// Adds a tag block.
    pub fn tag(self, tag: impl Into<Cow<'a, str>>) -> Self {
        self.add_block(LogBlock::Tag(TagBlock::new(tag.into())))
    }

    /// Adds a note block.
    pub fn note(self, title: impl Into<Cow<'a, str>>, message: impl Into<Cow<'a, str>>) -> Self {
        self.add_block(LogBlock::Note(NoteBlock::new(title.into(), message.into())))
    }

    /// Adds a stack block.
    pub fn stack<F>(self, message: impl Into<Cow<'a, str>>, builder: F) -> Self
    where
        F: FnOnce(StackBlock) -> StackBlock,
    {
        let stack = StackBlock::new(message.into());
        let stack = builder(stack);
        self.add_block(LogBlock::Stack(stack))
    }

    /// Adds an indent block.
    pub fn set_cause<F>(mut self, builder: F) -> Self
    where
        F: FnOnce(Log) -> Log,
    {
        let new_log = Log::new(self.level.clone());
        let new_log = builder(new_log);
        self.cause = Some(Box::new(new_log));
        self
    }
}
