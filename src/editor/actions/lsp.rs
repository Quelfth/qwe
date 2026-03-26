use crate::{
    editor::{Editor, markdown_view::MarkdownGadget},
    lsp::channel::{EditorToLspMessage, GotoKind},
};

impl Editor {
    pub fn hover(&mut self) {
        if let Some(cx) = &self.lsp
            && let Some(lang) = self.doc().language()
            && let Some(path) = self.filepath.clone()
            && let Some(pos) = self.doc().main_cursor_pos_utf16()
        {
            _ = cx.tx.send(EditorToLspMessage::Hover { lang, path, pos });
            self.open_gadget(MarkdownGadget::empty());
        }
    }

    pub fn complete(&mut self) {
        if let Some(cx) = &self.lsp
            && let Some(lang) = self.doc().language()
            && let Some(path) = self.filepath.clone()
            && let Some(pos) = self.doc().main_cursor_pos_utf16()
        {
            _ = cx.tx.send(EditorToLspMessage::Completion { lang, path, pos });
        }
    }

    pub fn goto(&mut self, kind: GotoKind) {
        if let Some(cx) = &self.lsp
            && let Some(lang) = self.doc().language()
            && let Some(path) = self.filepath.clone()
            && let Some(pos) = self.doc().main_cursor_pos_utf16()
        {
            _ = cx.tx.send(EditorToLspMessage::Goto {
                lang,
                path,
                pos,
                kind,
            });
        }
    }

    pub fn code_actions(&mut self) {
        if let Some(cx) = &self.lsp
            && let Some(lang) = self.doc().language()
            && let Some(path) = self.filepath.clone()
            && let Some(pos) = self.doc().main_cursor_pos_utf16()
        {
            _ = cx.tx.send(EditorToLspMessage::CodeActions { lang, path, pos });
        }
    }
}
