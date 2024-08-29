pub use chars::*;

#[cfg(feature = "ascii-chars")]
mod chars {
    // pub const DOWN_ARROW: char = '↓';
    // pub const UP_ARROW: char = '↑';
    // pub const RIGHT_ARROW: char = '→';
    // pub const LEFT_ARROW: char = '←';
    pub const VERTICAL_BAR: char = '│';
    pub const HORIZONTAL_BAR: char = '─';
    pub const TOP_LEFT_CORNER: char = '┘';
    pub const TOP_RIGHT_CORNER: char = '└';
    pub const BOTTOM_RIGHT_CORNER: char = '┌';
    // pub const BOTTOM_LEFT_CORNER: char = '┐';
    pub const VERTICAL_RIGHT_BAR: char = '├';
    pub const VERTICAL_LEFT_BAR: char = '┤';
    pub const HORIZONTAL_TOP_BAR: char = '┴';
    pub const HORIZONTAL_BOTTOM_BAR: char = '┬';
    // pub const HORIZONTAL_VERTICAL: char = '┼';
    pub const MIDDLE_DOT: char = '·';
    pub const NEW_LINE: char = '↩';
    pub const UP_POINTER: char = '^';
    pub const RIGHT_POINTER: char = '>';
    // pub const LEFT_POINTER: char = '<';
}

#[cfg(not(feature = "ascii-chars"))]
mod chars {
    // pub const DOWN_ARROW: char = '↓';
    // pub const UP_ARROW: char = '↑';
    pub const VERTICAL_BAR: char = '│';
    pub const HORIZONTAL_BAR: char = '─';
    pub const TOP_LEFT_CORNER: char = '┘';
    pub const TOP_RIGHT_CORNER: char = '└';
    pub const BOTTOM_RIGHT_CORNER: char = '┌';
    // pub const BOTTOM_LEFT_CORNER: char = '┐';
    pub const VERTICAL_RIGHT_BAR: char = '├';
    pub const VERTICAL_LEFT_BAR: char = '┤';
    pub const HORIZONTAL_TOP_BAR: char = '┴';
    pub const HORIZONTAL_BOTTOM_BAR: char = '┬';
    // pub const HORIZONTAL_VERTICAL: char = '┼';
    pub const MIDDLE_DOT: char = '·';
    pub const NEW_LINE: char = '↩';
    pub const UP_POINTER: char = '^';
    pub const RIGHT_POINTER: char = '>';
    // pub const LEFT_POINTER: char = '<';
}
