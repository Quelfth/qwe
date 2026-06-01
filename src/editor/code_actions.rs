use std::range::Range;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use lsp_types::{
    AnnotatedTextEdit, CodeAction as LspCodeAction, Command, CompletionTextEdit, CreateFile, DeleteFile, DocumentChanges, InsertReplaceEdit, OneOf, OptionalVersionedTextDocumentIdentifier, RenameFile, ResourceOp, TextDocumentEdit, TextEdit, Url, WorkspaceEdit
};

use crate::{
    color, draw::screen::Canvas, editor::{Editor, gadget::Gadget}, grapheme::GraphemeExt, pos::Utf16Pos, style::Style
};

#[derive(Clone, Debug)]
pub struct ActionChangeEdit {
    pub range: Range<Utf16Pos>,
    pub text: String,
}

impl ActionChangeEdit {
    pub fn from_text_edit(edit: TextEdit) -> Self {
        let TextEdit { range: lsp_types::Range { start, end }, new_text } = edit;
        Self {
            range: Utf16Pos::from_lsp_pos(start)..Utf16Pos::from_lsp_pos(end),
            text: new_text,
        }
    }

    pub fn from_completion_edit(edit: CompletionTextEdit) -> Self {
        use CompletionTextEdit::*;
        let (Edit(TextEdit { range, new_text }) | InsertAndReplace(InsertReplaceEdit{ new_text, replace: range, .. })) = edit;
        Self::from_text_edit(TextEdit { range, new_text })
    }
}

#[derive(Clone, Debug)]
pub struct ActionEdit {
    pub uri: Url,
    pub effect: ActionEditEffect,
}

#[derive(Clone, Debug)]
pub enum ActionEditEffect {
    Change(ActionChangeEdit),
    Create,
    Delete,
    Move(#[allow(unused)] Url),
}

impl ActionEdit {
    pub fn from_text_edit(uri: Url, edit: TextEdit) -> Self {
        Self {
            uri,
            effect: ActionEditEffect::Change (ActionChangeEdit::from_text_edit(edit)),
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

    pub fn from_workspace_edit(edit: WorkspaceEdit) -> Vec<Self> {
        let mut edits = Vec::new();
        let WorkspaceEdit {
            changes,
            document_changes,
            ..
        } = edit; 
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
                                        edits.push(ActionEdit { uri, effect: ActionEditEffect::Create });
                                    },
                                    Rename(RenameFile { old_uri, new_uri, .. }) => {
                                        edits.push(ActionEdit { uri: old_uri, effect: ActionEditEffect::Move(new_uri) });
                                    },
                                    Delete(DeleteFile { uri, .. }) => {
                                        edits.push(ActionEdit { uri, effect: ActionEditEffect::Delete });
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
        edits
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
        let edits = edit
            .map(ActionEdit::from_workspace_edit)
            .unwrap_or_default();
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
                    let CodeAction { edits, command: None, .. } = action else {return};
                    editor.apply_action_edits(edits);
                    editor.close_gadget();
                }))
            }

            _ => None,
        }
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        let style = (Style::fg(color::FG) + Style::bg(color::BG)).into();

        for (i, item) in (0..canvas.height()).into_iter().zip(&self.actions) {
            let style = if i == self.selected as u16 {
                (Style::fg(color::FG) + Style::bg(color::LIT_BG)).into()
            } else {
                style
            };
            for (j, g) in (0..canvas.width()).into_iter().zip(item.title.graphemes()) {
                let cell = &mut canvas[(i, j)];
                cell.grapheme = g;
                cell.style = style;
            }
        }
    }
}
