use std::collections::HashSet;

use crate::{
    constants::TAB_WIDTH,
    document::{Document, force_cursors},
    editor::cursors::{Cursor as _, CursorIndex, CursorState, Cursors, select::RangeCursorLine},
    ix::Ix,
    pos::Pos,
    util::indent_string,
};

mod insert;
mod select;
mod line_select;

impl Document {
    pub fn scroll_to_main_cursor(&mut self) {
        self.scroll = self
            .main_cursor_line()
            .saturating_sub(*self.view_height.lock() / 2)
    }

    pub fn cut_from_cursor(&mut self, cursor: CursorIndex) -> Option<String> {
        let text = self.copy_from_cursor(cursor)?;
        self.delete_at_cursor(cursor);
        Some(text)
    }

    pub fn copy_from_cursor(&self, cursor: CursorIndex) -> Option<String> {
        let Some(cursors) = &self.cursors else {return None};
        Some(match cursors {
            CursorState::Select(cursors) => {
                let mut s = String::new();
                let cursor = cursors.get(cursor)?;
                for (i, RangeCursorLine { start, end }) in cursor.lines_ix() {
                    let Some(line) = self.text.line(i) else { continue };
                    let range = line.column_range_to_byte_range(start..end);
                    s.extend(line.byte_slice(range).unwrap().chunks());
                    s += "\n";
                }
                s.pop();
                s
            },
            CursorState::LineSelect(c) => {
                let cursor = c.get(cursor)?;
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
                    s
                } else { String::new() }
            },
            _ => String::new()
        })
    }

    pub fn copy_text(&self) -> impl Iterator<Item = String> {
        self.cursors.iter()
            .flat_map(|c| c.indices())
            .flat_map(|i| self.copy_from_cursor(i))
    }

    pub fn copy_main_text(&self) -> Option<String> {
        self.copy_from_cursor(CursorIndex::Main)
    }

    pub fn paste_at_cursor(&mut self, text: String, ix: CursorIndex) {
        force_cursors!(self);
        let cursors = &self.cursors.as_ref().unwrap();
        use CursorState::*;

        let pos = match cursors {
            MirrorInsert(_) => return,
            Insert(c) => {
                let Some(cursor) = c.get(ix) else { return };
                cursor.pos
            }
            Select(c) => {
                let Some(cursor) = c.get(ix) else { return };
                cursor.end_pos()
            }
            LineSelect(c) => {
                let Some(cursor) = c.get(ix) else { return };
                let line = cursor.end();
                let height = cursor.height;
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
                let Some(CursorState::LineSelect(c)) = &mut self.cursors else {panic!()};
                c.get_mut(ix).unwrap().retract_down(height);

                return;
            }
        };
        let indent = self.text.context_indent_inc(pos.line);
        let indent = indent_string(indent);
        let mut lines = text.lines();
        let text = gen {
                if let Some(line) = lines.next() {
                    yield line.to_owned();
                }
                for line in lines {
                    yield format!("\n{indent}{line}");
                }
            }.collect::<String>();


        let change = self.insert_change(pos, text);
        if let Some(cursors) = &mut self.cursors
            && let CursorState::Select(c) = cursors && let Some(cursor) = c.get_mut(ix){
                cursor.collapse_to_end();
            }
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

                self.tab_line_in(line);
            }
        }
    }

    pub fn tab_lines_out(&mut self) {
        let Some(cursors) = &self.cursors else { return };

        let mut done_lines = HashSet::new();

        for index in cursors.indices() {
            let Some(range) = self.cursor_line_range(index) else { continue };
            for line in range {
                if done_lines.contains(&line) {
                    continue;
                }
                done_lines.insert(line);

                self.do_change(self.tab_out_change(Pos {
                    line,
                    column: Ix::new(TAB_WIDTH),
                }));
            }
        }
    }
}
