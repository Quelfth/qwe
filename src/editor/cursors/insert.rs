use std::cmp::Ordering;

use crate::{
    constants::TAB_WIDTH,
    document::CursorChange,
    editor::cursors::{
        Cursor, CursorSet,
        select::{RangeCursorLine, SelectCursor, SelectCursors},
    },
    pos::Pos,
};

pub type InsertCursors = CursorSet<InsertCursor>;

impl InsertCursors {
    pub fn to_select(&self) -> SelectCursors {
        self.map_to(|c| c.to_select())
    }

    pub fn tab(mut self) -> Self {
        self.iter_mut().for_each(|c| c.tab());
        self
    }

    pub fn move_x(&mut self, columns: isize) {
        self.iter_mut().for_each(|c| c.move_x(columns))
    }

    pub fn move_y(&mut self, rows: isize) {
        self.iter_mut().for_each(|c| c.move_y(rows))
    }
}

#[derive(Copy, Clone)]
pub struct InsertCursor {
    pub direction: InsertDirection,
    pub pos: Pos,
}

impl InsertCursor {
    pub fn forward(pos: Pos) -> Self {
        Self {
            direction: InsertDirection::Forward,
            pos,
        }
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

    fn tab(&mut self) {
        self.pos.column = (self.pos.column / TAB_WIDTH + 1) * TAB_WIDTH
    }

    fn move_x(&mut self, columns: isize) {
        match columns.cmp(&0) {
            Ordering::Less => self.pos.column = self.pos.column.saturating_sub((-columns) as usize),
            Ordering::Equal => (),
            Ordering::Greater => self.pos.column += columns as usize,
        }
    }

    fn move_y(&mut self, rows: isize) {
        match rows.cmp(&0) {
            Ordering::Less => self.pos.line = self.pos.line.saturating_sub((-rows) as usize),
            Ordering::Equal => (),
            Ordering::Greater => self.pos.line += rows as usize,
        }
    }
}

impl Cursor for InsertCursor {
    fn apply_change(&mut self, change: CursorChange) {
        self.pos = change.apply(self.pos);
    }
}

#[derive(Copy, Clone, Default)]
pub enum InsertDirection {
    #[default]
    Forward,
    Reverse,
}
