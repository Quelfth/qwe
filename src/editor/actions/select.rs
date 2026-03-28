use crate::{editor::{Editor, cursors::Cursors}, ix::Ix};

impl Editor {
    pub fn insert_before(&mut self) {
        self.doc.insert_before()
    }
    pub fn insert_after(&mut self) {
        self.doc.insert_after()
    }
    pub fn insert_before_line(&mut self) {
        self.doc.insert_before_line()
    }
    pub fn insert_after_line(&mut self) {
        self.doc.insert_after_line()
    }
    pub fn line_select(&mut self) {
        self.doc.line_select()
    }
    pub fn mirror_insert_in(&mut self) {
        self.doc.insert_around_in();
    }
    pub fn mirror_insert_out(&mut self) {
        self.doc.insert_around_out()
    }

    pub fn move_x(&mut self, columns: isize) {
        self.doc.move_x(Ix::new(columns));
    }

    pub fn move_y(&mut self, rows: isize) {
        self.doc.move_y(Ix::new(rows));
    }

    pub fn text_extend_up(&mut self, rows: usize) {
        self.doc.text_extend_up(Ix::new(rows));
    }
    pub fn text_extend_down(&mut self, rows: usize) {
        self.doc.text_extend_down(Ix::new(rows));
    }

    pub fn extend_left(&mut self, columns: usize) {
        self.doc.extend_left(Ix::new(columns));
    }
    pub fn extend_right(&mut self, columns: usize) {
        self.doc.extend_right(Ix::new(columns));
    }
    pub fn retract_up(&mut self, rows: usize) {
        self.doc.retract_up(Ix::new(rows));
    }
    pub fn retract_down(&mut self, rows: usize) {
        self.doc.retract_down(Ix::new(rows));
    }
    pub fn retract_left(&mut self, columns: usize) {
        self.doc.retract_left(Ix::new(columns));
    }
    pub fn retract_right(&mut self, columns: usize) {
        self.doc.retract_right(Ix::new(columns))
    }

    pub fn drop_other_selections(&mut self) {
        self.doc.drop_other_selections()
    }

    pub fn collapse_cursors_to_start(&mut self) {
        if let Some(cursors) = &mut self.doc.cursors {
            cursors.collapse_to_start();
        }
    }
    pub fn collapse_cursors_to_end(&mut self) {
        if let Some(cursors) = &mut self.doc.cursors {
            cursors.collapse_to_end();
        }
    }

    pub fn syntax_extend(&mut self) {
        self.doc.syntax_extend()
    }
}
