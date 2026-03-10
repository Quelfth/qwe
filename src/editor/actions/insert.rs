use crate::editor::Editor;

impl Editor {
    pub fn select(&mut self) {
        self.doc.select();
    }

    pub fn backspace(&mut self) {
        self.doc.backspace();
    }

    pub fn insert(&mut self, str: &str) {
        self.doc.insert(str);
    }

    pub fn insert_return(&mut self) {
        self.doc.insert_return();
    }

    // pub fn insert_tab(&mut self) {
    //     _ = self.doc.insert_tab();
    // }

    pub fn insert_tab_else_complete(&mut self) {
        if self.doc.insert_tab().is_err() {
            self.complete();
        }
    }

    pub fn tab_out(&mut self) {
        self.doc.tab_out()
    }
}
