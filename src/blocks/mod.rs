pub use document::*;
pub use document::*;
pub use indent::*;
pub use note::*;
pub use plain_text::*;
pub use separator::*;
pub use stack::*;
pub use stack_trace::*;
pub use tag::*;
pub use title::*;

use crate::Log;

mod document;
mod indent;
mod note;
mod plain_text;
mod separator;
mod stack;
mod stack_trace;
mod tag;
mod title;

/// One block that belongs to a log.
#[derive(Debug, Clone)]
pub enum LogBlock<'a> {
    Title(TitleBlock<'a>),
    PlainText(PlainTextBlock<'a>),
    Document(DocumentBlock<'a>),
    Separator(SeparatorBlock),
    Indent(IndentBlock<'a>),
    Stack(StackBlock<'a>),
    Tag(TagBlock<'a>),
    Note(NoteBlock<'a>),
}

impl<'a> LogBlock<'a> {
    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        match self {
            LogBlock::Title(block) => {
                block.to_text(log, in_ansi, buffer);
            }
            LogBlock::PlainText(block) => {
                block.to_text(buffer);
            }
            LogBlock::Document(block) => {
                block.to_text(log, in_ansi, buffer);
            }
            LogBlock::Separator(block) => {
                block.to_text(log, in_ansi, buffer);
            }
            LogBlock::Indent(block) => {
                block.to_text(in_ansi, buffer);
            }
            LogBlock::Stack(block) => {
                block.to_text(log, in_ansi, buffer);
            }
            LogBlock::Tag(block) => {
                block.to_text(log, in_ansi, buffer);
            }
            LogBlock::Note(block) => {
                block.to_text(log, in_ansi, buffer);
            }
        }
    }
}
