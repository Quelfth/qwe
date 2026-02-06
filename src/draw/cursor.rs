use std::iter;

use auto_enums::auto_enum;
use crossterm::style::Color;
use culit::culit;

use crate::{
    editor::cursors::{CursorState, select::RangeCursorLine},
    ix::{Column, Ix, Line},
};

use super::Range;

#[derive(Copy, Clone)]
pub struct CursorRange {
    pub kind: CursorRangeKind,
    pub range: Option<Range<Ix<Column>>>,
}

impl CursorRange {
    pub(super) fn thin(
        pos: Ix<Column>,
        left: CursorRangeKind,
        right: CursorRangeKind,
    ) -> impl Iterator<Item = Self> {
        [
            (pos > Ix::new(0)).then(|| Self {
                kind: left,
                range: Some(Range::one(pos - Ix::new(1))),
            }),
            Some(Self {
                kind: right,
                range: Some(Range::one(pos)),
            }),
        ]
        .into_iter()
        .flatten()
    }

    pub(super) fn insert(pos: Ix<Column>) -> impl Iterator<Item = Self> {
        Self::thin(
            pos,
            CursorRangeKind::InsertLeft,
            CursorRangeKind::InsertRight,
        )
    }

    pub(super) fn mirror_insert(pos: Ix<Column>, forward: bool) -> impl Iterator<Item = Self> {
        let (l, r) = (CursorRangeKind::InsertLeft, CursorRangeKind::InsertRight);
        let (l, r) = if forward { (l, r) } else { (r, l) };
        Self::thin(pos, l, r)
    }

    #[auto_enum(Iterator)]
    pub(super) fn select(start: Ix<Column>, end: Ix<Column>) -> impl Iterator<Item = Self> {
        match start == end {
            true => Self::thin(
                start,
                CursorRangeKind::SelectLeft,
                CursorRangeKind::SelectRight,
            ),
            false => iter::once(Self {
                kind: CursorRangeKind::Select,
                range: Some(Range { start, end }),
            }),
        }
    }

    fn line() -> Self {
        Self {
            kind: CursorRangeKind::Select,
            range: None,
        }
    }
}

#[derive(Copy, Clone)]
pub enum CursorRangeKind {
    InsertLeft,
    InsertRight,
    Select,
    SelectLeft,
    SelectRight,
    LineBetween,
}

#[derive(Copy, Clone)]
pub enum CursorStyle {
    Color(Color),
    Underline(Color),
}

impl CursorRangeKind {
    #[culit]
    pub(super) fn style(self) -> CursorStyle {
        use CursorStyle::*;
        match self {
            CursorRangeKind::InsertLeft => Color(0x003830rgb),
            CursorRangeKind::InsertRight => Color(0x007060rgb),
            CursorRangeKind::Select => Color(0x202070rgb),
            CursorRangeKind::SelectLeft => Color(0x101050rgb),
            CursorRangeKind::SelectRight => Color(0x404090rgb),
            CursorRangeKind::LineBetween => Underline(0x404090rgb),
        }
    }
}

impl CursorState {
    #[auto_enum(Iterator)]
    pub(super) fn line_ranges(&self, line: Ix<Line>) -> impl Iterator<Item = CursorRange> {
        use CursorState::*;
        match self {
            MirrorInsert(cursors) => cursors
                .iter()
                .flat_map(|c| [(c.forward, true), (c.reverse, false)])
                .flat_map(move |(c, forward)| {
                    (c.line == line).then(|| CursorRange::mirror_insert(c.column, forward))
                })
                .flatten(),
            Insert(cursors) => cursors
                .iter()
                .flat_map(move |c| (c.pos.line == line).then(|| CursorRange::insert(c.pos.column)))
                .flatten(),
            Select(cursors) => cursors
                .iter()
                .filter_map(move |c| {
                    let RangeCursorLine { start, end } = c.on_line(line)?;
                    Some(CursorRange::select(start, end))
                })
                .flatten(),
            LineSelect(cursors) => (cursors
                .iter()
                .any(|c| c.line <= line && c.line + c.height > line))
            .then(CursorRange::line)
            .or_else(|| {
                cursors
                    .iter()
                    .any(|c| c.line == line + Ix::new(1) && c.height == Ix::new(0))
                    .then_some(CursorRange {
                        kind: CursorRangeKind::LineBetween,
                        range: None,
                    })
            })
            .into_iter(),
        }
    }
}
