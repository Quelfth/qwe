use std::{io, ops::Range, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mutx::Mutex;

use crate::{
    PathedFile,
    document::{Change, CursorChange, Document},
    draw::screen::Screen,
    editor::{
        clipboard::Clip,
        cursors::{
            CursorState,
            insert::InsertCursors,
            select::{SelectCursor, SelectCursors},
        },
        finder::Finder,
        gadget::Gadget,
        inspect::Inspector,
        jump_labels::{CheckFail, JumpLabels},
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
    cursors: CursorState,
    pub screen: Mutex<Screen>,
    keymap: Keymaps,
    pub gadget: Option<Box<dyn Gadget>>,
    pub clipboard: Option<Clip>,
}

impl Editor {
    pub fn new(file: Option<PathedFile>) -> Self {
        match file {
            Some(PathedFile { path, file }) => Self {
                doc: Document::new(
                    path.extension()
                        .and_then(|e| Language::from_file_ext(&e.to_string_lossy())),
                    file,
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

    pub fn cursors(&self) -> &CursorState {
        &self.cursors
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

    fn mark_undo_checkpoint(&mut self) {
        self.doc.history.checkpoint();
    }

    fn do_insert(
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
                let reverse = self.doc.change(change.clone());
                self.doc.history.push(reverse);
            }
        }
        for change in changes {
            self.cursors.apply_change(change);
        }
    }

    fn jump_to(&mut self, dest: Pos) {
        self.cursors = CursorState::Select(SelectCursors::one(SelectCursor::one_pos(dest)))
    }

    fn select_ranges(&mut self, ranges: impl IntoIterator<Item = Range<usize>>) -> Result<(), ()> {
        if let Some(cursors) = SelectCursors::from_iter(
            ranges
                .into_iter()
                .map(|r| SelectCursor::range(r, self.doc())),
        ) {
            self.cursors = CursorState::Select(cursors);
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
