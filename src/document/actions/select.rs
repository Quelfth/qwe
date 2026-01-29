use crate::{
    document::{Document, force_cursors},
    editor::cursors::CursorState,
};

impl Document {
    pub fn insert_before(&mut self) {
        self.history.checkpoint();
        if let Some(c) = &self.cursors {
            match c {
                CursorState::Insert(_) => (),
                CursorState::Select(cursors) => {
                    self.cursors = Some(cursors.to_insert_before().into())
                }
                CursorState::LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_before(&self.text).into())
                }
            }
        }
    }
    pub fn insert_after(&mut self) {
        self.history.checkpoint();
        if let Some(c) = &self.cursors {
            match c {
                CursorState::Insert(_) => (),
                CursorState::Select(cursors) => {
                    self.cursors = Some(cursors.to_insert_after().into())
                }
                CursorState::LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_after(&self.text).into())
                }
            }
        }
    }
    pub fn insert_before_line(&mut self) {
        self.history.checkpoint();
        if let Some(c) = &self.cursors {
            match c {
                CursorState::Insert(_) => (),
                CursorState::Select(cursors) => {
                    self.cursors = Some(cursors.to_insert_before_line(&self.text).into())
                }
                CursorState::LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_before(&self.text).into())
                }
            }
        }
    }
    pub fn insert_after_line(&mut self) {
        self.history.checkpoint();
        if let Some(c) = &self.cursors {
            match c {
                CursorState::Insert(_) => (),
                CursorState::Select(cursors) => {
                    self.cursors = Some(cursors.to_insert_after_line(&self.text).into())
                }
                CursorState::LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_after(&self.text).into())
                }
            }
        }
    }
    pub fn line_select(&mut self) {
        if let Some(c) = &self.cursors {
            match c {
                CursorState::Insert(c) => self.cursors = Some(c.to_line_select().into()),
                CursorState::Select(c) => self.cursors = Some(c.to_line_select().into()),
                CursorState::LineSelect(_) => (),
            }
        }
    }

    pub fn move_x(&mut self, columns: isize) {
        if let Some(c) = &mut self.cursors {
            c.move_x(columns)
        }
    }

    pub fn move_y(&mut self, rows: isize) {
        force_cursors!(self).move_y(rows);
    }

    pub fn text_extend_up(&mut self, rows: usize) {
        force_cursors!(self).text_extend_up(rows, &self.text)
    }
    pub fn text_extend_down(&mut self, rows: usize) {
        force_cursors!(self).text_extend_down(rows, &self.text)
    }

    pub fn extend_left(&mut self, rows: usize) {
        force_cursors!(self).extend_left(rows);
    }
    pub fn extend_right(&mut self, rows: usize) {
        force_cursors!(self).extend_right(rows);
    }
    pub fn retract_up(&mut self, rows: usize) {
        force_cursors!(self).retract_up(rows);
    }
    pub fn retract_down(&mut self, rows: usize) {
        force_cursors!(self).retract_down(rows);
    }
    pub fn retract_left(&mut self, rows: usize) {
        force_cursors!(self).retract_left(rows);
    }
    pub fn retract_right(&mut self, rows: usize) {
        force_cursors!(self).retract_right(rows);
    }

    pub fn drop_other_selections(&mut self) {
        force_cursors!(self).drop_others();
    }
}
