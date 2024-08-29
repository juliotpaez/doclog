// pub use document::*;
pub use prefix::*;
// pub use note::*;
// pub use separator::*;
// pub use stack::*;
// pub use stack_trace::*;
// pub use tag::*;
use crate::printer::{Printable, Printer};
pub use text::*;
// pub use title::*;

// mod document;
mod prefix;
// mod note;
// mod separator;
// mod stack;
// mod stack_trace;
// mod tag;
mod text;
// mod title;

/// A block log.
#[derive(Debug, Clone)]
pub enum LogBlock<'a> {
    // Basic blocks.
    Text(TextBlock<'a>),
    Prefix(PrefixBlock<'a>),
    // TODO
    // Custom blocks.
    // Title(TitleBlock<'a>),
    // Document(DocumentBlock<'a>),
    // Separator(SeparatorBlock),
    // Stack(StackBlock<'a>),
    // Tag(TagBlock<'a>),
    // Note(NoteBlock<'a>),
}

impl<'a> LogBlock<'a> {
    // METHODS ----------------------------------------------------------------

    /// Makes this type owned, i.e. changing the lifetime to `'static`.
    pub fn make_owned(self) -> LogBlock<'static> {
        match self {
            LogBlock::Text(v) => LogBlock::Text(v.make_owned()),
            LogBlock::Prefix(v) => LogBlock::Prefix(v.make_owned()),
        }
    }
}

impl<'a> From<TextBlock<'a>> for LogBlock<'a> {
    fn from(block: TextBlock<'a>) -> Self {
        LogBlock::Text(block)
    }
}

impl<'a> Printable for LogBlock<'a> {
    fn print<'b>(&'b self, printer: &mut Printer<'b>) {
        match self {
            LogBlock::Text(v) => v.print(printer),
            LogBlock::Prefix(v) => v.print(printer),
        }
    }
}
