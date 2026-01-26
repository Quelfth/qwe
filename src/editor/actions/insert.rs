use crate::editor::{Editor, cursors::CursorState};

impl Editor {
    pub fn select(&mut self) {
        match &self.cursors {
            CursorState::Insert(c) => self.cursors = c.to_select().into(),
            CursorState::Select(_) => (),
            CursorState::LineSelect(c) => self.cursors = c.to_select(&self.doc).into(),
        }
    }

    pub fn backspace(&mut self) {
        match &self.cursors {
            CursorState::Insert(cursor) => {
                self.do_insert(&cursor.clone(), |doc, pos| doc.backspace_change(pos))
            }
            CursorState::Select(_) => todo!(),
            CursorState::LineSelect(_) => todo!(),
        }
    }

    pub fn insert(&mut self, str: &str) {
        match &self.cursors {
            CursorState::Insert(cursors) => self.do_insert(&cursors.clone(), |doc, pos| {
                doc.insert_change(pos, str.to_owned())
            }),
            CursorState::Select(_) => todo!(),
            CursorState::LineSelect(_) => todo!(),
        }
    }

    pub fn insert_return(&mut self) {
        match &self.cursors {
            CursorState::Insert(cursors) => {
                self.do_insert(&cursors.clone(), |doc, pos| doc.return_change(pos))
            }

            CursorState::Select(_) => todo!(),
            CursorState::LineSelect(_) => todo!(),
        }
    }

    pub fn insert_tab(&mut self) {
        match &self.cursors {
            CursorState::Insert(cursors) => self.cursors = cursors.clone().tab().into(),
            CursorState::Select(_) => todo!(),
            CursorState::LineSelect(_) => todo!(),
        }
    }
}
