use std::iter;

use auto_enums::auto_enum;
use crossterm::style::Color;
use culit::culit;

use crate::editor::cursors::{CursorState, select::RangeCursorLine};

use super::Range;

#[derive(Copy, Clone)]
pub(super) struct CursorRange {
    pub(super) kind: CursorRangeKind,
    pub(super) range: Range<usize>,
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
                range: Range::one(pos - 1),
            }),
            Some(Self {
                kind: right,
                range: Range::one(pos),
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
                range: Range { start, end },
            }),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum CursorRangeKind {
    InsertLeft,
    InsertRight,
    Select,
    SelectLeft,
    SelectRight,
}

impl CursorRangeKind {
    #[culit]
    pub(super) fn color(self) -> Color {
        match self {
            CursorRangeKind::InsertLeft => 0x003830rgb,
            CursorRangeKind::InsertRight => 0x007060rgb,
            CursorRangeKind::Select => 0x202070rgb,
            CursorRangeKind::SelectLeft => 0x101050rgb,
            CursorRangeKind::SelectRight => 0x404090rgb,
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
            _ => iter::empty(),
        }
    }
}
