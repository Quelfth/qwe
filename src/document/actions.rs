use crate::{
    document::{Document, force_cursors},
    editor::cursors::{CursorIndex, CursorState, select::RangeCursorLine},
    ix::Ix,
    pos::Pos,
    util::indent_string,
};

mod insert;
mod select;

impl Document {
    pub fn scroll_to_main_cursor(&mut self) {
        if let Some(cursors) = &self.cursors {
            self.scroll = match cursors {
                CursorState::MirrorInsert(cursors) => cursors.main().forward.line,
                CursorState::Insert(cursors) => cursors.main().pos.line,
                CursorState::Select(cursors) => cursors.main().start_pos().line,
                CursorState::LineSelect(cursors) => cursors.main().line,
            }
        }
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
                            }
                            yield s;
                        }
                    }
                    LineSelect(cursor_set) => {
                        for cursor in cursor_set.iter() {
                            if let Some(range) = cursor.text_range(&self.text) {
                                yield self.text.byte_slice(range).unwrap().to_string();
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
}
