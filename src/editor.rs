use std::{io, ops::Range, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mutx::Mutex;

use crate::{
    PathedFile,
    document::Document,
    draw::screen::Screen,
    editor::{
        clipboard::{Clip, Clipboard},
        cursors::{
            CursorState,
            select::{SelectCursor, SelectCursors},
        },
        gadget::Gadget,
        keymap::Keymaps,
    },
    lang::Language,
    pos::Pos,
};

mod actions;
mod clipboard;
pub mod cursors;
mod finder;
pub mod gadget;
mod inspect;
pub mod jump_labels;
mod keymap;

#[derive(Default)]
pub struct Editor {
    filepath: Option<PathBuf>,
    doc: Document,
    pub screen: Mutex<Screen>,
    keymap: Keymaps,
    pub gadget: Option<Box<dyn Gadget>>,
    pub clipboard: Clipboard,
}

impl Editor {
    pub fn new(file: Option<PathedFile>) -> Self {
        match file {
            Some(PathedFile { path, file }) => Self {
                doc: Document::new(
                    path.extension()
                        .and_then(|e| Language::from_file_ext(&e.to_string_lossy())),
                    file,
                    Some(Default::default()),
                ),
                filepath: Some(path),
                ..Self::default()
            },
            None => Self::default(),
        }
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn on_key_event(&mut self, event: KeyEvent) -> io::Result<()> {
        if let Some(gadget) = &mut self.gadget {
            match event {
                KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    self.gadget = None;
                    self.draw()?;
                }
                event => {
                    if let Some(effect) = gadget.on_key(event) {
                        effect(self);
                        self.draw()?;
                    }
                }
            }
            return Ok(());
        }

        if let Some(cursors) = &self.doc.cursors {
            match cursors {
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
        }

        Ok(())
    }

    fn jump_to(&mut self, dest: Pos) {
        self.doc.cursors = Some(CursorState::Select(SelectCursors::one(
            SelectCursor::one_pos(dest),
        )))
    }

    fn select_ranges(&mut self, ranges: impl IntoIterator<Item = Range<usize>>) -> Result<(), ()> {
        if let Some(cursors) = SelectCursors::from_iter(
            ranges
                .into_iter()
                .map(|r| SelectCursor::range(r, self.doc())),
        ) {
            self.doc.cursors = Some(CursorState::Select(cursors));
            Ok(())
        } else {
            Err(())
        }
    }

    fn open_gadget(&mut self, gadget: impl Gadget + 'static) {
        self.gadget = Some(Box::new(gadget))
    }

    fn close_gadget(&mut self) {
        self.gadget = None
    }

    fn noop(&mut self) {}
}
