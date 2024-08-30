use crate::printer::{Printable, Printer};
pub use header::*;
pub use note::*;
// pub use document::*;
pub use prefix::*;
pub use separator::*;
pub use stack::*;
pub use stack_trace::*;
pub use text::*;

// mod document;
mod header;
mod note;
mod prefix;
mod separator;
mod stack;
mod stack_trace;
mod text;

/// A block log.
#[derive(Debug, Clone)]
pub enum LogBlock<'a> {
    // Basic blocks.
    Text(TextBlock<'a>),
    Prefix(PrefixBlock<'a>),

    // TODO
    // Custom blocks.
    Separator(SeparatorBlock),
    Header(HeaderBlock<'a>),
    Note(NoteBlock<'a>),
    Stack(StackBlock<'a>),
    // Document(DocumentBlock<'a>),
}

impl<'a> LogBlock<'a> {
    // METHODS ----------------------------------------------------------------

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> LogBlock<'static> {
        match self {
            // Basic blocks.
            LogBlock::Text(v) => LogBlock::Text(v.make_owned()),
            LogBlock::Prefix(v) => LogBlock::Prefix(v.make_owned()),

            // Custom blocks.
            LogBlock::Separator(v) => LogBlock::Separator(v),
            LogBlock::Header(v) => LogBlock::Header(v.make_owned()),
            LogBlock::Note(v) => LogBlock::Note(v.make_owned()),
            LogBlock::Stack(v) => LogBlock::Stack(v.make_owned()),
        }
    }
}

impl<'a> From<TextBlock<'a>> for LogBlock<'a> {
    fn from(block: TextBlock<'a>) -> Self {
        LogBlock::Text(block)
    }
}

impl<'a> Printable<'a> for LogBlock<'a> {
    fn print<'s>(&'s self, printer: &mut Printer<'a>)
    where
        'a: 's,
    {
        match self {
            // Basic blocks.
            LogBlock::Text(v) => v.print(printer),
            LogBlock::Prefix(v) => v.print(printer),

            // Custom blocks.
            LogBlock::Separator(v) => v.print(printer),
            LogBlock::Header(v) => v.print(printer),
            LogBlock::Note(v) => v.print(printer),
            LogBlock::Stack(v) => v.print(printer),
        }
    }
}
