use std::ops::Range;

use crate::ix::{Column, Ix, Line, Utf16};

#[derive(Copy, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct Pos {
    pub line: Ix<Line>,
    pub column: Ix<Column>,
}

impl Pos {
    pub const ZERO: Self = Self {
        line: Ix::new(0),
        column: Ix::new(0),
    };
}

pub enum Region {
    Pos(Range<Pos>),
    Line(Range<Ix<Line>>),
}

#[derive(Copy, Clone)]
pub struct Utf16Pos {
    pub line: Ix<Line>,
    pub column: Ix<Utf16>,
}

impl Utf16Pos {
    pub fn from_lsp_pos(pos: lsp_types::Position) -> Self {
        Self {
            line: Ix::new(pos.line as _),
            column: Ix::new(pos.character as _),
        }
    }
}
