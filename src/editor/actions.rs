use std::fs;

use crate::{
    aprintln::aprintln,
    document::Document,
    editor::{Editor, finder::Finder, inspect::Inspector, jump_labels::JumpLabels},
    lang::Language,
    terminal_size::{self, terminal_size},
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
        self.open_gadget(Inspector::new(Document::new(
            Some(Language::Query),
            pretty_node(
                tree.root_node()
                    .descendant_for_byte_range(start, end)
                    .unwrap(),
            ),
        )))
    }

    pub fn undo(&mut self) {
        for change in self.doc.undo() {
            self.cursors.apply_change(change);
        }
    }

    pub fn redo(&mut self) {
        for change in self.doc.redo() {
            self.cursors.apply_change(change);
        }
    }

    pub fn debug_undo(&mut self) {
        aprintln!("{:#?}", self.doc.history);
    }

    pub fn jump(&mut self) {
        let (_, height) = terminal_size();
        self.open_gadget(JumpLabels::generate(&self.doc, height as usize))
    }

    pub fn find(&mut self) {
        self.open_gadget(Finder::new(self.doc().text().to_string(), 0));
    }

    pub fn delete(&mut self) {
        todo!()
    }

    pub fn cut(&mut self) {
        self.copy();
        self.delete();
    }

    pub fn copy(&mut self) {
        todo!()
    }
}
