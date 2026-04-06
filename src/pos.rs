use std::{ops::Range, str::FromStr};

use crate::{
    ix::{Column, Ix, Line, Utf16}
};

pub mod convert;

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

    pub fn offset(self, lines: Ix<Line>, columns: Ix<Column>) -> Self {
        Pos{
            line: self.line + lines,
            column: if lines == Ix::ZERO {
                self.column + columns
            } else {
                columns
            }
        }
    }
}

impl FromStr for Pos {
    type Err = <usize as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some((line, col)) = s.split_once(":") {
            Self {
                line: Ix::new(line.parse::<usize>()?.saturating_sub(1)),
                column: Ix::new(col.parse::<usize>()?.saturating_sub(1)),
            }
        } else {
            Self {
                line: Ix::new(s.parse::<usize>()?.saturating_sub(1)),
                column: Ix::new(0),
            }
        })
    }
}

pub enum Region {
    Pos(Range<Pos>),
    Line(Range<Ix<Line>>),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
