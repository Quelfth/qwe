use std::collections::HashSet;

use crate::{
    constants::TAB_WIDTH,
    document::{Document, force_cursors},
    editor::cursors::{CursorIndex, CursorState, Cursors, select::RangeCursorLine},
    ix::Ix,
    pos::Pos,
    util::indent_string,
};

mod insert;
mod select;

impl Document {
    pub fn scroll_to_main_cursor(&mut self) {
        self.scroll = self
            .main_cursor_line()
            .saturating_sub(*self.view_height.lock() / 2)
    }

    pub fn copy_text(&self) -> impl Iterator<Item = String> {
        gen {
            if let Some(cursors) = &self.cursors {
                use CursorState::*;
                match cursors {
                    MirrorInsert(_) => todo!(),
                    Insert(_) => (),
                    Select(c) => {
                        for cursor in c.iter() {
                            let mut s = String::new();
                            for (i, RangeCursorLine { start, end }) in cursor.lines_ix() {
                                let Some(line) = self.text.line(i) else {
                                    continue;
                                };
                                let range = line.column_range_to_byte_range(start..end);
                                s.extend(line.byte_slice(range).unwrap().chunks());
                                s += "\n";
                            }
                            s.pop();
                            yield s;
                        }
                    }
                    LineSelect(cursor_set) => {
                        for cursor in cursor_set.iter() {
                            if let Some(range) = cursor.text_range(&self.text) {
                                let mut s = String::new();
                                let slice = self.text.byte_slice(range).unwrap();
                                let indent = cursor
                                    .lines()
                                    .map(|l| self.text.indent_on_line(l))
                                    .min()
                                    .unwrap_or_default();
                                for line in slice.lines() {
                                    let indent = line.columns_to_bytes(indent);
                                    let cropped = &line.to_string()[indent.inner()..];
                                    s += cropped;
                                    s += "\n";
                                }
                                yield s.to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn paste_at_cursor(&mut self, text: String, cursor: CursorIndex) {
        force_cursors!(self);
        let cursors = &self.cursors.as_ref().unwrap();
        use CursorState::*;

        let pos = match cursors {
            MirrorInsert(_) => todo!(),
            Insert(c) => {
                let Some(cursor) = c.get(cursor) else { return };
                cursor.pos
            }
            Select(c) => {
                let Some(cursor) = c.get(cursor) else { return };
                cursor.end_pos()
            }
            LineSelect(c) => {
                let Some(cursor) = c.get(cursor) else { return };
                let line = cursor.end();
                let indent = line
                    .checked_sub(Ix::new(1))
                    .map(|line| self.text.indent_on_line(line))
                    .unwrap_or(Ix::new(0));
                let pos = Pos {
                    line,
                    column: Ix::new(0),
                };
                let indent = indent_string(indent);
                let text = text
                    .lines()
                    .map(|l| format!("{indent}{l}\n"))
                    .collect::<String>();
                let change = self.insert_change(pos, text);
                self.do_change(change);

                return;
            }
        };
        let indent = self.text.indent_on_line(pos.line);
        let indent = indent_string(indent);
        let mut lines = text.lines();
        let text = gen {
            if let Some(line) = lines.next() {
                yield line.to_owned();
            }
            for line in lines {
                yield format!("{indent}{line}");
            }
        }
        .collect();
        let change = self.insert_change(pos, text);
        self.do_change(change)
    }

    pub fn cursor_line_split(&mut self) {
        if let Some(cursors) = &mut self.cursors {
            match cursors {
                CursorState::Select(cursors) => cursors.line_split(),
                CursorState::LineSelect(cursors) => cursors.line_split(),
                _ => (),
            }
        }
    }

    pub fn incremental_select(&mut self) {
        let cursors = force_cursors!(self);
        match cursors {
            CursorState::MirrorInsert(_) => (),
            CursorState::Insert(_) => (),
            CursorState::Select(c) => {
                for c in c.iter_mut() {
                    c.incremental_select(&self.text);
                }
            }
            CursorState::LineSelect(_) => todo!(),
        }
    }

    pub fn tab_lines_in(&mut self) {
        let Some(cursors) = &self.cursors else { return };

        let mut done_lines = HashSet::new();

        for index in cursors.indices() {
            for line in self.cursors.as_ref().unwrap().line_range_at(index) {
                if done_lines.contains(&line) {
                    continue;
                }
                done_lines.insert(line);

                if self
                    .text
                    .line(line)
                    .is_none_or(|l| l.chars().all(char::is_whitespace))
                {
                    continue;
                }

                self.direct_insert(
                    Pos {
                        line,
                        column: Ix::new(0),
                    },
                    &indent_string(Ix::new(TAB_WIDTH)),
                );
            }
        }
    }

    pub fn tab_lines_out(&mut self) {
        let Some(cursors) = &self.cursors else { return };

        let mut done_lines = HashSet::new();

        for index in cursors.indices() {
            for line in self.cursors.as_ref().unwrap().line_range_at(index) {
                if done_lines.contains(&line) {
                    continue;
                }
                done_lines.insert(line);

                if self
                    .text
                    .line(line)
                    .is_none_or(|l| l.chars().all(char::is_whitespace))
                {
                    continue;
                }

                self.do_change(self.tab_out_change(Pos {
                    line,
                    column: Ix::new(TAB_WIDTH),
                }));
            }
        }
    }
}
