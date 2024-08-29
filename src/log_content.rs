use crate::blocks::LogBlock;
use crate::printer::{Printable, Printer, PrinterFormat};
use smallvec::SmallVec;
use std::fmt::Display;

/// A list of log elements.
#[derive(Default, Debug, Clone)]
pub struct LogContent<'a> {
    blocks: SmallVec<[LogBlock<'a>; 3]>,
}

impl<'a> LogContent<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new content.
    pub fn new() -> Self {
        Self::default()
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the list of log blocks.
    pub fn blocks(&self) -> &[LogBlock<'a>] {
        &self.blocks
    }

    /// Returns a mutable reference to the list of log blocks.
    pub fn blocks_mut(&mut self) -> &mut SmallVec<[LogBlock<'a>; 3]> {
        &mut self.blocks
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a new block.
    pub fn add_block(mut self, block: impl Into<LogBlock<'a>>) -> Self {
        self.blocks.push(block.into());
        self
    }

    /// Makes this type owned, i.e. changing the lifetime to `static`.
    pub fn make_owned(self) -> LogContent<'static> {
        LogContent {
            blocks: self.blocks.into_iter().map(|v| v.make_owned()).collect(),
        }
    }
}

impl<'a> Printable for LogContent<'a> {
    fn print<'b>(&'b self, printer: &mut Printer<'b>) {
        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                printer.push_plain_str("\n");
            }

            block.print(printer);
        }
    }
}

impl<'a> Display for LogContent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = Printer::new(PrinterFormat::Plain);
        self.print(&mut printer);
        printer.fmt(f, PrinterFormat::Plain)
    }
}
