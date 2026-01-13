use crate::editor::Editor;

mod insert;
mod select;

impl Editor {
    pub fn scroll_up(&mut self, lines: usize) {
        self.doc.scroll = self.doc.scroll.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.doc.scroll += lines;
    }
}
