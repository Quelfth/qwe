use std::cmp::Ordering;

use crate::{
    constants::TAB_WIDTH,
    document::{CursorChange, CursorChangeBias},
    editor::cursors::{
        Cursor, CursorSet,
        line_select::{LineCursor, LineCursors},
        select::{RangeCursorLine, SelectCursor, SelectCursors},
    },
    ix::{Column, Ix, Line},
    pos::{Pos, Region},
    rope::Rope,
};

pub type InsertCursors = CursorSet<InsertCursor>;

impl InsertCursors {
    pub fn to_select(&self) -> SelectCursors {
        self.map_to(|c| c.to_select())
    }

    pub fn to_line_select(&self) -> LineCursors {
        self.map_to(|c| c.to_line_select())
    }

    pub fn move_x(&mut self, columns: Ix<Column, isize>) {
        self.iter_mut().for_each(|c| c.move_x(columns))
    }

    pub fn move_y(&mut self, rows: Ix<Line, isize>) {
        self.iter_mut().for_each(|c| c.move_y(rows))
    }
}

#[derive(Copy, Clone)]
pub struct InsertCursor {
    pub pos: Pos,
}

impl InsertCursor {
    pub fn forward(pos: Pos) -> Self {
        Self { pos }
    }

    fn to_select(self) -> SelectCursor {
        let Self {
            pos: Pos { line, column },
            ..
        } = self;
        SelectCursor {
            line,
            first_line: RangeCursorLine {
                start: column,
                end: column,
            },
            other_lines: Vec::new(),
        }
    }

    fn to_line_select(self) -> LineCursor {
        LineCursor {
            line: self.pos.line,
            height: Ix::new(1),
        }
    }

    pub fn tab(&mut self) {
        self.pos.column = Ix::new((self.pos.column.inner() / TAB_WIDTH + 1) * TAB_WIDTH)
    }

    pub fn move_x(&mut self, columns: Ix<Column, isize>) {
        match columns.cmp(&Ix::new(0)) {
            Ordering::Less => {
                self.pos.column = self.pos.column.saturating_sub((-columns).to_usize())
            }
            Ordering::Equal => (),
            Ordering::Greater => self.pos.column += columns.to_usize(),
        }
    }

    fn move_y(&mut self, rows: Ix<Line, isize>) {
        match rows.cmp(&Ix::new(0)) {
            Ordering::Less => self.pos.line = self.pos.line.saturating_sub((-rows).to_usize()),
            Ordering::Equal => (),
            Ordering::Greater => self.pos.line += rows.to_usize(),
        }
    }

    pub fn inspect_range(&self) -> Region {
        todo!()
    }
}

impl Cursor for InsertCursor {
    fn apply_change(&mut self, change: CursorChange, _: &Rope) {
        self.pos = change.apply(self.pos, CursorChangeBias::Right);
    }

    fn location_cmp(left: &Self, right: &Self) -> Ordering {
        left.pos.cmp(&right.pos)
    }
}
