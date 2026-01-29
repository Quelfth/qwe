use std::{iter, ops::Range};

use crate::{document::CursorChange, pos::Pos, rope::Rope};

use auto_enums::auto_enum;
use insert::InsertCursors;
use line_select::LineCursors;
use select::SelectCursors;

pub mod insert;
pub mod line_select;
pub mod select;

pub enum CursorState {
    Insert(InsertCursors),
    Select(SelectCursors),
    LineSelect(LineCursors),
}

impl CursorState {
    pub fn drop_others(&mut self) {
        match self {
            CursorState::Insert(c) => c.drop_others(),
            CursorState::Select(c) => c.drop_others(),
            CursorState::LineSelect(c) => c.drop_others(),
        }
    }
}

#[derive(Clone, Default)]
pub struct CursorSet<Cursor> {
    main: Cursor,
    others: Vec<Cursor>,
}

impl<T> CursorSet<T> {
    pub fn one(cursor: T) -> Self {
        Self {
            main: cursor,
            others: Vec::new(),
        }
    }

    pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Option<Self> {
        let mut iter = iter.into_iter();
        Some(Self {
            main: iter.next()?,
            others: iter.collect(),
        })
    }

    pub fn drop_others(&mut self) {
        self.others.clear();
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::Select(Default::default())
    }
}

impl From<InsertCursors> for CursorState {
    fn from(value: InsertCursors) -> Self {
        Self::Insert(value)
    }
}

impl From<SelectCursors> for CursorState {
    fn from(value: SelectCursors) -> Self {
        Self::Select(value)
    }
}

impl From<LineCursors> for CursorState {
    fn from(value: LineCursors) -> Self {
        Self::LineSelect(value)
    }
}

impl CursorState {
    pub fn apply_change(&mut self, change: CursorChange) {
        match self {
            CursorState::Insert(cursors) => cursors.apply_change(change),
            CursorState::Select(cursors) => cursors.apply_change(change),
            CursorState::LineSelect(cursors) => cursors.apply_change(change),
        }
    }

    pub fn move_x(&mut self, columns: isize) {
        match self {
            CursorState::Insert(cursors) => cursors.move_x(columns),
            CursorState::Select(cursors) => cursors.move_x(columns),
            CursorState::LineSelect(_) => (),
        }
    }

    pub fn move_y(&mut self, rows: isize) {
        match self {
            CursorState::Insert(c) => c.move_y(rows),
            CursorState::Select(c) => c.move_y(rows),
            CursorState::LineSelect(c) => c.move_y(rows),
        }
    }

    pub fn text_extend_up(&mut self, rows: usize, text: &Rope) {
        match self {
            CursorState::Insert(_) => (),
            CursorState::Select(c) => c.iter_mut().for_each(|c| c.text_extend_up(rows, text)),
            CursorState::LineSelect(c) => c.iter_mut().for_each(|c| c.extend_up(rows)),
        }
    }

    pub fn text_extend_down(&mut self, rows: usize, text: &Rope) {
        match self {
            CursorState::Insert(_) => (),
            CursorState::Select(c) => c.iter_mut().for_each(|c| c.text_extend_down(rows, text)),
            CursorState::LineSelect(c) => c.iter_mut().for_each(|c| c.extend_down(rows)),
        }
    }
    pub fn extend_left(&mut self, rows: usize) {
        if let CursorState::Select(c) = self {
            c.iter_mut().for_each(|c| c.extend_left(rows))
        }
    }
    pub fn extend_right(&mut self, rows: usize) {
        if let CursorState::Select(c) = self {
            c.iter_mut().for_each(|c| c.extend_right(rows))
        }
    }
    pub fn retract_up(&mut self, rows: usize) {
        match self {
            CursorState::Insert(_) => (),
            CursorState::Select(c) => c.iter_mut().for_each(|c| c.retract_up(rows)),
            CursorState::LineSelect(c) => c.iter_mut().for_each(|c| c.retract_up(rows)),
        }
    }

    pub fn retract_down(&mut self, rows: usize) {
        match self {
            CursorState::Insert(_) => (),
            CursorState::Select(c) => c.iter_mut().for_each(|c| c.retract_down(rows)),
            CursorState::LineSelect(c) => c.iter_mut().for_each(|c| c.retract_down(rows)),
        }
    }
    pub fn retract_left(&mut self, rows: usize) {
        if let CursorState::Select(c) = self {
            c.iter_mut().for_each(|c| c.retract_left(rows))
        }
    }
    pub fn retract_right(&mut self, rows: usize) {
        if let CursorState::Select(c) = self {
            c.iter_mut().for_each(|c| c.retract_right(rows))
        }
    }

    pub fn inspect_range(&self) -> (Pos, Pos) {
        match self {
            CursorState::Insert(c) => c.main.inspect_range(),
            CursorState::Select(c) => c.main.inspect_range(),
            CursorState::LineSelect(_) => todo!(),
        }
    }

    #[auto_enum(Iterator)]
    pub fn delete_ranges(&self, text: &Rope) -> impl Iterator<Item = Range<usize>> {
        match self {
            CursorState::Insert(_) => iter::empty(),
            CursorState::Select(c) => c.delete_ranges(text),
            CursorState::LineSelect(c) => c.delete_ranges(text),
        }
    }
}

impl<T> CursorSet<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        iter::once(&self.main).chain(&self.others)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        iter::once(&mut self.main).chain(&mut self.others)
    }

    pub fn map_to<U>(&self, map: impl Fn(&T) -> U) -> CursorSet<U> {
        CursorSet {
            main: map(&self.main),
            others: self.others.iter().map(map).collect(),
        }
    }

    pub fn apply_change(&mut self, change: CursorChange)
    where
        T: Cursor,
    {
        for cursor in self.iter_mut() {
            cursor.apply_change(change)
        }
    }
}

pub trait Cursor {
    fn apply_change(&mut self, change: CursorChange);
}
