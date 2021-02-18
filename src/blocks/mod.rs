pub use indent::*;
pub use note::*;
pub use plain_text::*;
pub use separator::*;
pub use tag::*;
pub use title::*;

use crate::Log;

mod indent;
mod note;
mod plain_text;
mod separator;
mod tag;
mod title;

/// One block that belongs to a log.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LogBlock {
    PlainText(PlainTextBlock),
    Separator(SeparatorBlock),
    Indent(IndentBlock),
    Tag(TagBlock),
    Note(NoteBlock),
    Title(TitleBlock),
}

impl LogBlock {
    pub(crate) fn to_text(&self, log: &Log, in_ansi: bool, buffer: &mut String) {
        match self {
            LogBlock::PlainText(block) => {
                block.to_text(buffer);
            }
            LogBlock::Separator(block) => {
                block.to_text(log, in_ansi, buffer);
            }
            LogBlock::Indent(block) => {
                block.to_text(in_ansi, buffer);
            }
            LogBlock::Tag(block) => {
                block.to_text(log, in_ansi, buffer);
            }
            LogBlock::Note(block) => {
                block.to_text(log, in_ansi, buffer);
            }
            LogBlock::Title(block) => {
                block.to_text(log, in_ansi, buffer);
            }
        }
    }
}
