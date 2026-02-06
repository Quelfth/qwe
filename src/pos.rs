use std::ops::Range;

use crate::ix::{Column, Ix, Line};

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
