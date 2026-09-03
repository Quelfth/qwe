use std::{collections::HashMap, fs, io::stdout, path::Path, sync::Arc};

use crossterm::{ExecutableCommand, clipboard::CopyToClipboard};

use crate::{
    aprintln::aprintln, editor::{
        Editor, cursors::{CursorSet, Cursors, line_select::LineCursor, select::{SelectCursor, SelectCursors}}, finder::Finder, inspect::Inspector, jump_labels::JumpLabels, log::LogViewer, picker::Picker
    }, ix::{Ix, ix}, lsp::channel::EditorToLspMessage, pos::Pos, terminal_size::terminal_size, timeline::TimeDirection
};

mod insert;
mod line_select;
mod select;
mod lsp;

impl Editor {
    pub fn scroll_up(&mut self, lines: usize) {
        self.doc.scroll = self.doc.scroll.saturating_sub(Ix::new(lines));
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.doc.scroll += Ix::new(lines);
    }

    pub fn scroll_left(&mut self, columns: usize) {
        self.doc.horizontal_scroll = self.doc.horizontal_scroll.saturating_sub(Ix::new(columns));
    }

    pub fn scroll_right(&mut self, columns: usize) {
        self.doc.horizontal_scroll += Ix::new(columns);
    }

    pub fn scroll_to_main_cursor(&mut self) {
        self.doc.scroll_to_main_cursor();
    }

    pub fn save_file(&mut self) {
        for path in self.bg_docs.take_save_list() {
            self.save_document(path);
        }

        if let Some(path) = self.filepath.clone() {
            self.save_document(path);
        }
    }

    pub fn save_document(&mut self, path: Arc<Path>) {
        let doc = if let Some(fp) = self.filepath.clone() && let Ok(eq) = try {path.canonicalize()? == fp.canonicalize()?} && eq {
            self.doc.declare_saved();
            &self.doc
        } else {
            let Some(doc) = self.bg_docs.by_path_mut(&path) else { return };
            doc.declare_saved();
            self.bg_docs.by_path(&path).unwrap()
        };

        _= fs::write(&path, format!("{}{}", if doc.byte_order_mark {"\u{feff}"} else {""}, doc.text()).as_bytes());
        if let Some(cx) = &self.cmn.lsp
            && let Some(lang) = doc.language()
        {
            _= cx.tx.send(EditorToLspMessage::Save { lang, path });
        }
    }

    fn copy_to_system_clipboard(&mut self, text: &str) {
        _= stdout().execute(CopyToClipboard::to_clipboard_from(text));
    }

    pub fn copy_file(&mut self) {
        self.copy_to_system_clipboard(&self.doc.text().to_string())
    }

    pub fn system_copy(&mut self) {
        try {
            self.copy_to_system_clipboard(&self.doc.copy_main_text()?)
        };
    }

    pub fn view_log(&mut self) {
        self.open_gadget(LogViewer::new());
    }

    pub fn inspect(&mut self) {
        let Some(tree) = &self.doc.tree() else { return };
        let (start, end) = self.doc.inspect_range();
        let [Ok(start), Ok(end)] = [start, end].map(|p| self.doc.text().byte_pos_of_pos(p)) else {
            return;
        };
        self.open_gadget(Inspector::new(self.doc.semtoks.ranges(), tree, start..end))
    }

    fn unredo(&mut self, dir: TimeDirection) {
        let Err(cp) = self.doc.unredo(dir) else {return};
        let docs = self.cmn.global_timeline[dir].pop(cp);

        let cp = self.cmn.global_timeline[dir.rev()].checkpoint();

        let mut doc_counts = HashMap::<_, u32>::new();

        for doc in docs {
            *doc_counts.entry(doc).or_default() += 1;
        }

        for (doc, count) in doc_counts {
            let Ok(doc) = doc.canonicalize() else {continue};
            self.cmn.global_timeline[dir.rev()].push_doc_change(doc.to_owned().into());
            if self.filepath.as_ref().and_then(|f| f.canonicalize().ok()).is_some_and(|f| f == doc) {
                self.doc.global_unredo(dir, cp, count);
            }

            if let Some(doc) = self.bg_docs.by_path_mut(&doc) {
                doc.global_unredo(dir, cp, count);
            }
        }
    }

    pub fn undo(&mut self) {
        self.unredo(TimeDirection::History)
    }

    pub fn redo(&mut self) {
        self.unredo(TimeDirection::Future)
    }

    #[allow(unused)]
    pub fn debug_undo(&mut self) {
        aprintln!("{:#?}", self.doc.timeline);
    }

    pub fn jump(&mut self) {
        let (_, height) = terminal_size();
        self.open_gadget(JumpLabels::generate(&self.doc, Ix::new(height as usize)))
    }

    pub fn find(&mut self) {
        self.open_gadget(Finder::new(self.doc.find_haystacks()));
    }

    pub fn find_all(&mut self) {
        self.open_gadget(Finder::new(vec![self.doc.full_haystack()]))
    }

    pub fn find_in(&mut self) {
        let haystacks = self.doc.cursor_haystacks();
        if haystacks.is_empty() { return }
        self.open_gadget(Finder::new(haystacks));
    }

    pub fn pick_file(&mut self) {
        self.open_gadget(Picker::file());
    }

    pub fn delete(&mut self) {
        self.doc.do_delete();
    }

    pub fn cut(&mut self) {
        self.copy();
        self.delete();
    }

    pub fn copy(&mut self) {
        self.cmn.clipboard.new_clip();
        for text in self.doc.copy_text() {
            self.cmn.clipboard.append(text);
        }
    }

    pub fn paste(&mut self) {
        self.doc.timeline.history.checkpoint();
        if let Some(cursors) = &self.doc.cursors {
            for cursor in cursors.indices() {
                let text = self.cmn.clipboard.next_clip_elt();
                self.doc.paste_at_cursor(text.to_owned(), cursor);
            }
        }
    }

    pub fn refresh_lsp(&mut self) {
        if let Some(cx) = &self.cmn.lsp {
            for (path, doc) in self.bg_docs.pathed() {
                if let Some(lang) = doc.language() {
                    _= cx.tx.send(EditorToLspMessage::OpenDoc{
                        lang,
                        path,
                        text: doc.text().to_string(),
                    });
                }
            }
            if let Some(path) = self.filepath.clone()
                && let Some(lang) = self.doc.language() {
                _= cx.tx.send(EditorToLspMessage::OpenDoc{
                    lang,
                    path,
                    text: self.doc.text().to_string(),
                });
            }
            _= cx.tx.send(EditorToLspMessage::RefreshSemanticTokens);
        }
    }

    pub fn cursor_line_split(&mut self) {
        self.doc.cursor_line_split();
    }

    pub fn incremental_select(&mut self) {
        self.doc.incremental_select();
    }

    pub fn cycle_cursors_forward(&mut self) {
        if let Some(c) = &mut self.doc.cursors {
            c.cycle_forward();
            self.doc.scroll_main_cursor_on_screen();
        }
    }

    pub fn cycle_cursors_backward(&mut self) {
        if let Some(c) = &mut self.doc.cursors {
            c.cycle_backward();
            self.doc.scroll_main_cursor_on_screen();
        }
    }

    pub fn tab_lines_in(&mut self) {
        self.doc.tab_lines_in();
    }

    pub fn tab_lines_out(&mut self) {
        self.doc.tab_lines_out();
    }

    pub fn previous_file(&mut self) {
        if let Some(file) = self.cmn.file_history.pop() {
            _= self.reopen_file_doc(file);
        }
    }
    
    pub fn next_file(&mut self) {
        if let Some(file) = self.cmn.file_future.pop() {
            _= self.open_file_doc(file);
        }
    }

    pub fn mouse_select_new(&mut self) {
        let (row, col) = self.mouse_pos;
        let row = ix(row as _) + self.doc.scroll;
        let col = ix((col.saturating_sub(self.doc.gutter_width())) as _) + self.doc.horizontal_scroll;
        self.doc.cursors = Some(SelectCursors::one(SelectCursor::one_pos(Pos { line: row, column: col })).into());
    }

    pub fn mouse_select_continue(&mut self) {
        let Some(start) = self.doc.main_cursor_pos() else {
            self.mouse_select_new();
            return;
        };
        let (row, col) = self.mouse_pos;
        let row = ix(row as _) + self.doc.scroll;
        let col = ix((col.saturating_sub(self.doc.gutter_width())) as _) + self.doc.horizontal_scroll;
        let end = Pos { line: row, column: col };
        let (start, end) = if start > end { (end, start) } else { (start, end) };
        self.doc.cursors = Some(SelectCursors::one(SelectCursor::range(start..end, self.doc.text())).into());
    }

    pub fn mouse_line_select_new(&mut self) {
        let (row, _) = self.mouse_pos;
        let row = ix(row as _) + self.doc.scroll;
        self.doc.cursors = Some(CursorSet::one(LineCursor { line: row, height: ix(0) }).into())
    }

    pub fn mouse_line_select_continue(&mut self) {
        let Some(start) = self.doc.main_cursor_pos() else {
            self.mouse_select_new();
            return;
        };
        let (row, _) = self.mouse_pos;
        let row = ix(row as _) + self.doc.scroll;

        self.doc.cursors = Some(CursorSet::one(LineCursor { line: start.line.min(row), height: start.line.abs_diff(row) }).into());
    }
}
