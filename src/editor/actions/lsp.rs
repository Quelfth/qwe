use std::{collections::HashSet, path::Path, sync::Arc};

use crate::{
    aprintln::aprintln,
    editor::{Editor, code_actions::ActionEdit, documents::DocKey},
    lang::Language,
    lsp::channel::{EditorToLspMessage, GotoKind},
    pos::Utf16Pos,
    util::uri_to_canon_path,
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

    pub fn apply_action_edits(&mut self, mut edits: Vec<ActionEdit>) {
        let mut global = false;
        for edit in &edits {
            let ActionEdit::Change { uri, .. } = edit else {return};
            if uri.scheme() != "file" {return};
            let Ok(path) = uri.to_file_path() else {return};
            if global {continue}
            if self
                .filepath
                .as_ref()
                .and_then(|f|
                    (f.canonicalize().ok()? == path.canonicalize().ok()?).then_some(())
                )
                .is_some()
            {
                global = true;
            }
        }

        edits.sort_by_key(|e| {
            let ActionEdit::Change{ range, .. } = e else { panic!() };
            range.start
        });
        self.doc.timeline.history.checkpoint();
        let cp = global.then(|| self.global_timeline.history.checkpoint());
        let mut doc_edited = false;
        let mut bg_docs_edited = HashSet::<DocKey>::new();
        for edit in edits.into_iter().rev() {
            let ActionEdit::Change {
                uri,
                range,
                text,
            } = edit else {continue};
            
            let Some(path) = uri_to_canon_path(uri) else {return};
            aprintln!("found path {path:?}");

            let doc = if self.filepath.as_ref().and_then(|f| (f.canonicalize().ok()? == path).then_some(())).is_some() {
                aprintln!("open path {path:?}");
                let doc = &mut self.doc;
                if !doc_edited {
                    if global {
                        doc.timeline.history.global_checkpoint();
                        self.global_timeline.history.push_doc_change(self.filepath.clone().unwrap());
                    } else {
                        doc.timeline.history.checkpoint();
                    }
                    doc_edited = true;
                }
                doc
            } else {
                aprintln!("background path {path:?}");
                let path: Arc<Path> = path.into();
                let key = if let Some(key) = self.bg_docs.key_from_path(&path) {key} else{
                    let Ok(key) = self.open_bg_doc(path.clone()) else {continue};
                    key.unwrap()
                };
                let doc = self.bg_docs.by_key_mut(key).unwrap();
                if !bg_docs_edited.contains(&key) {
                    doc.timeline.history.global_checkpoint();
                    self.global_timeline.history.push_doc_change(path);
                    bg_docs_edited.insert(key);
                }
                doc
            };

            let Some(start) = doc.text().byte_of_utf16_pos(range.start) else {continue};
            let Some(end) = doc.text().byte_of_utf16_pos(range.end) else {continue};
            let Some(pos) = doc.text().pos_of_byte_pos(start) else {continue};
            doc.delete(start..end);
            doc.direct_insert(pos, &text);
        }
        if let Some(cp) = cp {
            if doc_edited {
                self.doc.timeline.history.push_global_jump(cp);
            }
            for doc in bg_docs_edited {
                self.bg_docs.by_key_mut(doc).unwrap().timeline.history.push_global_jump(cp);
                self.bg_docs.push_save(doc);
            }
        }
    }
}
