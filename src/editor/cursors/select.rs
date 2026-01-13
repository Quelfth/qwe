use std::{cmp::Ordering::*, iter};

use crate::{
    document::{CursorChange, Document},
    editor::cursors::{Cursor, CursorSet},
    grapheme::Grapheme,
    pos::Pos,
};

use super::insert::*;

pub type SelectCursors = CursorSet<SelectCursor>;

impl SelectCursors {
    pub fn to_insert_before(&self) -> InsertCursors {
        self.map_to(SelectCursor::to_insert_before)
    }
    pub fn to_insert_after(&self) -> InsertCursors {
        self.map_to(SelectCursor::to_insert_after)
    }
    pub fn to_insert_before_line(&self, doc: &Document) -> InsertCursors {
        self.map_to(|c| SelectCursor::to_insert_before_line(c, doc))
    }
    pub fn to_insert_after_line(&self, doc: &Document) -> InsertCursors {
        self.map_to(|c| SelectCursor::to_insert_after_line(c, doc))
    }

    pub fn move_x(&mut self, columns: isize) {
        for cursor in self.iter_mut() {
            cursor.move_x(columns);
        }
    }

    pub fn move_y(&mut self, rows: isize) {
        for cursor in self.iter_mut() {
            cursor.move_y(rows);
        }
    }
}

impl Cursor for SelectCursor {
    fn apply_change(&mut self, change: CursorChange) {
        todo!()
    }
}

#[derive(Clone, Default)]
pub struct SelectCursor {
    pub line: usize,
    pub first_line: RangeCursorLine,
    pub other_lines: Vec<RangeCursorLine>,
}

#[derive(Copy, Clone, Default)]
pub struct RangeCursorLine {
    pub start: usize,
    pub end: usize,
}

impl SelectCursor {
    pub(super) fn to_insert_before(&self) -> InsertCursor {
        let Self {
            line, first_line, ..
        } = self;
        InsertCursor::forward(Pos {
            line: *line,
            column: first_line.start,
        })
    }

    pub(super) fn to_insert_after(&self) -> InsertCursor {
        InsertCursor::forward(Pos {
            line: self.last_line_ix(),
            column: self.last_line().end,
        })
    }

    pub(super) fn to_insert_before_line(&self, doc: &Document) -> InsertCursor {
        let Self { line, .. } = *self;
        InsertCursor::forward(Pos {
            line,
            column: doc.indent_on_line(line),
        })
    }

    pub(super) fn to_insert_after_line(&self, doc: &Document) -> InsertCursor {
        let line = self.last_line_ix();
        InsertCursor::forward(Pos {
            line,
            column: doc.columns_in_line(line),
        })
    }

    pub fn on_line(&self, line: usize) -> Option<RangeCursorLine> {
        if line < self.line {
            return None;
        }

        let line = line - self.line;
        if line == 0 {
            return Some(self.first_line);
        }

        self.other_lines.get(line - 1).copied()
    }

    fn last_line(&self) -> RangeCursorLine {
        self.other_lines.last().copied().unwrap_or(self.first_line)
    }

    fn last_line_mut(&mut self) -> &mut RangeCursorLine {
        self.other_lines.last_mut().unwrap_or(&mut self.first_line)
    }

    fn last_line_ix(&self) -> usize {
        self.line + self.other_lines.len()
    }

    fn lines_mut(&mut self) -> impl Iterator<Item = &mut RangeCursorLine> {
        iter::once(&mut self.first_line).chain(&mut self.other_lines)
    }

    pub fn move_x(&mut self, columns: isize) {
        self.lines_mut().for_each(|l| l.move_x(columns))
    }

    pub fn move_y(&mut self, rows: isize) {
        match rows.cmp(&0) {
            Less => self.line = self.line.saturating_sub(-rows as usize),
            Equal => (),
            Greater => self.line += rows as usize,
        }
    }

    pub fn block_extend_up(&mut self, rows: usize) {
        self.line = self.line.saturating_sub(rows);
        self.other_lines
            .splice(0..0, iter::repeat_n(self.first_line, rows));
    }

    pub fn block_extend_down(&mut self, rows: usize) {
        let line = self.last_line();
        self.other_lines.extend(iter::repeat_n(line, rows));
    }

    pub fn text_extend_up(&mut self, rows: usize, doc: &Document) {
        if rows == 0 {
            return;
        }
        let left_margin = self
            .other_lines
            .first()
            .map(|l| l.start)
            .unwrap_or(doc.indent_on_line(self.line));
        self.block_extend_up(rows);
        self.other_lines
            .iter_mut()
            .take(rows)
            .for_each(|l| l.left_align(left_margin));
        (self.line..)
            .zip(self.lines_mut())
            .take(rows)
            .for_each(|(i, l)| l.right_align(doc.columns_in_line(i)))
    }

    pub fn text_extend_down(&mut self, rows: usize, doc: &Document) {
        if rows == 0 {
            return;
        }
        let line = self.last_line();
        let line_lix = self.other_lines.len();
        let line_ix = self.last_line_ix();
        let left_align = (line_lix == 0).then(|| doc.indent_on_line(self.line));
        self.other_lines.extend(iter::repeat_n(line, rows));
        (line_ix..)
            .zip(self.lines_mut().skip(line_lix).take(rows))
            .for_each(|(i, l)| {
                l.right_align(doc.columns_in_line(i));
            });
        if let Some(align) = left_align {
            self.other_lines
                .iter_mut()
                .for_each(|l| l.left_align(align));
        }
    }

    pub fn extend_left(&mut self, columns: usize) {
        self.first_line.extend_left(columns)
    }
    pub fn extend_right(&mut self, columns: usize) {
        self.last_line_mut().extend_right(columns);
    }

    pub fn retract_down(&mut self, rows: usize) {
        let start = self.first_line.start;
        self.line += 1;
        if self.other_lines.len() <= rows {
            self.first_line = self.last_line();
            self.other_lines.clear();
        } else {
            self.first_line = self
                .other_lines
                .splice(..rows, [])
                .fold(self.first_line, |_, n| n);
        }
        self.first_line.start = start;
    }

    pub fn retract_up(&mut self, rows: usize) {
        let end = self.last_line().end;
        self.other_lines
            .truncate(self.other_lines.len().saturating_sub(rows));
        self.last_line_mut().end = end;
    }

    pub fn retract_right(&mut self, columns: usize) {
        self.first_line.retract_right(columns);
    }

    pub fn retract_left(&mut self, columns: usize) {
        self.last_line_mut().retract_left(columns);
    }
}

impl RangeCursorLine {
    fn right_align_to_line(&mut self, line: impl IntoIterator<Item = Grapheme>) {
        self.right_align(line.into_iter().map(|g| g.columns()).sum())
    }

    fn right_align(&mut self, end: usize) {
        self.end = end;
        self.start = self.start.min(self.end);
    }

    fn left_align_to_line(&mut self, line: impl IntoIterator<Item = Grapheme>) {
        self.left_align(
            line.into_iter()
                .filter(|g| g.is_whitespace())
                .map(|g| g.columns())
                .sum(),
        )
    }

    fn left_align(&mut self, start: usize) {
        self.start = start;
        self.end = self.end.max(self.start);
    }

    pub fn move_x(&mut self, columns: isize) {
        match columns.cmp(&0) {
            Less => {
                self.start = self.start.saturating_sub(-columns as usize);
                self.end = self.end.saturating_sub(1);
            }
            Equal => (),
            Greater => {
                self.start += columns as usize;
                self.end += columns as usize;
            }
        }
    }

    pub fn extend_left(&mut self, columns: usize) {
        self.start = self.start.saturating_sub(columns);
    }

    pub fn extend_right(&mut self, columns: usize) {
        self.end += columns;
    }

    pub fn retract_right(&mut self, columns: usize) {
        self.start = (self.start + columns).min(self.end)
    }

    pub fn retract_left(&mut self, columns: usize) {
        self.end = self.end.saturating_sub(columns).max(self.start)
    }
}
