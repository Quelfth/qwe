use std::{
    iter,
    range::Range,
};

use auto_enums::auto_enum;
use crossterm::style::Color;
use culit::culit;

use crate::{
    editor::cursors::{CursorIndex, CursorState},
    ix::{Column, Ix, Line, ix}, pos::Pos,
};

#[derive(Copy, Clone)]
pub struct CursorRange {
    pub r#type: CursorType,
    pub range: CursorRangeShape,
}

#[derive(Copy, Clone)]
pub enum CursorRangeShape {
    Range {
        line: Ix<Line>,
        range: Range<Ix<Column>>,
    },
    Line(Range<Ix<Line>>),
}

impl CursorRange {
    pub(super) fn thin(
        pos: Pos,
        left: CursorType,
        right: CursorType,
    ) -> impl Iterator<Item = Self> {
        use CursorRangeShape::*;
        [
            (pos.column > Ix::new(0)).then(|| Self {
                r#type: left,
                range: Range {
                    line: pos.line,
                    range: pos.column - Ix::new(1)..pos.column
                },
            }),
            Some(Self {
                r#type: right,
                range: Range {
                    line: pos.line,
                    range: pos.column..pos.column + Ix::new(1)
                },
            }),
        ]
        .into_iter()
        .flatten()
    }

    pub(super) fn insert(pos: Pos, order: impl ToCursorOrder) -> impl Iterator<Item = Self> {
        Self::thin(
            pos,
            cursor_type!(Insert Start[order]),
            cursor_type!(Insert End[order]),
        )
    }

    pub(super) fn mirror_insert(
        pos: Pos,
        forward: bool,
        order: impl ToCursorOrder,
    ) -> impl Iterator<Item = Self> {
        let (l, r) = (
            cursor_type!(Insert Start[order]),
            cursor_type!(Insert End[order]),
        );
        let (l, r) = if forward { (l, r) } else { (r, l) };
        Self::thin(pos, l, r)
    }

    #[auto_enum(Iterator)]
    pub(super) fn select(
        line: Ix<Line>,
        range: Range<Ix<Column>>,
        index: impl ToCursorOrder,
    ) -> impl Iterator<Item = Self> {
        let Range { start, end } = range;
        match start == end {
            true => Self::thin(
                Pos{ line, column: start },
                cursor_type!(Select Start [index]),
                cursor_type!(Select End [index]),
            ),
            false => iter::once(Self {
                r#type: cursor_type!(Select[index]),
                range: CursorRangeShape::Range { line, range },
            }),
        }
    }

    fn line(lines: Range<Ix<Line>>, order: impl ToCursorOrder) -> Self {
        Self {
            r#type: cursor_type!(Select[order]),
            range: CursorRangeShape::Line(lines),
        }
    }

    fn between_line(line: Ix<Line>, order: impl ToCursorOrder) -> Self {
        Self {
            r#type: cursor_type!(Select Line [order]),
            range: CursorRangeShape::Line(line.saturating_sub(ix(1))..line),
        }
    }
}

macro_rules! cursor_type {
    ($cat:ident [$order:expr]) => {
        $crate::draw::cursor::CursorType {
            part: $crate::draw::cursor::CursorPart::Middle,
            category: $crate::draw::cursor::CursorCategory::$cat,
            order: $crate::draw::cursor::ToCursorOrder::to_cursor_order($order),
        }
    };
    ($cat:ident $part:ident[$order:expr]) => {
        $crate::draw::cursor::CursorType {
            part: $crate::draw::cursor::CursorPart::$part,
            category: $crate::draw::cursor::CursorCategory::$cat,
            order: $crate::draw::cursor::ToCursorOrder::to_cursor_order($order),
        }
    };
}
pub(crate) use cursor_type;

#[derive(Copy, Clone)]
pub struct CursorType {
    part: CursorPart,
    category: CursorCategory,
    order: CursorOrder,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CursorPart {
    Start,
    Middle,
    End,
    Line,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CursorCategory {
    Insert,
    Select,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CursorOrder {
    Main,
    Odd,
    Even,
}

impl CursorOrder {
    pub fn iter() -> CursorOrderIter {
        Default::default()
    }
}

pub struct CursorOrderIter(CursorOrder);

impl Default for CursorOrderIter {
    fn default() -> Self {
        Self(CursorOrder::Main)
    }
}

impl Iterator for CursorOrderIter {
    type Item = CursorOrder;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.0;
        self.0 = if self.0 == CursorOrder::Odd {
            CursorOrder::Even
        } else {
            CursorOrder::Odd
        };
        Some(result)
    }
}

pub trait ToCursorOrder: Copy {
    fn to_cursor_order(self) -> CursorOrder;
}

impl ToCursorOrder for CursorOrder {
    fn to_cursor_order(self) -> CursorOrder {
        self
    }
}

impl ToCursorOrder for CursorIndex {
    fn to_cursor_order(self) -> CursorOrder {
        use CursorOrder::*;
        match self {
            CursorIndex::Main => Main,
            CursorIndex::Other(i) => {
                if i & 1 == 0 {
                    Odd
                } else {
                    Even
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum CursorStyle {
    Color(Color),
    Underline(Color),
}

impl CursorType {
    #[culit]
    pub(super) fn style(self) -> CursorStyle {
        use crossterm::style::Color as Col;
        use {CursorCategory::*, CursorOrder::*, CursorPart::*, CursorStyle::*};
        const I_LEFT: Col = 0x003830rgb;
        const I_RIGHT: Col = 0x007060rgb;

        const IO_RIGHT: Col = 0x307030rgb;
        const IE_RIGHT: Col = 0x607020rgb;

        const S_LEFT: Col = 0x101050rgb;
        const S_RIGHT: Col = 0x404090rgb;
        const S: Col = 0x202070rgb;
        const S_LINE: Col = 0x404090rgb;

        const SO_RIGHT: Col = 0x604090rgb;
        const SO: Col = 0x402070rgb;
        const SE_RIGHT: Col = 0x804090rgb;
        const SE: Col = 0x702070rgb;

        match self.category {
            Insert => match self.order {
                Main => match self.part {
                    Start => Color(I_LEFT),
                    End => Color(I_RIGHT),
                    Middle => Color(I_RIGHT),
                    Line => Underline(I_RIGHT),
                },
                Odd => match self.part {
                    Start => Color(I_LEFT),
                    End => Color(IO_RIGHT),
                    Middle => Color(IO_RIGHT),
                    Line => Underline(IO_RIGHT),
                },
                Even => match self.part {
                    Start => Color(I_LEFT),
                    End => Color(IE_RIGHT),
                    Middle => Color(IE_RIGHT),
                    Line => Underline(IE_RIGHT),
                },
            },
            Select => match self.order {
                Main => match self.part {
                    Start => Color(S_LEFT),
                    End => Color(S_RIGHT),
                    Middle => Color(S),
                    Line => Underline(S_LINE),
                },
                Odd => match self.part {
                    Start => Color(S_LEFT),
                    End => Color(SO_RIGHT),
                    Middle => Color(SO),
                    Line => Underline(SO_RIGHT),
                },
                Even => match self.part {
                    Start => Color(S_LEFT),
                    End => Color(SE_RIGHT),
                    Middle => Color(SE),
                    Line => Underline(SE_RIGHT),
                },
            },
        }
    }
}

impl CursorState {
    #[auto_enum(Iterator)]
    pub(super) fn ranges_for_line(&self, line: Ix<Line>) -> impl Iterator<Item = CursorRange> {
        use CursorState::*;
        match self {
            MirrorInsert(cursors) => cursors
                .sorted_iter()
                .zip(CursorOrder::iter())
                .flat_map(|(c, o)| [((c.forward, true), o), ((c.reverse, false), o)])
                .flat_map(move |((c, forward), o)| {
                    (c.line == line).then(|| CursorRange::mirror_insert(c, forward, o))
                })
                .flatten(),
            Insert(cursors) => cursors
                .sorted_iter()
                .zip(CursorOrder::iter())
                .flat_map(move |(c, o)| {
                    (c.pos.line == line).then(|| CursorRange::insert(c.pos, o))
                })
                .flatten(),
            Select(cursors) => cursors
                .sorted_iter()
                .zip(CursorOrder::iter())
                .filter_map(move |(c, o)| {
                    let range = c.on_line(line)?;
                    Some(CursorRange::select(line, range, o))
                })
                .flatten(),
            LineSelect(cursors) => (cursors
                .sorted_iter()
                .zip(CursorOrder::iter())
                .find(|(c, _)| c.line <= line && c.line + c.height > line))
            .map(|(c, o)| CursorRange::line(c.range(), o))
            .or_else(|| {
                cursors
                    .sorted_iter()
                    .zip(CursorOrder::iter())
                    .find(|(c, _)| c.line == line + Ix::new(1) && c.height == Ix::new(0))
                    .map(|(c, o)| CursorRange::between_line(c.line, o))
            })
            .into_iter(),
        }
    }

    #[auto_enum(Iterator)]
    pub(super) fn ranges(&self) -> impl Iterator<Item = CursorRange> {
        use CursorState::*;
        match self {
            MirrorInsert(cursors) => cursors
                .sorted_iter()
                .zip(CursorOrder::iter())
                .flat_map(|(c, o)| [((c.forward, true), o), ((c.reverse, false), o)])
                .flat_map(move |((c, forward), o)| {
                    (true).then(|| CursorRange::mirror_insert(c, forward, o))
                })
                .flatten(),
            Insert(cursors) => cursors
                .sorted_iter()
                .zip(CursorOrder::iter())
                .flat_map(move |(c, o)| {
                    (true).then(|| CursorRange::insert(c.pos, o))
                })
                .flatten(),
            Select(cursors) => cursors
                .sorted_iter()
                .zip(CursorOrder::iter())
                .flat_map(move |(c, o)| {
                    c.lines_ix().map(move |(line, range)| CursorRange::select(line, range, o))
                }).flatten(),
            LineSelect(cursors) => cursors
                .sorted_iter()
                .zip(CursorOrder::iter())
                .map(|(c, o)| if c.height != ix(0) {
                    CursorRange::line(c.range(), o)
                } else {
                    CursorRange::between_line(c.line, o)
                }),
        }
    }
}
