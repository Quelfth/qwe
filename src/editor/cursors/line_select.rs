use std::cmp::Ordering::*;

use crate::{
    document::{CursorChange, Document},
    editor::cursors::{
        Cursor, CursorSet,
        select::{RangeCursorLine, SelectCursor, SelectCursors},
    },
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

    pub fn to_select(&self, doc: &Document) -> SelectCursors {
        self.map_to(|c| c.to_select(doc))
    }
}

#[derive(Default)]
pub struct LineCursor {
    pub line: usize,
    pub height: usize,
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
        let line = line + height.min(1) - 1;
        InsertCursor::forward(Pos {
            line,
            column: doc.indent_on_line(line),
        })
    }

    pub fn to_select(&self, doc: &Document) -> SelectCursor {
        let Self { line, height } = *self;
        let height = height.min(1);
        let end_line = line + height;

        let start = (line..end_line)
            .map(|l| doc.indent_on_line(l))
            .min()
            .unwrap();

        SelectCursor {
            line,
            first_line: RangeCursorLine {
                start,
                end: doc.columns_in_line(line),
            },
            other_lines: (line + 1..end_line)
                .map(|l| RangeCursorLine {
                    start,
                    end: doc.columns_in_line(l),
                })
                .collect(),
        }
    }

    pub fn move_y(&mut self, rows: isize) {
        match rows.cmp(&0) {
            Less => self.line = self.line.saturating_sub(-rows as usize),
            Equal => (),
            Greater => self.line += rows as usize,
        }
    }

    pub fn extend_up(&mut self, rows: usize) {
        let rows = rows.min(self.line);
        self.line -= rows;
        self.height += rows;
    }

    pub fn extend_down(&mut self, rows: usize) {
        self.height += rows;
    }

    pub fn retract_down(&mut self, rows: usize) {
        self.line += rows;
        self.height = self.height.saturating_sub(rows);
    }

    pub fn retract_up(&mut self, rows: usize) {
        self.height = self.height.saturating_sub(rows);
    }
}

impl Cursor for LineCursor {
    fn apply_change(&mut self, _: CursorChange) {}
}
