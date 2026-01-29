use crate::{document::Document, editor::cursors::CursorState};

impl Document {
    pub fn select(&mut self) {
        let Some(cursors) = &self.cursors else {
            return;
        };
        match cursors {
            CursorState::Insert(c) => self.cursors = Some(c.to_select().into()),
            CursorState::Select(_) => (),
            CursorState::LineSelect(c) => self.cursors = Some(c.to_select(&self.text).into()),
        }
    }

    pub fn backspace(&mut self) {
        self.do_insert(|doc, pos| doc.backspace_change(pos))
    }

    pub fn insert(&mut self, str: &str) {
        self.do_insert(|doc, pos| doc.insert_change(pos, str.to_owned()))
    }

    pub fn insert_return(&mut self) {
        self.do_insert(|doc, pos| doc.return_change(pos))
    }

    pub fn insert_tab(&mut self) {
        if let Some(c) = &self.cursors {
            match c {
                CursorState::Insert(cursors) => self.cursors = Some(cursors.clone().tab().into()),
                CursorState::Select(_) => todo!(),
                CursorState::LineSelect(_) => todo!(),
            }
        }
    }
}
