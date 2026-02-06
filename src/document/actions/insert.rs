use crate::{
    document::Document,
    editor::cursors::{CursorState, mirror_insert::InsertDirection},
    util::mirror_string,
};

impl Document {
    pub fn select(&mut self) {
        let Some(cursors) = &self.cursors else {
            return;
        };
        use CursorState::*;
        match cursors {
            MirrorInsert(c) => self.cursors = Some(c.to_select(&self.text).into()),
            Insert(c) => self.cursors = Some(c.to_select().into()),
            Select(_) => (),
            LineSelect(c) => self.cursors = Some(c.to_select(&self.text).into()),
        }
    }

    pub fn backspace(&mut self) {
        self.do_insert(|doc, pos, dir| match dir {
            InsertDirection::Forward => doc.backspace_change(pos),
            InsertDirection::Reverse => doc.reverse_backspace_change(pos),
        })
    }

    pub fn insert(&mut self, str: &str) {
        self.do_insert(|doc, pos, dir| {
            let text = match dir {
                InsertDirection::Forward => str.to_owned(),
                InsertDirection::Reverse => mirror_string(str),
            };
            doc.insert_change(pos, text)
        })
    }

    pub fn insert_return(&mut self) {
        self.do_insert(|doc, pos, _| doc.return_change(pos))
    }

    pub fn insert_tab(&mut self) {
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(cursors) => self.cursors = Some(cursors.clone().tab().into()),
                Select(_) => todo!(),
                LineSelect(_) => todo!(),
            }
        }
    }
}
