use crate::editor::Editor;

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

    pub fn move_x(&mut self, columns: isize) {
        self.doc.move_x(columns);
    }

    pub fn move_y(&mut self, rows: isize) {
        self.doc.move_y(rows);
    }

    pub fn text_extend_up(&mut self, rows: usize) {
        self.doc.text_extend_up(rows);
    }
    pub fn text_extend_down(&mut self, rows: usize) {
        self.doc.text_extend_down(rows);
    }

    pub fn extend_left(&mut self, rows: usize) {
        self.doc.extend_left(rows);
    }
    pub fn extend_right(&mut self, rows: usize) {
        self.doc.extend_right(rows);
    }
    pub fn retract_up(&mut self, rows: usize) {
        self.doc.retract_up(rows);
    }
    pub fn retract_down(&mut self, rows: usize) {
        self.doc.retract_down(rows);
    }
    pub fn retract_left(&mut self, rows: usize) {
        self.doc.retract_left(rows);
    }
    pub fn retract_right(&mut self, rows: usize) {
        self.doc.retract_right(rows)
    }

    pub fn drop_other_selections(&mut self) {
        self.doc.drop_other_selections()
    }
}
