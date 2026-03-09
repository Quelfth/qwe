use std::{
    collections::HashMap,
    io,
    ops::Range,
    path::Path,
    sync::{Arc, mpsc::Receiver},
};

use tokio::sync::mpsc::UnboundedSender;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mutx::Mutex;

use crate::{
    PathedFile,
    document::Document,
    draw::screen::Screen,
    editor::{
        clipboard::Clipboard,
        cursors::{
            CursorState,
            select::{SelectCursor, SelectCursors},
        },
        gadget::Gadget,
        keymap::Keymaps,
    },
    ix::{Byte, Ix},
    lang::Language,
    language_server::LanguageServer,
    lsp::channel::{EditorToLspMessage, LspToEditorMessage},
    pos::Pos,
};

mod actions;
mod clipboard;
pub mod cursors;
pub mod finder;
pub mod gadget;
mod inspect;
pub mod jump_labels;
mod keymap;
pub mod picker;
mod poll;

#[derive(Default)]
pub struct Editor {
    filepath: Option<Arc<Path>>,
    doc: Document,
    pub screen: Mutex<Screen>,
    keymap: Keymaps,
    pub gadget: Option<Box<dyn Gadget>>,
    pub clipboard: Clipboard,
    pub lsp_recv: Option<Receiver<LspToEditorMessage>>,
    pub lsp_send: Option<UnboundedSender<EditorToLspMessage>>,
    pub language_servers: HashMap<Language, Vec<LanguageServer>>,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_scratch_doc(&mut self) {
        self.doc = Document::new(None, "", Some(Default::default()));
    }

    pub fn open_new_doc(&mut self, doc: PathedFile) {
        let PathedFile { path, file } = doc;
        self.doc = Document::new(
            path.extension()
                .and_then(|e| Language::from_file_ext(&e.to_string_lossy())),
            file,
            Some(Default::default()),
        );
        self.filepath = Some(path.clone());

        if let Some(lsp_send) = &self.lsp_send
            && let Some(lang) = self.doc.language()
        {
            lsp_send
                .send(EditorToLspMessage::OpenDoc {
                    lang,
                    path,
                    text: self.doc().text().to_string(),
                })
                .unwrap();
        }
    }

    pub fn set_lsp_channels(
        &mut self,
        send: UnboundedSender<EditorToLspMessage>,
        recv: Receiver<LspToEditorMessage>,
    ) {
        self.lsp_recv = Some(recv);
        self.lsp_send = Some(send);
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
            use CursorState::*;
            match cursors {
                MirrorInsert(_) | Insert(_) => {
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
                Select(_) => {
                    if let Some(action) = self.keymap.select.map_event(event) {
                        action(self);
                        self.draw()?;
                    }
                }
                LineSelect(_) => {
                    if let Some(action) = self.keymap.line_select.map_event(event) {
                        action(self);
                        self.draw()?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn jump_to(&mut self, dest: Pos) {
        self.doc.cursors = Some(CursorState::Select(SelectCursors::one(
            SelectCursor::one_pos(dest),
        )))
    }

    pub fn scroll_main_cursor_on_screen(&mut self) {
        self.doc.scroll_main_cursor_on_screen();
    }

    fn select_ranges(
        &mut self,
        ranges: impl IntoIterator<Item = Range<Ix<Byte>>>,
    ) -> Result<(), ()> {
        if let Some(cursors) = SelectCursors::from_iter(
            ranges
                .into_iter()
                .map(|r| SelectCursor::byte_range(r, self.doc().text())),
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
