use crate::{
    document::Document,
    editor::cursors::{CursorState, select::RangeCursorLine},
    rope::RopeSlice,
};

mod insert;
mod select;

impl Document {
    pub fn copy_text(&self) -> impl Iterator<Item = String> {
        gen {
            if let Some(cursors) = &self.cursors {
                match cursors {
                    CursorState::Insert(_) => (),
                    CursorState::Select(c) => {
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
                    CursorState::LineSelect(cursor_set) => {
                        for cursor in cursor_set.iter() {
                            let range = cursor.text_range(&self.text).unwrap();
                            yield self.text.byte_slice(range).unwrap().to_string();
                        }
                    }
                }
            }
        }
    }
}
