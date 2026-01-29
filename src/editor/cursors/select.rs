use std::{cmp::Ordering::*, iter, mem, ops::Range};

use crate::{
    document::{CursorChange, Document},
    editor::cursors::{
        Cursor, CursorSet,
        line_select::{LineCursor, LineCursors},
    },
    pos::Pos,
    rope::Rope,
    util::MapBounds,
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
    pub fn to_insert_before_line(&self, doc: &Rope) -> InsertCursors {
        self.map_to(|c| SelectCursor::to_insert_before_line(c, doc))
    }
    pub fn to_insert_after_line(&self, doc: &Rope) -> InsertCursors {
        self.map_to(|c| SelectCursor::to_insert_after_line(c, doc))
    }

    pub fn to_line_select(&self) -> LineCursors {
        self.map_to(|c| c.to_line_select())
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

    pub fn delete_ranges(&self, text: &Rope) -> impl Iterator<Item = Range<usize>> {
        self.iter().flat_map(|c| c.delete_ranges(text))
    }
}

impl Cursor for SelectCursor {
    fn apply_change(&mut self, #[allow(unused)] change: CursorChange) {}
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
    pub fn one_pos(pos: Pos) -> Self {
        Self {
            line: pos.line,
            first_line: RangeCursorLine {
                start: pos.column,
                end: pos.column,
            },
            other_lines: Vec::new(),
        }
    }

    pub fn range(range: Range<usize>, doc: &Document) -> Self {
        let Range { start, end } = range;
        let Pos { line, column } = doc.text().pos_of_byte_pos(start).unwrap();
        let Pos {
            line: eline,
            column: ecolumn,
        } = doc.text().pos_of_byte_pos(end).unwrap();
        if line != eline {
            todo!()
        }

        Self {
            line,
            first_line: RangeCursorLine {
                start: column,
                end: ecolumn,
            },
            other_lines: Vec::new(),
        }
    }

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

    pub(super) fn to_insert_before_line(&self, doc: &Rope) -> InsertCursor {
        let Self { line, .. } = *self;
        InsertCursor::forward(Pos {
            line,
            column: doc.indent_on_line(line),
        })
    }

    pub(super) fn to_insert_after_line(&self, doc: &Rope) -> InsertCursor {
        let line = self.last_line_ix();
        InsertCursor::forward(Pos {
            line,
            column: doc.columns_in_line(line),
        })
    }

    pub fn to_line_select(&self) -> LineCursor {
        LineCursor {
            line: self.line,
            height: self.other_lines.len() + 1,
        }
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

    pub fn lines(&self) -> impl Iterator<Item = RangeCursorLine> {
        iter::once(self.first_line).chain(self.other_lines.iter().copied())
    }

    pub fn lines_ix(&self) -> impl Iterator<Item = (usize, RangeCursorLine)> {
        (self.line..).zip(self.lines())
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

    pub fn text_extend_up(&mut self, rows: usize, text: &Rope) {
        if rows == 0 {
            return;
        }
        let left_margin = self
            .other_lines
            .first()
            .map(|l| l.start)
            .unwrap_or(text.indent_on_line(self.line));

        self.line = self.line.saturating_sub(rows);
        self.other_lines
            .splice(0..0, iter::repeat_n(self.first_line, rows));
        self.other_lines
            .iter_mut()
            .take(rows)
            .for_each(|l| l.left_align(left_margin));
        let first_line_ix = self.line;
        let num_other_lines = self.other_lines.len();
        (first_line_ix..)
            .zip(self.lines_mut())
            .take((rows + 1).min(num_other_lines))
            .for_each(|(i, l)| l.right_align(text.columns_in_line(i), i != first_line_ix))
    }

    pub fn text_extend_down(&mut self, rows: usize, text: &Rope) {
        if rows == 0 {
            return;
        }
        let first_line_ix = self.line;
        let line = self.last_line();
        let line_lix = self.other_lines.len();
        let line_ix = self.last_line_ix();
        let left_align = (line_lix == 0).then(|| text.indent_on_line(self.line));
        self.other_lines.extend(iter::repeat_n(line, rows));
        (line_ix..)
            .zip(self.lines_mut().skip(line_lix).take(rows))
            .for_each(|(i, l)| {
                l.right_align(text.columns_in_line(i), i != first_line_ix);
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
        let rows = rows.min(self.other_lines.len());
        if rows == 0 {
            return;
        }
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

    pub fn inspect_range(&self) -> (Pos, Pos) {
        (
            Pos {
                line: self.line,
                column: self.first_line.start,
            },
            Pos {
                line: self.line + self.other_lines.len(),
                column: self.last_line().end,
            },
        )
    }

    pub fn delete_ranges(&self, text: &Rope) -> impl Iterator<Item = Range<usize>> {
        let mut so_far: Option<Range<usize>> = None;
        gen move {
            for (i, line) in self.lines_ix() {
                let Some(next) = text.line(i) else { continue };
                let next = next
                    .column_range_to_byte_range(line.start..line.end)
                    .map_bounds(|b| b + text.byte_of_line(i).unwrap());
                let Some(so_far) = &mut so_far else {
                    so_far = Some(next);
                    continue;
                };

                if text
                    .byte_slice(so_far.end..next.start)
                    .unwrap()
                    .graphemes()
                    .all(|g| g.is_whitespace())
                {
                    so_far.end = next.end;
                } else {
                    yield mem::replace(so_far, next);
                }
            }
            if let Some(so_far) = so_far {
                yield so_far;
            }
        }
    }
}

impl RangeCursorLine {
    fn right_align(&mut self, mut end: usize, force: bool) {
        if !force {
            end = self.start.max(end);
        }
        self.end = end;
        self.start = self.start.min(self.end);
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
