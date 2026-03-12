use crate::{
    editor::{Editor, markdown_view::MarkdownGadget},
    lsp::channel::{EditorToLspMessage, GotoKind},
};

impl Editor {
    pub fn hover(&mut self) {
        if let Some(tx) = &self.lsp_send
            && let Some(lang) = self.doc().language()
            && let Some(path) = self.filepath.clone()
            && let Some(pos) = self.doc().main_cursor_pos_utf16()
        {
            _ = tx.send(EditorToLspMessage::Hover { lang, path, pos });
            self.open_gadget(MarkdownGadget::empty());
        }
    }

    pub fn complete(&mut self) {
        if let Some(tx) = &self.lsp_send
            && let Some(lang) = self.doc().language()
            && let Some(path) = self.filepath.clone()
            && let Some(pos) = self.doc().main_cursor_pos_utf16()
        {
            _ = tx.send(EditorToLspMessage::Completion { lang, path, pos });
        }
    }

    pub fn goto(&mut self, kind: GotoKind) {
        if let Some(tx) = &self.lsp_send
            && let Some(lang) = self.doc().language()
            && let Some(path) = self.filepath.clone()
            && let Some(pos) = self.doc().main_cursor_pos_utf16()
        {
            _ = tx.send(EditorToLspMessage::Goto {
                lang,
                path,
                pos,
                kind,
            });
        }
    }
}
