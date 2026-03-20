use crate::editor::Editor;

impl Editor {
    pub fn insert_newline_above(&mut self) {
        self.doc.insert_newline_above();
    }

    pub fn insert_newline_below(&mut self) {
        self.doc.insert_newline_below();
    }

    pub fn insert_on_newline_before(&mut self) {
        self.insert_newline_above();
        self.move_y(-1);
        self.insert_before();
    }

    pub fn insert_on_newline_after(&mut self) {
        self.insert_newline_below();
        self.insert_after();
    }
}
