use std::fs;

use crate::{
    document::Document,
    editor::{Editor, inspect::Inspector},
    lang::Language,
    util::pretty_node,
};

mod insert;
mod line_select;
mod select;

impl Editor {
    pub fn scroll_up(&mut self, lines: usize) {
        self.doc.scroll = self.doc.scroll.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.doc.scroll += lines;
    }

    pub fn save_file(&mut self) {
        if let Some(path) = self.filepath.as_deref() {
            _ = fs::write(path, self.doc.text().to_string().as_bytes());
        }
    }

    pub fn inspect(&mut self) {
        let Some(tree) = &self.doc.tree() else { return };
        let (start, end) = self.cursors.inspect_range();
        let [Ok(start), Ok(end)] = [start, end].map(|p| self.doc.byte_pos_of_pos(p)) else {
            return;
        };
        self.inspector = Some(Inspector::new(Document::new(
            Some(Language::Query),
            pretty_node(
                tree.root_node()
                    .descendant_for_byte_range(start, end)
                    .unwrap(),
            ),
        )))
    }

    pub fn exit_inspect(&mut self) {
        self.inspector = None;
    }
}
