use std::{collections::HashSet, convert::identity, ops::Range};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use lsp_types::{
    CodeAction as LspCodeAction, Command, WorkspaceEdit, Url,
    TextEdit, DocumentChanges, TextDocumentEdit, OptionalVersionedTextDocumentIdentifier,
    AnnotatedTextEdit, OneOf, ResourceOp, CreateFile, RenameFile, DeleteFile,
};

use crate::{
    color, draw::screen::Canvas, editor::{Editor, documents::DocKey, gadget::Gadget}, grapheme::GraphemeExt, pos::Utf16Pos, style::Style
};

#[expect(unused)]
pub enum ActionEdit {
    Change {
        uri: Url,
        range: Range<Utf16Pos>,
        text: String,
    },
    Create {
        uri: Url,
    },
    Delete {
        uri: Url,
    },
    Move {
        uri: Url,
        new_uri: Url,
    },
}

impl ActionEdit {
    pub fn from_text_edit(uri: Url, edit: TextEdit) -> Self {
        let TextEdit { range: lsp_types::Range { start, end }, new_text } = edit;
        Self::Change {
            uri,
            range: Utf16Pos::from_lsp_pos(start)..Utf16Pos::from_lsp_pos(end),
            text: new_text,
        }
    }

    pub fn from_text_document_edit(edit: TextDocumentEdit) -> impl Iterator<Item = Self> {
        gen {
            let TextDocumentEdit{
                text_document: OptionalVersionedTextDocumentIdentifier { uri, .. },
                edits: changes,
            } = edit; 
            for OneOf::Left(edit) | OneOf::Right(AnnotatedTextEdit { text_edit: edit, .. }) in changes {
                let edit = Self::from_text_edit(uri.clone(), edit);
                yield edit;
            }
        }
    }
}

pub struct CodeAction {
    title: String,
    edits: Vec<ActionEdit>,
    command: Option<Command>,
}

impl CodeAction {
    pub fn from_lsp(action: LspCodeAction) -> Self {
        let LspCodeAction { title, edit, command, .. } = action;
        let mut edits = Vec::new();
        if let Some(WorkspaceEdit {
            changes,
            document_changes,
            ..
        }) = edit {
            if let Some(changes) = changes {
                for (uri, changes) in changes {
                    for edit in changes {
                        edits.push(ActionEdit::from_text_edit(uri.clone(), edit));
                    }
                }
            }
            if let Some(changes) = document_changes {
                use DocumentChanges::*;
                match changes {
                    Edits(changes) => {
                        for edit in changes {
                            edits.extend(ActionEdit::from_text_document_edit(edit));
                        }
                    },
                    Operations(ops) => {
                        for op in ops {
                            use lsp_types::DocumentChangeOperation::*;
                            match op {
                                Op(op) => {
                                    use ResourceOp::*;
                                    match op {
                                        Create(CreateFile{ uri, .. }) => {
                                            edits.push(ActionEdit::Create { uri });
                                        },
                                        Rename(RenameFile { old_uri, new_uri, .. }) => {
                                            edits.push(ActionEdit::Move { uri: old_uri, new_uri });
                                        },
                                        Delete(DeleteFile { uri, .. }) => {
                                            edits.push(ActionEdit::Delete { uri });
                                        },
                                    }
                                },
                                Edit(edit) => {
                                    edits.extend(ActionEdit::from_text_document_edit(edit));
                                },
                            }
                        }
                    },
                }
            }
        }
        Self {
            title,
            edits,
            command,
        }
    }
}

pub struct CodeActionsGadget {
    actions: Vec<CodeAction>,
    selected: usize,
}

impl CodeActionsGadget {
    pub fn new(actions: Vec<CodeAction>) -> Self {
        Self {
            actions,
            selected: 0,
        }
    }
}

impl Gadget for CodeActionsGadget {
    fn on_key(&mut self, event: KeyEvent) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
        macro_rules! xx {
            ($($tokens: tt)*) => {
                Some(Box::new($($tokens)*))
            };
        }
        match event {
            KeyEvent {
                code: KeyCode::Char(_),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => None,

            KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => None,

            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => {
                if self.actions.is_empty() {
                    return None;
                }
                self.selected = (self.selected + 1) % self.actions.len();
                xx!(Editor::noop)
            }
            KeyEvent {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => {
                if self.actions.is_empty() {
                    return None;
                }
                self.selected = self.selected.wrapping_sub(1) % self.actions.len();
                xx!(Editor::noop)
            }

            KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            } => {
                let action = self.actions.remove(self.selected);
                Some(Box::new(move |editor| {
                    let CodeAction { mut edits, command: None, .. } = action else {return};
                    let mut global = false;
                    for edit in &edits {
                        let ActionEdit::Change { uri, .. } = edit else {return};
                        if uri.scheme() != "file" {return};
                        let Ok(path) = uri.to_file_path() else {return};
                        if global {continue}
                        if !editor
                            .filepath
                            .as_ref()
                            .and_then(|f|
                                Some(f.canonicalize().ok()? == path.canonicalize().ok()?)
                            )
                            .is_some_and(identity)
                        {
                            global = true;
                        }
                    }

                    edits.sort_by_key(|e| {
                        let ActionEdit::Change{ range, .. } = e else { panic!() };
                        range.start
                    });
                    editor.doc.timeline.history.checkpoint();
                    let cp = global.then(|| editor.global_timeline.history.checkpoint());
                    let mut doc_edited = false;
                    let mut bg_docs_edited = HashSet::<DocKey>::new();
                    for edit in edits.into_iter().rev() {
                        let ActionEdit::Change {
                            uri,
                            range,
                            text,
                        } = edit else {continue};

                        if uri.scheme() != "file" {return};
                        let Ok(path) = uri.to_file_path() else {return};
                        let Ok(path) = path.canonicalize() else {return};

                        let doc = if editor.filepath.as_ref().and_then(|f| Some(f.canonicalize().ok()? == path.canonicalize().ok()?)).is_some_and(identity) {
                            let doc = &mut editor.doc;
                            if !doc_edited {
                                if global {
                                    doc.timeline.history.global_checkpoint();
                                    editor.global_timeline.history.push_doc_change(editor.filepath.clone().unwrap());
                                } else {
                                    doc.timeline.history.checkpoint();
                                }
                                doc_edited = true;
                            }
                            doc
                        } else {
                            let Some(key) = editor.bg_docs.key_from_path(&path) else {continue};
                            let doc = editor.bg_docs.by_key_mut(key).unwrap();
                            if !bg_docs_edited.contains(&key) {
                                doc.timeline.history.global_checkpoint();
                                editor.global_timeline.history.push_doc_change(path.into());
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
                            editor.doc.timeline.history.push_global_jump(cp);
                        }
                        for doc in bg_docs_edited {
                            editor.bg_docs.by_key_mut(doc).unwrap().timeline.history.push_global_jump(cp);
                        }
                    }
                    editor.close_gadget();
                }))
            }

            _ => None,
        }
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        let style = (Style::fg(color::FG) + Style::bg(color::BG)).into();

        for (i, item) in (0..canvas.height()).zip(&self.actions) {
            let style = if i == self.selected as u16 {
                (Style::fg(color::FG) + Style::bg(color::LIT_BG)).into()
            } else {
                style
            };
            for (j, g) in (0..canvas.width()).zip(item.title.graphemes()) {
                let cell = &mut canvas[(i, j)];
                cell.grapheme = g;
                cell.style = style;
            }
        }
    }
}
