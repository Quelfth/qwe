use std::{io, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mutx::Mutex;

use crate::{
    PathedFile,
    document::{Change, CursorChange, Document},
    draw::screen::Screen,
    editor::{
        cursors::{CursorState, insert::InsertCursors},
        keymap::Keymaps,
    },
    lang::Language,
    pos::Pos,
};

mod actions;
pub mod cursors;
mod keymap;

#[derive(Default)]
pub struct Editor {
    filepath: Option<PathBuf>,
    doc: Document,
    cursors: CursorState,
    pub screen: Mutex<Screen>,
    keymap: Keymaps,
}

impl Editor {
    pub fn new(file: Option<PathedFile>) -> Self {
        match file {
            Some(PathedFile { path, file }) => Self {
                doc: {
                    let doc = Document::new(
                        path.extension()
                            .and_then(|e| Language::from_file_ext(&e.to_string_lossy())),
                        file,
                    );
                    // doc.print_tree();
                    doc
                },
                filepath: Some(path),
                ..Self::default()
            },
            None => Self::default(),
        }
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn cursors(&self) -> &CursorState {
        &self.cursors
    }

    pub fn on_key_event(&mut self, event: KeyEvent) -> io::Result<()> {
        match &self.cursors {
            CursorState::Insert(_) => {
                if let Some(action) = self.keymap.insert.map_event(event) {
                    action(self);
                    self.draw()?;
                } else if let KeyEvent {
                    code: KeyCode::Char(char),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press | KeyEventKind::Repeat,
                    ..
                } = event
                {
                    self.insert(&String::from(char));
                    self.draw()?;
                }
            }
            CursorState::Select(_) => {
                if let Some(action) = self.keymap.select.map_event(event) {
                    action(self);
                    self.draw()?;
                }
            }
            CursorState::LineSelect(_) => {
                if let Some(action) = self.keymap.line_select.map_event(event) {
                    action(self);
                    self.draw()?;
                }
            }
        }

        Ok(())
    }

    fn change_insert(
        &mut self,
        cursors: &InsertCursors,
        change: impl Fn(&Document, Pos) -> (Option<Change>, Option<CursorChange>),
    ) {
        let mut changes = Vec::<CursorChange>::new();
        for cursor in cursors.iter() {
            let pos = changes.iter().fold(cursor.pos, |p, c| c.apply(p));
            let (change, cursor_change) = change(&self.doc, pos);
            if let Some(change) = cursor_change {
                changes.push(change);
            }
            if let Some(change) = change {
                self.doc.change(change.clone());
            }
        }
        for change in changes {
            self.cursors.apply_change(change);
        }
    }
}
