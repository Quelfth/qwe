use crate::{
    document::{Document, force_cursors},
    editor::cursors::{CursorState, Cursors},
    ix::{Column, Ix, Line},
};

impl Document {
    pub fn insert_before(&mut self) {
        self.timeline.history.checkpoint();
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(_) => (),
                Select(cursors) => self.cursors = Some(cursors.to_insert_before().into()),
                LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_before(&self.text).into())
                }
            }
        }
    }
    pub fn insert_after(&mut self) {
        self.timeline.history.checkpoint();
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(_) => (),
                Select(cursors) => self.cursors = Some(cursors.to_insert_after().into()),
                LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_after(&self.text).into())
                }
            }
        }
    }
    pub fn insert_before_line(&mut self) {
        self.timeline.history.checkpoint();
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(cursors) => {
                    self.cursors = Some(cursors.to_insert_before_line(&self.text).into())
                }
                LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_before(&self.text).into())
                }
            }
        }
    }
    pub fn insert_after_line(&mut self) {
        self.timeline.history.checkpoint();
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(_) => (),
                Select(cursors) => {
                    self.cursors = Some(cursors.to_insert_after_line(&self.text).into())
                }
                LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_after(&self.text).into())
                }
            }
        }
    }
    pub fn line_select(&mut self) {
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(c) => self.cursors = Some(c.to_line_select().into()),
                Select(c) => self.cursors = Some(c.to_line_select().into()),
                LineSelect(_) => (),
            }
        }
    }

    pub fn insert_around_in(&mut self) {
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(c) => self.cursors = Some(c.to_mirror_insert_in().into()),
                LineSelect(c) => self.cursors = Some(c.to_insert_around_in(&self.text).into()),
            }
        }
    }
    pub fn insert_around_out(&mut self) {
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(c) => self.cursors = Some(c.to_mirror_insert_out().into()),
                LineSelect(c) => self.cursors = Some(c.to_insert_around_out(&self.text).into()),
            }
        }
    }

    pub fn block_select(&mut self) {
        if let Some(c) = &mut self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(c) => c.block_select(),
                LineSelect(c) => self.cursors = Some(c.to_block_select(&self.text).into()),
            }
        }
    }

    pub fn text_select(&mut self) {
        if let Some(c) = &mut self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(c) => c.text_select(&self.text),
                LineSelect(c) => self.cursors = Some(c.to_select(&self.text).into()),
            }
        }
    }

    pub fn move_x(&mut self, columns: Ix<Column, isize>) {
        if let Some(c) = &mut self.cursors {
            c.move_x(columns)
        }
    }

    pub fn move_y(&mut self, rows: Ix<Line, isize>) {
        force_cursors!(self).move_y(rows);
    }

    pub fn text_extend_up(&mut self, rows: Ix<Line>) {
        force_cursors!(self).text_extend_up(rows, &self.text)
    }
    pub fn text_extend_down(&mut self, rows: Ix<Line>) {
        force_cursors!(self).text_extend_down(rows, &self.text)
    }

    pub fn extend_left(&mut self, columns: Ix<Column>) {
        force_cursors!(self).extend_left(columns);
    }
    pub fn extend_right(&mut self, columns: Ix<Column>) {
        force_cursors!(self).extend_right(columns);
    }
    pub fn retract_up(&mut self, rows: Ix<Line>) {
        force_cursors!(self).retract_up(rows);
    }
    pub fn retract_down(&mut self, rows: Ix<Line>) {
        force_cursors!(self).retract_down(rows);
    }
    pub fn retract_left(&mut self, rows: Ix<Column>) {
        force_cursors!(self).retract_left(rows);
    }
    pub fn retract_right(&mut self, rows: Ix<Column>) {
        force_cursors!(self).retract_right(rows);
    }

    pub fn drop_other_selections(&mut self) {
        force_cursors!(self).drop_others();
    }

    pub fn syntax_extend(&mut self) {
        if let Some(tree) = &self.tree {
            if let Some(c) = &mut self.cursors {
                c.syntax_extend(&self.text, tree)
            }
        }
    }
}
