use std::{io, ops::Range, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mutx::Mutex;

use crate::{
    PathedFile,
    document::{Change, CursorChange, Document},
    draw::screen::Screen,
    editor::{
        cursors::{
            CursorState,
            insert::InsertCursors,
            select::{SelectCursor, SelectCursors},
        },
        finder::Finder,
        inspect::Inspector,
        jump_labels::{CheckFail, JumpLabels},
        keymap::Keymaps,
    },
    lang::Language,
    pos::Pos,
};

mod actions;
pub mod cursors;
mod finder;
mod gadget;
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
    pub inspector: Option<Inspector>,
    pub jump_labels: Option<JumpLabels>,
    pub finder: Option<Finder>,
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
        if self.inspector.is_some() {
            if let KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            } = event
            {
                self.exit_inspect();
                self.draw()?;
            }

            return Ok(());
        }

        if let Some(finder) = &mut self.finder {
            match event {
                KeyEvent {
                    code: KeyCode::Char(char),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press | KeyEventKind::Repeat,
                    ..
                } => {
                    finder.r#type(char);
                    self.draw()?;
                }

                KeyEvent {
                    code: KeyCode::Backspace,
                    kind: KeyEventKind::Press | KeyEventKind::Repeat,
                    ..
                } => {
                    finder.backspace();
                    self.draw()?;
                }

                KeyEvent {
                    code: KeyCode::Enter,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    if let Some(find) = finder.find() {
                        self.finder = None;
                        _ = self.select_ranges(find);
                    }
                    self.draw()?;
                }

                KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    self.finder = None;
                    self.draw()?;
                }
                _ => (),
            }
            return Ok(());
        }

        if let Some(labels) = &mut self.jump_labels {
            match event {
                KeyEvent {
                    code: KeyCode::Char(char),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press | KeyEventKind::Repeat,
                    ..
                } => {
                    if char == ' ' {
                        self.jump_labels = None;
                        self.draw()?;
                        return Ok(());
                    }
                    labels.r#type(char);

                    match labels.check() {
                        Ok(jump) => {
                            self.jump_to(jump);
                            self.jump_labels = None;
                        }
                        Err(CheckFail::NotYet) => (),
                        Err(CheckFail::TooLong) => self.jump_labels = None,
                    }
                    self.draw()?;
                }

                KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    self.jump_labels = None;
                    self.draw()?;
                }
                _ => (),
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
}
