use crate::printer::{Printable, Printer};

pub use code::*;
pub use header::*;
pub use note::*;
pub use prefix::*;
pub use separator::*;
pub use stack::*;
pub use stack_trace::*;
pub use step::*;
pub use text::*;

mod code;
mod header;
mod note;
mod prefix;
mod separator;
mod stack;
mod stack_trace;
mod step;
mod text;

/// A block log.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum LogBlock<'a> {
    // Basic blocks.
    Text(TextBlock<'a>),
    Prefix(PrefixBlock<'a>),

    // Custom blocks.
    Separator(SeparatorBlock),
    Header(HeaderBlock<'a>),
    Note(NoteBlock<'a>),
    Stack(StackBlock<'a>),
    Code(CodeBlock<'a>),
    Steps(StepsBlock<'a>),
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
            LogBlock::Code(v) => LogBlock::Code(v.make_owned()),
            LogBlock::Steps(v) => LogBlock::Steps(v.make_owned()),
        }
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
            LogBlock::Code(v) => v.print(printer),
            LogBlock::Steps(v) => v.print(printer),
        }
    }
}

impl<'a> From<TextBlock<'a>> for LogBlock<'a> {
    fn from(block: TextBlock<'a>) -> Self {
        LogBlock::Text(block)
    }
}

impl<'a> From<PrefixBlock<'a>> for LogBlock<'a> {
    fn from(block: PrefixBlock<'a>) -> Self {
        LogBlock::Prefix(block)
    }
}

impl<'a> From<SeparatorBlock> for LogBlock<'a> {
    fn from(block: SeparatorBlock) -> Self {
        LogBlock::Separator(block)
    }
}

impl<'a> From<HeaderBlock<'a>> for LogBlock<'a> {
    fn from(block: HeaderBlock<'a>) -> Self {
        LogBlock::Header(block)
    }
}

impl<'a> From<NoteBlock<'a>> for LogBlock<'a> {
    fn from(block: NoteBlock<'a>) -> Self {
        LogBlock::Note(block)
    }
}

impl<'a> From<StackBlock<'a>> for LogBlock<'a> {
    fn from(block: StackBlock<'a>) -> Self {
        LogBlock::Stack(block)
    }
}

impl<'a> From<CodeBlock<'a>> for LogBlock<'a> {
    fn from(block: CodeBlock<'a>) -> Self {
        LogBlock::Code(block)
    }
}

impl<'a> From<StepsBlock<'a>> for LogBlock<'a> {
    fn from(block: StepsBlock<'a>) -> Self {
        LogBlock::Steps(block)
    }
}
