use crate::editor::Editor;

impl Editor {
    pub fn insert_on_newline_before(&mut self) {
        self.insert_before();
        self.insert_return();
        self.move_y(-1);
    }

    pub fn insert_on_newline_after(&mut self) {
        self.insert_after();
        self.insert_return();
    }
}
