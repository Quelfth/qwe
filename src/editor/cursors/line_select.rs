use std::{cmp::Ordering::*, ops::Range};

use crate::{
    document::CursorChange,
    editor::cursors::{
        Cursor, CursorSet,
        select::{RangeCursorLine, SelectCursor, SelectCursors},
    },
    pos::Pos,
    rope::Rope,
};

use super::insert::*;

pub type LineCursors = CursorSet<LineCursor>;

impl LineCursors {
    pub fn move_y(&mut self, rows: isize) {
        self.iter_mut().for_each(|c| c.move_y(rows))
    }

    pub fn to_insert_before(&self, text: &Rope) -> InsertCursors {
        self.map_to(|c| c.to_insert_before(text))
    }
    pub fn to_insert_after(&self, text: &Rope) -> InsertCursors {
        self.map_to(|c| c.to_insert_after(text))
    }

    pub fn to_select(&self, doc: &Rope) -> SelectCursors {
        self.map_to(|c| c.to_select(doc))
    }

    pub fn delete_ranges(&self, doc: &Rope) -> impl Iterator<Item = Range<usize>> {
        self.iter().filter_map(|c| c.text_range(doc))
    }
}

#[derive(Default)]
pub struct LineCursor {
    pub line: usize,
    pub height: usize,
}

impl LineCursor {
    fn to_insert_before(&self, doc: &Rope) -> InsertCursor {
        let Self { line, .. } = *self;
        InsertCursor::forward(Pos {
            line,
            column: doc.indent_on_line(line),
        })
    }
    fn to_insert_after(&self, doc: &Rope) -> InsertCursor {
        let Self { line, height } = *self;
        let line = line + height.max(1) - 1;
        InsertCursor::forward(Pos {
            line,
            column: doc.columns_in_line(line),
        })
    }

    pub fn to_select(&self, doc: &Rope) -> SelectCursor {
        let Self { line, height } = *self;
        let height = height.max(1);
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

    pub fn text_range(&self, text: &Rope) -> Option<Range<usize>> {
        if self.height == 0 {
            return None;
        }
        let start = text.byte_of_line(self.line)?;
        let end_line = self.line + self.height - 1;
        if end_line >= text.line_len() {
            return Some(start..text.byte_len());
        }
        let mut end =
            text.byte_of_line(end_line).unwrap() + text.line(end_line).unwrap().byte_len();
        if text.byte_slice(end..=end).unwrap().to_string() == "\r" {
            end += 1;
        }
        if text.byte_slice(end..=end).unwrap().to_string() == "\n" {
            end += 1;
        }
        Some(start..end)
    }
}

impl Cursor for LineCursor {
    fn apply_change(&mut self, c: CursorChange) {
        let start = c.apply_to_line(self.line);
        let end = c.apply_to_line(self.line + self.height);

        self.line = start;
        self.height = end.saturating_sub(start);
    }
}
