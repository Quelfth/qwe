use crate::editor::{Editor, cursors::CursorState};

impl Editor {
    pub fn insert_before(&mut self) {
        match &self.cursors {
            CursorState::Insert(_) => (),
            CursorState::Select(cursors) => self.cursors = cursors.to_insert_before().into(),
            CursorState::LineSelect(cursors) => {
                self.cursors = cursors.to_insert_before(&self.doc).into()
            }
        }
    }
    pub fn insert_after(&mut self) {
        match &self.cursors {
            CursorState::Insert(_) => (),
            CursorState::Select(cursors) => self.cursors = cursors.to_insert_after().into(),
            CursorState::LineSelect(cursors) => {
                self.cursors = cursors.to_insert_after(&self.doc).into()
            }
        }
    }
    pub fn insert_before_line(&mut self) {
        match &self.cursors {
            CursorState::Insert(_) => (),
            CursorState::Select(cursors) => {
                self.cursors = cursors.to_insert_before_line(&self.doc).into()
            }
            CursorState::LineSelect(cursors) => {
                self.cursors = cursors.to_insert_before(&self.doc).into()
            }
        }
    }
    pub fn insert_after_line(&mut self) {
        match &self.cursors {
            CursorState::Insert(_) => (),
            CursorState::Select(cursors) => {
                self.cursors = cursors.to_insert_after_line(&self.doc).into()
            }
            CursorState::LineSelect(cursors) => {
                self.cursors = cursors.to_insert_after(&self.doc).into()
            }
        }
    }
    pub fn line_select(&mut self) {
        match &self.cursors {
            CursorState::Insert(c) => self.cursors = c.to_line_select().into(),
            CursorState::Select(c) => self.cursors = c.to_line_select().into(),
            CursorState::LineSelect(_) => (),
        }
    }

    pub fn move_x(&mut self, columns: isize) {
        self.cursors.move_x(columns)
    }

    pub fn move_y(&mut self, rows: isize) {
        self.cursors.move_y(rows)
    }

    pub fn text_extend_up(&mut self, rows: usize) {
        self.cursors.text_extend_up(rows, &self.doc);
    }
    pub fn text_extend_down(&mut self, rows: usize) {
        self.cursors.text_extend_down(rows, &self.doc);
    }

    pub fn block_extend_up(&mut self, rows: usize) {
        self.cursors.block_extend_up(rows);
    }
    pub fn block_extend_down(&mut self, rows: usize) {
        self.cursors.block_extend_down(rows);
    }
    pub fn extend_left(&mut self, rows: usize) {
        self.cursors.extend_left(rows);
    }
    pub fn extend_right(&mut self, rows: usize) {
        self.cursors.extend_right(rows);
    }
    pub fn retract_up(&mut self, rows: usize) {
        self.cursors.retract_up(rows);
    }
    pub fn retract_down(&mut self, rows: usize) {
        self.cursors.retract_down(rows);
    }
    pub fn retract_left(&mut self, rows: usize) {
        self.cursors.retract_left(rows);
    }
    pub fn retract_right(&mut self, rows: usize) {
        self.cursors.retract_right(rows);
    }
}
