use std::{cmp::Ordering::*, collections::HashMap, iter, mem, ops::Range};

use crate::{
    document::{CursorChange, CursorChangeBias},
    editor::cursors::{
        Cursor, CursorSet,
        line_select::{LineCursor, LineCursors},
        mirror_insert::{MirrorInsertCursor, MirrorInsertCursors},
    },
    ix::{Byte, Column, Ix, Line, MappedRange, ixto},
    pos::{Pos, Region},
    rope::Rope,
    util::MapBounds,
};

use super::insert::*;

mod incremental;

pub type SelectCursors = CursorSet<SelectCursor>;

impl SelectCursors {
    pub fn to_insert_before(&self) -> InsertCursors {
        self.map_to(SelectCursor::to_insert_before)
    }
    pub fn to_insert_after(&self) -> InsertCursors {
        self.map_to(SelectCursor::to_insert_after)
    }
    pub fn to_insert_before_line(&self, doc: &Rope) -> InsertCursors {
        self.map_to(|c| c.to_insert_before_line(doc))
    }
    pub fn to_insert_after_line(&self, doc: &Rope) -> InsertCursors {
        self.map_to(|c| c.to_insert_after_line(doc))
    }

    pub fn to_line_select(&self) -> LineCursors {
        self.map_to(|c| c.to_line_select())
    }

    pub fn to_mirror_insert_in(&self) -> MirrorInsertCursors {
        self.map_to(SelectCursor::to_mirror_insert_in)
    }
    pub fn to_mirror_insert_out(&self) -> MirrorInsertCursors {
        self.map_to(SelectCursor::to_mirror_insert_out)
    }

    pub fn block_select(&mut self) {
        self.iter_mut().for_each(|s| s.block_select())
    }

    pub fn text_select(&mut self, text: &Rope) {
        self.iter_mut().for_each(|s| s.text_select(text))
    }

    pub fn move_x(&mut self, columns: Ix<Column, isize>) {
        for cursor in self.iter_mut() {
            cursor.move_x(columns);
        }
    }

    pub fn move_y(&mut self, rows: Ix<Line, isize>) {
        for cursor in self.iter_mut() {
            cursor.move_y(rows);
        }
    }

    pub fn delete_ranges(&self, text: &Rope) -> impl Iterator<Item = Range<Ix<Byte>>> {
        self.iter().flat_map(|c| c.delete_ranges(text))
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

impl Cursor for SelectCursor {
    fn apply_change(&mut self, change: CursorChange, text: &Rope) {
        let line = self.line;
        let selections = self
            .lines()
            .map(|RangeCursorLine { start, end }| {
                Pos {
                    line,
                    column: start,
                }..Pos { line, column: end }
            })
            .map(|Range { start, end }| {
                use CursorChangeBias::*;
                change.apply(start, Left)..change.apply(end, Right)
            })
            .collect::<Vec<_>>();
        let mut lines = HashMap::<_, Vec<_>>::new();

        for sel in &selections {
            if sel.start.line == sel.end.line {
                lines
                    .entry(sel.start.line)
                    .or_default()
                    .push(sel.start.column..sel.end.column);
            } else {
                lines.entry(sel.start.line).or_default().push(
                    sel.start.column..text.columns_in_line(sel.start.line).max(sel.start.column),
                );
                let indent = (sel.start.line..=sel.end.line)
                    .map(|l| text.indent_on_line(l))
                    .min()
                    .unwrap_or(Ix::new(0));
                for line in sel.start.line + Ix::new(1)..sel.end.line {
                    lines
                        .entry(line)
                        .or_default()
                        .push(indent..text.columns_in_line(line).max(indent));
                }
                lines
                    .entry(sel.end.line)
                    .or_default()
                    .push(indent.min(sel.end.column)..sel.end.column);
            }
        }

        let min_line = *lines.keys().min().unwrap();
        let max_line = *lines.keys().max().unwrap();
        let mut new_lines = Vec::new();

        for line in min_line..=max_line {
            match lines.get(&line) {
                Some(list) => {
                    let start = list.iter().map(|l| l.start).min().unwrap();
                    let end = list.iter().map(|l| l.end).max().unwrap();
                    new_lines.push(RangeCursorLine { start, end });
                }
                None => {
                    let last = new_lines.last().unwrap().start;
                    new_lines.push(RangeCursorLine {
                        start: last,
                        end: last,
                    })
                }
            }
        }

        *self = Self {
            line,
            first_line: *new_lines.first().unwrap(),
            other_lines: new_lines.into_iter().skip(1).collect(),
        }
    }

    fn location_cmp(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.start_pos().cmp(&right.start_pos())
    }

    fn collapse_to_start(&mut self) {
        self.other_lines.clear();
        self.first_line.end = self.first_line.start
    }

    fn collapse_to_end(&mut self) {
        self.line += Ix::new(self.other_lines.len());
        self.first_line = self.last_line();
        self.other_lines.clear();
        self.first_line.start = self.first_line.end
    }

    fn line_range(&self) -> Range<Ix<Line>> {
        self.line..self.line + Ix::new(self.other_lines.len() + 1)
    }

    fn syntax_extend(&mut self, text: &Rope, tree: &tree_sitter::Tree) {
        try {
            let start = self.start_pos();
            let end = self.end_pos();

            let start_byte = text.byte_pos_of_pos(start).ok()?.inner();
            let end_byte = text.byte_pos_of_pos(end).ok()?.inner();

            let mut node = tree.root_node().descendant_for_byte_range(start_byte, end_byte)?;
            while node.range().start_byte == start_byte && node.range().end_byte == end_byte {
                node = node.parent()?;
            }

            let new_start = text.pos_of_byte_pos(Ix::new(node.range().start_byte))?;
            let new_end = text.pos_of_byte_pos(Ix::new(node.range().end_byte))?;

            *self = Self::range(new_start..new_end, text);
        };
    }
}

#[derive(Clone, Default)]
pub struct SelectCursor {
    pub line: Ix<Line>,
    pub first_line: RangeCursorLine,
    pub other_lines: Vec<RangeCursorLine>,
}

#[derive(Copy, Clone, Default)]
pub struct RangeCursorLine {
    pub start: Ix<Column>,
    pub end: Ix<Column>,
}

impl RangeCursorLine {
    pub fn point(column: Ix<Column>) -> Self {
        Self {
            start: column,
            end: column,
        }
    }
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

    pub fn byte_range(range: Range<Ix<Byte>>, text: &Rope) -> Self {
        let Range { start, end } = range;
        let start = text.pos_of_byte_pos(start).unwrap();
        let end = text.pos_of_byte_pos(end).unwrap();
        Self::range(start..end, text)
    }

    pub fn range(range: Range<Pos>, text: &Rope) -> Self {
        let Range { start, end } = range;
        if start.line == end.line {
            return Self {
                line: start.line,
                first_line: RangeCursorLine {
                    start: start.column,
                    end: end.column,
                },
                other_lines: Vec::new(),
            };
        }
        let indent = (start.line..=end.line)
            .flat_map(|i| text.line_has_content(i).then(|| text.indent_on_line(i)))
            .min()
            .unwrap_or(Ix::new(0))
            .min(end.column);

        Self {
            line: start.line,
            first_line: RangeCursorLine {
                start: start.column,
                end: text.columns_in_line(start.line),
            },
            other_lines: (start.line + Ix::new(1)..end.line)
                .map(|i| RangeCursorLine {
                    start: indent,
                    end: indent.max(text.columns_in_line(i)),
                })
                .chain(iter::once(RangeCursorLine {
                    start: indent,
                    end: end.column,
                }))
                .collect(),
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
            column: doc.context_indent_inc(line),
        })
    }

    pub(super) fn to_insert_after_line(&self, doc: &Rope) -> InsertCursor {
        let line = self.last_line_ix();
        InsertCursor::forward(Pos {
            line,
            column: doc.context_columns_in_line(line),
        })
    }

    pub fn to_line_select(&self) -> LineCursor {
        LineCursor {
            line: self.line,
            height: Ix::new(self.other_lines.len()) + Ix::new(1),
        }
    }

    fn to_mirror_insert_in(&self) -> MirrorInsertCursor {
        MirrorInsertCursor {
            forward: self.start_pos(),
            reverse: self.end_pos(),
        }
    }

    fn to_mirror_insert_out(&self) -> MirrorInsertCursor {
        MirrorInsertCursor {
            forward: self.end_pos(),
            reverse: self.start_pos(),
        }
    }

    pub fn on_line(&self, line: Ix<Line>) -> Option<RangeCursorLine> {
        if line < self.line {
            return None;
        }

        let line = line - self.line;
        if line == Ix::new(0) {
            return Some(self.first_line);
        }

        self.other_lines.get(line.inner() - 1).copied()
    }

    fn last_line(&self) -> RangeCursorLine {
        self.other_lines.last().copied().unwrap_or(self.first_line)
    }

    fn last_line_mut(&mut self) -> &mut RangeCursorLine {
        self.other_lines.last_mut().unwrap_or(&mut self.first_line)
    }

    fn last_line_ix(&self) -> Ix<Line> {
        self.line + Ix::new(self.other_lines.len())
    }

    pub fn start_pos(&self) -> Pos {
        Pos {
            line: self.line,
            column: self.first_line.start,
        }
    }

    pub fn end_pos(&self) -> Pos {
        Pos {
            line: self.last_line_ix(),
            column: self.last_line().end,
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = RangeCursorLine> {
        iter::once(self.first_line).chain(self.other_lines.iter().copied())
    }

    pub fn lines_ix(&self) -> impl Iterator<Item = (Ix<Line>, RangeCursorLine)> {
        (self.line..).zip(self.lines())
    }

    fn lines_mut(&mut self) -> impl Iterator<Item = &mut RangeCursorLine> {
        iter::once(&mut self.first_line).chain(&mut self.other_lines)
    }

    pub fn block_select(&mut self) {
        let a = self.first_line.start;
        let b = self.last_line().end;

        let range = RangeCursorLine {
            start: a.min(b),
            end: a.max(b)
        };
        for line in self.lines_mut() {
            *line = range;
        }
    }

    pub fn text_select(&mut self, text: &Rope) {
        let line_range = self.line_range();
        let indent = line_range.clone().map(|line| text.context_indent_inc(line)).min().unwrap_or(Ix::ZERO);
        if self.first_line.start < indent {
            self.first_line.start = indent;
        }
        for line in &mut self.other_lines {
            line.start = indent;
        }

        let last_line = self.last_line_mut();

        let last_line_len = text.columns_in_line(line_range.end.saturating_sub(Ix::new(1)));
        if last_line.end > last_line_len {
            last_line.end = last_line_len;
        }
        let other_lines_len = self.other_lines.len();
        for (line, i) in self.lines_mut().zip(line_range).take(other_lines_len) {
            line.end = text.columns_in_line(i);
        }
    }

    pub fn move_x(&mut self, columns: Ix<Column, isize>) {
        self.lines_mut().for_each(|l| l.move_x(columns))
    }

    pub fn move_y(&mut self, rows: Ix<Line, isize>) {
        match rows.cmp(&Ix::new(0)) {
            Less => self.line = self.line.saturating_sub((-rows).to_usize()),
            Equal => (),
            Greater => self.line += rows.to_usize(),
        }
    }

    pub fn text_extend_up(&mut self, rows: Ix<Line>, text: &Rope) {
        if rows == Ix::new(0) {
            return;
        }
        let left_margin = self
            .other_lines
            .first()
            .map(|l| l.start)
            .unwrap_or(text.indent_on_line(self.line));

        self.line = self.line.saturating_sub(rows);
        self.other_lines
            .splice(0..0, iter::repeat_n(self.first_line, rows.inner()));
        self.other_lines
            .iter_mut()
            .take(rows.inner())
            .for_each(|l| l.left_align(left_margin));
        let first_line_ix = self.line;
        let num_other_lines = Ix::new(self.other_lines.len());
        (first_line_ix..)
            .zip(self.lines_mut())
            .take((rows + Ix::new(1)).min(num_other_lines).inner())
            .for_each(|(i, l)| l.right_align(text.columns_in_line(i), i != first_line_ix))
    }

    pub fn text_extend_down(&mut self, rows: Ix<Line>, text: &Rope) {
        ixto!(rows);
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

    pub fn extend_left(&mut self, columns: Ix<Column>) {
        self.first_line.extend_left(columns)
    }
    pub fn extend_right(&mut self, columns: Ix<Column>) {
        self.last_line_mut().extend_right(columns);
    }

    pub fn retract_down(&mut self, rows: Ix<Line>) {
        let rows = rows.min(Ix::new(self.other_lines.len()));
        if rows == Ix::new(0) {
            return;
        }
        let start = self.first_line.start;
        self.line += Ix::new(1);
        if Ix::new(self.other_lines.len()) <= rows {
            self.first_line = self.last_line();
            self.other_lines.clear();
        } else {
            self.first_line = self
                .other_lines
                .splice(MappedRange::new(..rows), [])
                .fold(self.first_line, |_, n| n);
        }
        self.first_line.start = start;
    }

    pub fn retract_up(&mut self, rows: Ix<Line>) {
        let end = self.last_line().end;
        self.other_lines
            .truncate(self.other_lines.len().saturating_sub(rows.inner()));
        self.last_line_mut().end = end;
    }

    pub fn retract_right(&mut self, columns: Ix<Column>) {
        self.first_line.retract_right(columns);
    }

    pub fn retract_left(&mut self, columns: Ix<Column>) {
        self.last_line_mut().retract_left(columns);
    }

    pub fn inspect_range(&self) -> Region {
        Region::Pos(
            Pos {
                line: self.line,
                column: self.first_line.start,
            }..Pos {
                line: self.line + Ix::new(self.other_lines.len()),
                column: self.last_line().end,
            },
        )
    }

    pub fn delete_ranges(&self, text: &Rope) -> impl Iterator<Item = Range<Ix<Byte>>> {
        let mut so_far = None;
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

    fn line_split(&self) -> impl Iterator<Item = Self> {
        gen move {
            for (line, first_line) in self.lines_ix() {
                yield Self {
                    line,
                    first_line,
                    ..Default::default()
                };
            }
        }
    }
}

impl RangeCursorLine {
    fn right_align(&mut self, mut end: Ix<Column>, force: bool) {
        if !force {
            end = self.start.max(end);
        }
        self.end = end;
        self.start = self.start.min(self.end);
    }

    fn left_align(&mut self, start: Ix<Column>) {
        self.start = start;
        self.end = self.end.max(self.start);
    }

    pub fn move_x(&mut self, columns: Ix<Column, isize>) {
        match columns.cmp(&Ix::new(0)) {
            Less => {
                self.start = self.start.saturating_sub((-columns).to_usize());
                self.end = self.end.saturating_sub(Ix::new(1));
            }
            Equal => (),
            Greater => {
                self.start += columns.to_usize();
                self.end += columns.to_usize();
            }
        }
    }

    pub fn extend_left(&mut self, columns: Ix<Column>) {
        self.start = self.start.saturating_sub(columns);
    }

    pub fn extend_right(&mut self, columns: Ix<Column>) {
        self.end += columns;
    }

    pub fn retract_right(&mut self, columns: Ix<Column>) {
        self.start = (self.start + columns).min(self.end)
    }

    pub fn retract_left(&mut self, columns: Ix<Column>) {
        self.end = self.end.saturating_sub(columns).max(self.start)
    }
}
