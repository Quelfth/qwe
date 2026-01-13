use std::cmp::Ordering::*;

use crate::{
    document::{CursorChange, Document},
    editor::cursors::{Cursor, CursorSet},
    pos::Pos,
};

use super::insert::*;

pub type LineCursors = CursorSet<LineCursor>;

impl LineCursors {
    pub fn move_y(&mut self, rows: isize) {
        self.iter_mut().for_each(|c| c.move_y(rows))
    }

    pub fn to_insert_before(&self, doc: &Document) -> InsertCursors {
        self.map_to(|c| c.to_insert_before(doc))
    }
    pub fn to_insert_after(&self, doc: &Document) -> InsertCursors {
        self.map_to(|c| c.to_insert_after(doc))
    }
}

#[derive(Default)]
pub struct LineCursor {
    line: usize,
    height: usize,
}

impl LineCursor {
    fn to_insert_before(&self, doc: &Document) -> InsertCursor {
        let Self { line, .. } = *self;
        InsertCursor::forward(Pos {
            line,
            column: doc.indent_on_line(line),
        })
    }
    fn to_insert_after(&self, doc: &Document) -> InsertCursor {
        let Self { line, height } = *self;
        let line = line + height;
        InsertCursor::forward(Pos {
            line,
            column: doc.indent_on_line(line),
        })
    }

    pub fn move_y(&mut self, rows: isize) {
        match rows.cmp(&0) {
            Less => self.line = self.line.saturating_sub(-rows as usize),
            Equal => (),
            Greater => self.line += rows as usize,
        }
    }
}

impl Cursor for LineCursor {
    fn apply_change(&mut self, change: CursorChange) {
        todo!()
    }
}
