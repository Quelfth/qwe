use std::{cmp::Ordering::*, ops::Range};

use crate::{
    document::CursorChange,
    editor::cursors::{
        Cursor, CursorSet,
        select::{RangeCursorLine, SelectCursor, SelectCursors},
    },
    ix::{Byte, Ix, Line},
    pos::{Pos, Region},
    rope::Rope,
};

use super::insert::*;

pub type LineCursors = CursorSet<LineCursor>;

impl LineCursors {
    pub fn move_y(&mut self, rows: Ix<Line, isize>) {
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

    pub fn to_insert_around_in(&self, _: &Rope) -> InsertCursors {
        todo!()
    }

    pub fn to_insert_around_out(&self, _: &Rope) -> InsertCursors {
        todo!()
    }

    pub fn delete_ranges(&self, doc: &Rope) -> impl Iterator<Item = Range<Ix<Byte>>> {
        self.iter().filter_map(|c| c.text_range(doc))
    }

    pub fn line_split(&mut self) {
        let mut iter = self.main.line_split();
        let m = iter.next().unwrap();
        self.others = iter
            .chain(self.others.iter().flat_map(|c| c.line_split()))
            .collect();
        self.main = m;
    }
}

#[derive(Copy, Clone, Default)]
pub struct LineCursor {
    pub line: Ix<Line>,
    pub height: Ix<Line>,
}

impl LineCursor {
    /// The first line after the selection (this is exclusive)
    pub fn end(&self) -> Ix<Line> {
        self.line + self.height
    }

    fn to_insert_before(self, doc: &Rope) -> InsertCursor {
        let Self { line, .. } = self;
        InsertCursor::forward(Pos {
            line,
            column: doc.indent_on_line(line),
        })
    }
    fn to_insert_after(self, doc: &Rope) -> InsertCursor {
        let Self { line, height } = self;
        let line = line + height.max(Ix::new(1)) - Ix::new(1);
        InsertCursor::forward(Pos {
            line,
            column: doc.columns_in_line(line),
        })
    }

    pub fn to_select(self, doc: &Rope) -> SelectCursor {
        let Self { line, height } = self;
        let height = height.max(Ix::new(1));
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
            other_lines: (line + Ix::new(1)..end_line)
                .map(|l| RangeCursorLine {
                    start,
                    end: doc.columns_in_line(l),
                })
                .collect(),
        }
    }

    pub fn move_y(&mut self, rows: Ix<Line, isize>) {
        match rows.cmp(&Ix::new(0)) {
            Less => self.line = self.line.saturating_sub((-rows).to_usize()),
            Equal => (),
            Greater => self.line += rows.to_usize(),
        }
    }

    pub fn extend_up(&mut self, rows: Ix<Line>) {
        let rows = rows.min(self.line);
        self.line -= rows;
        self.height += rows;
    }

    pub fn extend_down(&mut self, rows: Ix<Line>) {
        self.height += rows;
    }

    pub fn retract_down(&mut self, rows: Ix<Line>) {
        self.line += rows;
        self.height = self.height.saturating_sub(rows);
    }

    pub fn retract_up(&mut self, rows: Ix<Line>) {
        self.height = self.height.saturating_sub(rows);
    }

    pub fn text_range(&self, text: &Rope) -> Option<Range<Ix<Byte>>> {
        if self.height == Ix::new(0) {
            return None;
        }
        let start = text.byte_of_line(self.line)?;
        let end_line = self.line + self.height - Ix::new(1);
        if end_line >= text.line_len() {
            return Some(start..text.byte_len());
        }
        let mut end =
            text.byte_of_line(end_line).unwrap() + text.line(end_line).unwrap().byte_len();
        if text
            .byte_slice(end..=end)
            .is_some_and(|b| b.to_string() == "\r")
        {
            end += Ix::new(1);
        }
        if text
            .byte_slice(end..=end)
            .is_some_and(|b| b.to_string() == "\n")
        {
            end += Ix::new(1);
        }
        Some(start..end)
    }

    pub fn inspect_range(&self) -> Region {
        Region::Line(self.line..self.line + self.height)
    }

    pub fn line_split(&self) -> impl Iterator<Item = Self> {
        gen move {
            if self.height == Ix::new(0) {
                yield *self;
                return;
            }
            for line in self.line..self.line + self.height {
                yield Self {
                    line,
                    height: Ix::new(1),
                };
            }
        }
    }
}

impl Cursor for LineCursor {
    fn apply_change(&mut self, c: CursorChange, _: &Rope) {
        let start = c.apply_to_line(self.line);
        let end = c.apply_to_line(self.line + self.height);

        self.line = start;
        self.height = end.saturating_sub(start);
    }
}
