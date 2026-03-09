use std::{fs, iter};

use crate::{
    aprintln::aprintln,
    document::Document,
    editor::{
        Editor, cursors::Cursors, finder::Finder, inspect::Inspector, jump_labels::JumpLabels,
        picker::Picker,
    },
    ix::Ix,
    lang::Language,
    lsp::channel::EditorToLspMessage,
    terminal_size::terminal_size,
    util::{RangeOverlap, pretty_node},
};

mod insert;
mod line_select;
mod select;

impl Editor {
    pub fn scroll_up(&mut self, lines: usize) {
        self.doc.scroll = self.doc.scroll.saturating_sub(Ix::new(lines));
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.doc.scroll += Ix::new(lines);
    }

    pub fn scroll_to_main_cursor(&mut self) {
        self.doc.scroll_to_main_cursor();
    }

    pub fn save_file(&mut self) {
        if let Some(path) = self.filepath.as_deref() {
            _ = fs::write(path, self.doc.text().to_string().as_bytes());
            crate::aprintln::aprintln!("saved");
            if let Some(channel) = &self.lsp_send
                && let Some(lang) = self.doc.language()
                && let Some(path) = self.filepath.clone()
            {
                _ = channel.send(EditorToLspMessage::Save { lang, path });
            }
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

    pub fn undo(&mut self) {
        self.doc.undo()
    }

    pub fn redo(&mut self) {
        self.doc.redo()
    }

    pub fn debug_undo(&mut self) {
        aprintln!("{:#?}", self.doc.history);
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
        self.doc.history.checkpoint();
        if let Some(cursors) = &self.doc.cursors {
            for cursor in cursors.indices() {
                let text = self.clipboard.next_clip_elt();
                self.doc.paste_at_cursor(text.to_owned(), cursor);
            }
        }
    }

    pub fn refresh_semantic_tokens(&mut self) {
        if let Some(send) = &self.lsp_send {
            send.send(EditorToLspMessage::RefreshSemanticTokens)
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
}
