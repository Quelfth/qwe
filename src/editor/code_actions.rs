use std::ops::Range;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use lsp_types::{
    CodeAction as LspCodeAction, Command, WorkspaceEdit, Url,
    TextEdit, DocumentChanges, TextDocumentEdit, OptionalVersionedTextDocumentIdentifier,
    AnnotatedTextEdit, OneOf, ResourceOp, CreateFile, RenameFile, DeleteFile,
};

use crate::{
    color,
    draw::screen::Canvas,
    editor::{Editor, gadget::Gadget},
    grapheme::GraphemeExt,
    style::Style,
    pos::{
        Utf16Pos,
    },
};

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
                todo!()
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
