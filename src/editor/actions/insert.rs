use crate::editor::{Editor, cursors::CursorState};

impl Editor {
    pub fn select(&mut self) {
        match &self.cursors {
            CursorState::Insert(cursors) => self.cursors = cursors.to_select().into(),
            CursorState::Select(_) => (),
            CursorState::LineSelect(_) => todo!(),
        }
    }

    pub fn backspace(&mut self) {
        match &self.cursors {
            CursorState::Insert(cursor) => {
                self.change_insert(&cursor.clone(), |doc, pos| doc.backspace_change(pos))
            }
            CursorState::Select(_) => todo!(),
            CursorState::LineSelect(_) => todo!(),
        }
    }

    pub fn insert(&mut self, str: &str) {
        match &self.cursors {
            CursorState::Insert(cursors) => self.change_insert(&cursors.clone(), |doc, pos| {
                doc.insert_change(pos, str.to_owned())
            }),
            CursorState::Select(_) => todo!(),
            CursorState::LineSelect(_) => todo!(),
        }
    }

    pub fn r#return(&mut self) {
        match &self.cursors {
            CursorState::Insert(cursors) => {
                self.change_insert(&cursors.clone(), |doc, pos| doc.return_change(pos))
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
