use std::{path::Path, sync::Arc};

use crate::{
    editor::Editor, lang::Language, lsp::channel::{EditorToLspMessage, GotoKind}, pos::Utf16Pos
};

impl Editor {
    pub fn hover(&mut self) {
        self.send_positional_lsp_message(|lang, path, pos|
            EditorToLspMessage::Hover { lang, path, pos }
        );
    }

    pub fn complete(&mut self) {
        self.send_positional_lsp_message(|lang, path, pos|
            EditorToLspMessage::Completion { lang, path, pos }
        );
    }

    pub fn goto(&mut self, kind: GotoKind) {
        self.send_positional_lsp_message(|lang, path, pos|
            EditorToLspMessage::Goto {
                lang,
                path,
                pos,
                kind,
            }
        );
    }

    pub fn code_actions(&mut self) {
        self.send_positional_lsp_message(|lang, path, pos| EditorToLspMessage::CodeActions { lang, path, pos });
    }

    pub fn rename(&mut self) {
        self.send_positional_lsp_message(|lang, path, pos| EditorToLspMessage::Rename { lang, path, pos });
    }

    pub fn complete_rename(&mut self, name: String) {
        self.send_positional_lsp_message(|lang, path, pos| EditorToLspMessage::CompleteRename { lang, path, pos, name });
    }

    pub fn send_positional_lsp_message(&mut self, message: impl FnOnce(Language, Arc<Path>, Utf16Pos) -> EditorToLspMessage) {
        if let Some(cx) = &self.lsp
            && let Some(lang) = self.doc().language()
            && let Some(path) = self.filepath.clone()
            && let Some(pos) = self.doc().main_cursor_pos_utf16()
        {
            _ = cx.tx.send(message(lang, path, pos));
        }
    }
}
