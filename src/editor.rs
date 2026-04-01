use std::{
    io, mem,
    ops::Range,
    path::Path,
    sync::{Arc, mpsc::Receiver},
};

use tokio::sync::mpsc::UnboundedSender;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};

use crate::{
    PathedFile, document::Document, editor::{
        clipboard::Clipboard,
        cursors::{
            CursorState,
            select::{SelectCursor, SelectCursors},
        },
        gadget::Gadget,
        keymap::Keymaps,
    }, ix::{Byte, Ix}, lang::Language, language_server::{LspContext}, lsp::channel::{EditorToLspMessage, LspToEditorMessage}, navigator::Navigator, pos::{Pos, convert::TextConvertablePos}, presenter::{Present, Presenter}
};

use documents::Documents;
use keymap::{InputCode, InputEvent, Key, ScrollDir};

mod actions;
pub mod clipboard;
pub mod code_actions;
pub mod completer;
pub mod cursors;
pub mod documents;
pub mod finder;
pub mod gadget;
mod inspect;
pub mod jump_labels;
pub mod keymap;
pub mod markdown_view;
pub mod picker;
mod poll;

#[derive(Default)]
pub struct Editor {
    filepath: Option<Arc<Path>>,
    doc: Document,
    file_history: Vec<Arc<Path>>,
    file_future: Vec<Arc<Path>>,
    bg_docs: Documents,
    keymap: Keymaps,
    pub gadget: Option<Box<dyn Gadget>>,
    pub clipboard: Clipboard,
    pub lsp: Option<LspContext>,
    pub presenter: Presenter,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_doc(&mut self, new_doc: Document) {
        let old_doc = mem::replace(&mut self.doc, new_doc);
        if let Some(fp) = self.filepath.clone() {
            self.bg_docs.insert_pathed(fp, old_doc);
        }
    }

    pub fn open_scratch_doc(&mut self) {
        self.replace_doc(Document::new(None, "", Some(Default::default())));
    }

    pub fn open_file_doc(&mut self, path: Arc<Path>) -> io::Result<()> {
        if let Some(path) = self.open_file_doc_impl(path)? {
            self.file_history.push(path);
        }
        Ok(())
    }
    pub fn reopen_file_doc(&mut self, path: Arc<Path>) -> io::Result<()> {
        if let Some(path) = self.open_file_doc_impl(path)? {
            self.file_future.push(path);
        }
        Ok(())
    }
    pub fn open_file_doc_at(
        &mut self,
        path: Arc<Path>,
        pos: impl TextConvertablePos<Pos>,
    ) -> io::Result<()> {
        self.open_file_doc(path)?;
        self.jump_to(pos.convert(self.doc.text()));
        self.doc.scroll_main_cursor_on_screen();
        Ok(())
    }

    fn open_file_doc_impl(&mut self, path: Arc<Path>) -> io::Result<Option<Arc<Path>>> {
        let doc = if let Some(doc) = self.bg_docs.extract_by_path(&path) {
            doc
        } else {
            let PathedFile { path, file } = PathedFile::open(path.clone())?;
            Document::new(
                path.extension()
                    .and_then(|e| Language::from_file_ext(&e.to_string_lossy())),
                file,
                Some(Default::default()),
            )
        };
        self.replace_doc(doc);

        let old_path = self.filepath.clone();
        self.filepath = Some(path.clone());

        if let Some(lsp) = &self.lsp
            && let Some(lang) = self.doc.language()
        {
            lsp.tx
                .send(EditorToLspMessage::OpenDoc {
                    lang,
                    path,
                    text: self.doc().text().to_string(),
                })
                .unwrap();
        }

        Ok(old_path)
    }

    pub fn set_lsp_channels(
        &mut self,
        send: UnboundedSender<EditorToLspMessage>,
        recv: Receiver<LspToEditorMessage>,
    ) {
        self.lsp = Some(LspContext::new(recv, send));
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn on_key_event(&mut self, event: InputEvent) -> io::Result<()> {
        if let Some(gadget) = &mut self.gadget {
            match event {
                InputEvent::Event(KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    self.gadget = None;
                    self.draw()?;
                }
                InputEvent::Event(event) => {
                    if let Some(effect) = gadget.on_key(event) {
                        effect(self);
                        self.draw()?;
                    }
                }
                _ => (),
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
                    } else if let InputEvent::Event(KeyEvent {
                        code: KeyCode::Char(char),
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                        kind: KeyEventKind::Press | KeyEventKind::Repeat,
                        ..
                    }) = event
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

    pub fn on_mouse_event(&mut self, event: MouseEvent) -> io::Result<()> {
        let MouseEvent {
            kind, modifiers, ..
        } = event;
        let code = match kind {
            MouseEventKind::Down(button) => InputCode::Mouse(button),
            MouseEventKind::ScrollDown => InputCode::Scroll(ScrollDir::Down),
            MouseEventKind::ScrollUp => InputCode::Scroll(ScrollDir::Up),
            MouseEventKind::ScrollLeft => InputCode::Scroll(ScrollDir::Left),
            MouseEventKind::ScrollRight => InputCode::Scroll(ScrollDir::Right),
            _ => return Ok(()),
        };
        self.on_key_event(InputEvent::Key(
            match (
                modifiers & KeyModifiers::CONTROL != KeyModifiers::NONE,
                modifiers & KeyModifiers::ALT != KeyModifiers::NONE,
            ) {
                (false, false) => Key::base(code),
                (true, false) => Key::ctrl(code),
                (false, true) => Key::alt(code),
                (true, true) => Key::ctrl_alt(code),
            },
        ))?;
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

impl Editor {
    pub fn into_navigator(self) -> Navigator {
        let Self { filepath, doc, mut bg_docs, keymap, clipboard, lsp, presenter, .. } = self;
        if let Some(fp) = filepath.clone() {
            bg_docs.insert_pathed(fp, doc);
        }
        Navigator::new(filepath, bg_docs, keymap, clipboard, lsp, presenter)
    }

    pub fn from_parts(doc: (Option<Arc<Path>>, Document), bg_docs: Documents, keymap: Keymaps, clipboard: Clipboard, lsp: Option<LspContext>, presenter: Presenter) -> Self {
        let (filepath, doc) = doc;
        Self {
            filepath,
            doc,
            file_history: Default::default(),
            file_future: Default::default(),
            bg_docs,
            keymap,
            gadget: None,
            clipboard,
            lsp,
            presenter,
        }
    }
}