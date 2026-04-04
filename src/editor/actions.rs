use std::{collections::HashMap, fs, iter, path::Path, sync::Arc};

use crate::{
    aprintln::aprintln, document::Document, editor::{
        Editor, cursors::Cursors, finder::Finder, inspect::Inspector, jump_labels::JumpLabels,
        picker::Picker,
    }, ix::Ix, lang::Language, language_server::LspContext, lsp::channel::EditorToLspMessage, terminal_size::terminal_size, timeline::TimeDirection, util::{RangeOverlap, pretty_node}
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
        fn save_doc(path: Arc<Path>, doc: &Document, lsp: Option<&LspContext>) {
            _ = fs::write(&path, doc.text().to_string().as_bytes());
            if let Some(cx) = &lsp
                && let Some(lang) = doc.language()
            {
                _ = cx.tx.send(EditorToLspMessage::Save { lang, path });
            }
        }

        if let Some(path) = self.filepath.clone() {
            save_doc(path, &self.doc, self.lsp.as_ref());
        }

        for doc in self.bg_docs.take_save_list() {
            let path = self.bg_docs.path_from_key(doc).unwrap();
            let doc = self.bg_docs.by_key(doc).unwrap();
            save_doc(path, doc, self.lsp.as_ref());
        }
    }

    pub fn inspect(&mut self) {
        let Some(tree) = &self.doc.tree() else { return };
        let (start, end) = self.doc.inspect_range();
        let [Ok(start), Ok(end)] = [start, end].map(|p| self.doc.text().byte_pos_of_pos(p)) else {
            return;
        };
        self.open_gadget(Inspector::new(
            Document::new(
                None,
                self.doc
                    .semtoks
                    .ranges()
                    .filter(|(r, _)| r.overlaps(start..end))
                    .map(|(_, s)| {
                        iter::once((*s.r#type).to_owned())
                            .chain(s.mods.iter().map(|m| " ".to_owned() + m))
                            .collect::<String>()
                            + "\n"
                    })
                    .collect::<String>(),
                None,
            ),
            Document::new(
                Some(Language::Query),
                pretty_node(
                    tree.root_node()
                        .descendant_for_byte_range(start.inner(), end.inner())
                        .unwrap(),
                ),
                None,
            ),
        ))
    }

    fn unredo(&mut self, dir: TimeDirection) {
        let Err(cp) = self.doc.unredo(dir) else {return};
        let docs = self.global_timeline[dir].pop(cp);

        let cp = self.global_timeline[dir.rev()].checkpoint();

        let mut doc_counts = HashMap::<_, u32>::new();

        for doc in docs {
            *doc_counts.entry(doc).or_default() += 1;
        }

        for (doc, count) in doc_counts {
            let Ok(doc) = doc.canonicalize() else {continue};
            self.global_timeline[dir.rev()].push_doc_change((&doc).to_owned().into());
            if self.filepath.as_ref().and_then(|f| f.canonicalize().ok()).is_some_and(|f| &f == &doc) {
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
        self.clipboard.new_clip();
        for text in self.doc.copy_text() {
            self.clipboard.append(text);
        }
    }

    pub fn paste(&mut self) {
        self.doc.timeline.history.checkpoint();
        if let Some(cursors) = &self.doc.cursors {
            for cursor in cursors.indices() {
                let text = self.clipboard.next_clip_elt();
                self.doc.paste_at_cursor(text.to_owned(), cursor);
            }
        }
    }

    pub fn refresh_semantic_tokens(&mut self) {
        if let Some(cx) = &self.lsp {
            cx.tx.send(EditorToLspMessage::RefreshSemanticTokens)
                .unwrap();
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
        if let Some(file) = self.file_history.pop() {
            _= self.reopen_file_doc(file);
        }
    }
    
    pub fn next_file(&mut self) {
        if let Some(file) = self.file_future.pop() {
            _= self.open_file_doc(file);
        }
    }
}
