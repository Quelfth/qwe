use std::ops::Range;

use crate::{
    document::{CursorChange, CursorChangeBias},
    editor::cursors::{
        Cursor, CursorSet,
        select::{SelectCursor, SelectCursors},
    },
    ix::{Ix, Line},
    pos::Pos,
    rope::Rope,
};

pub type MirrorInsertCursors = CursorSet<MirrorInsertCursor>;

impl MirrorInsertCursors {
    pub fn to_select(&self, text: &Rope) -> SelectCursors {
        self.map_to(|c| c.to_select(text))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MirrorInsertCursor {
    pub forward: Pos,
    pub reverse: Pos,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InsertDirection {
    Forward,
    Reverse,
}

impl MirrorInsertCursor {
    pub fn to_select(self, text: &Rope) -> SelectCursor {
        let Self { forward, reverse } = self;
        if forward < reverse {
            SelectCursor::range(forward..reverse, text)
        } else {
            SelectCursor::range(reverse..forward, text)
        }
    }
}

impl Cursor for MirrorInsertCursor {
    fn apply_change(&mut self, change: CursorChange, _: &Rope) {
        let Self { forward, reverse } = self;
        *forward = change.apply(*forward, CursorChangeBias::Right);
        *reverse = change.apply(*reverse, CursorChangeBias::Left);
    }

    fn location_cmp(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.forward
            .min(left.reverse)
            .cmp(&right.forward.min(right.reverse))
    }

    fn line_range(&self) -> Range<Ix<Line>> {
        self.forward.line.min(self.reverse.line)
            ..self.forward.line.max(self.reverse.line) + Ix::new(1)
    }
}
