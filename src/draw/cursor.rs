use std::iter;

use auto_enums::auto_enum;
use crossterm::style::Color;
use culit::culit;

use crate::editor::cursors::{CursorState, select::RangeCursorLine};

use super::Range;

#[derive(Copy, Clone)]
pub struct CursorRange {
    pub kind: CursorRangeKind,
    pub range: Option<Range<usize>>,
}

impl CursorRange {
    pub(super) fn thin(
        pos: usize,
        left: CursorRangeKind,
        right: CursorRangeKind,
    ) -> impl Iterator<Item = Self> {
        [
            (pos > 0).then(|| Self {
                kind: left,
                range: Some(Range::one(pos - 1)),
            }),
            Some(Self {
                kind: right,
                range: Some(Range::one(pos)),
            }),
        ]
        .into_iter()
        .flatten()
    }

    pub(super) fn insert(pos: usize) -> impl Iterator<Item = Self> {
        Self::thin(
            pos,
            CursorRangeKind::InsertLeft,
            CursorRangeKind::InsertRight,
        )
    }

    #[auto_enum(Iterator)]
    pub(super) fn select(start: usize, end: usize) -> impl Iterator<Item = Self> {
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
    pub(super) fn line_ranges(&self, line: usize) -> impl Iterator<Item = CursorRange> {
        match self {
            CursorState::Insert(cursors) => cursors
                .iter()
                .flat_map(move |c| (c.pos.line == line).then(|| CursorRange::insert(c.pos.column)))
                .flatten(),
            CursorState::Select(cursors) => cursors
                .iter()
                .filter_map(move |c| {
                    let RangeCursorLine { start, end } = c.on_line(line)?;
                    Some(CursorRange::select(start, end))
                })
                .flatten(),
            CursorState::LineSelect(cursors) => (cursors
                .iter()
                .any(|c| c.line <= line && c.line + c.height > line))
            .then(CursorRange::line)
            .or_else(|| {
                cursors
                    .iter()
                    .any(|c| c.line == line + 1 && c.height == 0)
                    .then_some(CursorRange {
                        kind: CursorRangeKind::LineBetween,
                        range: None,
                    })
            })
            .into_iter(),
        }
    }
}
