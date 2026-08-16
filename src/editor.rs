use std::{
    io, mem,
    range::Range,
    path::Path,
    sync::Arc,
};

use crate::{
    action::Action as _,
    app::{AppSignal, CommonState},
    document::Document,
    editor::{
        cursors::{
            CursorState,
            select::{SelectCursor, SelectCursors},
        },
        documents::DocKey,
        gadget::Gadget,
    },
    global_config::{CharSpecial, GLOBAL_CONFIG},
    ix::{Byte, Ix},
    key::{KeyOrChar, key},
    lang::Language,
    language_server::LspContext,
    lsp::channel::{EditorToLspMessage, EditorToLspSender, LspToEditorReceiver},
    navigator::Navigator,
    pathed_file::PathedFile,
    pos::{Pos, convert::TextConvertablePos},
};

use documents::Documents;

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
pub mod renamer;
pub mod line_jumper;
pub mod log;

#[derive(Default)]
pub struct Editor {
    filepath: Option<Arc<Path>>,
    doc: Document,
    bg_docs: Documents,
    pub gadget: Option<Box<dyn Gadget>>,
    pub cmn: CommonState,
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
        self.open_scratch_doc_with("");
    }

    pub fn open_scratch_doc_with(&mut self, text: impl AsRef<str>) {
        self.replace_doc(Document::new(None, text, Some(Default::default())));
        if let Some(path) = self.filepath.take() {
            self.cmn.file_history.push(path);
        }
    }

    pub fn open_file_doc(&mut self, path: Arc<Path>) -> io::Result<()> {
        if let Some(path) = self.open_file_doc_impl(path)? {
            self.cmn.file_history.push(path);
        }
        Ok(())
    }
    pub fn reopen_file_doc(&mut self, path: Arc<Path>) -> io::Result<()> {
        if let Some(path) = self.open_file_doc_impl(path)? {
            self.cmn.file_future.push(path);
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

    fn open_bg_doc(&mut self, path: Arc<Path>) -> io::Result<Option<DocKey>> {
        if let Some(fp) = self.filepath.as_ref()
            && let Ok(fp) = fp.canonicalize()
            && let Ok(path) = path.canonicalize()
            && fp == path
        {
            return Ok(None);
        }

        if let Some(key) = self.bg_docs.key_from_path(&path) { return Ok(Some(key)) }

        let PathedFile { path, file } = PathedFile::open(path.clone())?;
        let lang = path.extension()
            .and_then(|e| Language::from_file_ext(&e.to_string_lossy()));

        let key = self.bg_docs.insert_pathed(path.clone(), Document::new(
            lang,
            file,
            Some(Default::default()),
        ));

        if let Some(lsp) = &self.cmn.lsp
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

        Ok(Some(key))
    }

    fn open_file_doc_impl(&mut self, path: Arc<Path>) -> io::Result<Option<Arc<Path>>> {
        if let Some(fp) = self.filepath.as_ref()
            && let Ok(fp) = fp.canonicalize()
            && let Ok(path) = path.canonicalize()
            && fp == path
        {
            return Ok(None);
        }

        let doc = if let Some(doc) = self.bg_docs.extract_by_path(&path) {
            doc
        } else {
            let PathedFile { path, file } = PathedFile::open(path.clone())?;
            let lang = path.extension()
                .and_then(|e| Language::from_file_ext(&e.to_string_lossy()));

            if let Some(lsp) = &self.cmn.lsp
                && let Some(lang) = lang
            {
                lsp.tx
                    .send(EditorToLspMessage::OpenDoc {
                        lang,
                        path: path.clone(),
                        text: file.clone(),
                    })
                    .unwrap();
            }

            Document::new(
                lang,
                file,
                Some(Default::default()),
            )
        };
        self.replace_doc(doc);

        let old_path = self.filepath.clone();
        self.filepath = Some(path.clone());


        Ok(old_path)
    }

    pub fn set_lsp_channels(
        &mut self,
        send: EditorToLspSender,
        recv: LspToEditorReceiver,
    ) {
        self.cmn.lsp = Some(LspContext::new(recv, send));
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn doc_mut(&mut self) -> &mut Document {
        &mut self.doc
    }

    pub fn on_key_or_char(&mut self, event: KeyOrChar) -> io::Result<Option<AppSignal>> {
        if let Some(gadget) = &mut self.gadget {
            if event == key![esc].into() {
                self.gadget = None;
                self.upkeep()?;
            } else if let Some(effect) = gadget.on_key(event) {
                effect(self);
                self.upkeep()?;
            }
            return Ok(None);
        }

        let mut signal = None;

        if let Some(cursors) = &self.doc.cursors {
            use CursorState::*;
            match cursors {    
                MirrorInsert(_) | Insert(_) => {
                    let keymap = if matches!(cursors, MirrorInsert(_)) { &GLOBAL_CONFIG.keymaps.mirror_insert } else { &GLOBAL_CONFIG.keymaps.insert };
                    if let Some(key) = event.key()
                        && let Some(action) = keymap.load()[key]
                    {
                        signal = action.act(self);
                        self.upkeep()?;
                    } else if let Some(char) = event.char() {
                        'insert: {
                            if matches!(cursors, Insert(_)) && let Some(special) = GLOBAL_CONFIG.special_chars.lock().get(&char) {
                                match special {
                                    CharSpecial::StrongLeft(right) => {
                                        self.insert_pair(&String::from(char), &String::from(*right));
                                        break 'insert;
                                    },
                                    CharSpecial::Right | CharSpecial::AltInsert | CharSpecial::WeakPair => {
                                        self.insert_reluctant(&String::from(char));
                                        break 'insert;
                                    },
                                    _ => ()
                                }
                            }
                            self.insert(&String::from(char));
                        }
                        self.upkeep()?;
                    } else if let Some(char) = event.alt_char() && let Some(special) = GLOBAL_CONFIG.special_chars.lock().get(&char) {
                        match special {
                            CharSpecial::StrongLeft(right) | CharSpecial::WeakLeft(right) => {
                                self.insert_pair(&String::from(char), &String::from(*right));
                                self.upkeep()?;
                            },
                            CharSpecial::Right => (),
                            CharSpecial::WeakPair => {
                                let string = String::from(char);
                                self.insert_pair(&string, &string);
                                self.upkeep()?;
                            },
                            CharSpecial::AltInsert => {
                                self.insert_pair("", &String::from(char));
                                self.upkeep()?;
                            }
                        }
                    }
                }
                Select(_) => {
                    if let Some(key) = event.key() && let Some(action) = GLOBAL_CONFIG.keymaps.select.load()[key] {
                        signal = action.act(self);
                        self.upkeep()?;
                    }
                }
                LineSelect(_) => {
                    if let Some(key) = event.key() && let Some(action) = GLOBAL_CONFIG.keymaps.line_select.load()[key] {
                        signal = action.act(self);
                        self.upkeep()?;
                    }
                }
            }
        }

        Ok(signal)
    }

    pub fn on_paste(&mut self, text: String) -> io::Result<()> {
        self.open_scratch_doc_with(text);
        self.upkeep()
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

    pub fn open_gadget(&mut self, gadget: impl Gadget + 'static) {
        self.gadget = Some(Box::new(gadget))
    }

    fn close_gadget(&mut self) {
        self.gadget = None
    }

    fn noop(&mut self) {}

    fn upkeep(&mut self) -> io::Result<()> {
        use crate::presenter::Present;
        self.doc.upkeep();
        self.draw()
    }
}

impl Editor {
    pub fn into_navigator(self) -> Navigator {
        let Self {
            filepath,
            doc,
            mut bg_docs,
            cmn,
            ..
        } = self;
        if let Some(fp) = filepath.clone() {
            bg_docs.insert_pathed(fp, doc);
        }
        Navigator::new(
            filepath,
            bg_docs,
            cmn,
        )
    }

    pub fn from_parts(
        doc: (Option<Arc<Path>>, Document),
        bg_docs: Documents,
        cmn: CommonState,
    ) -> Self {
        let (filepath, doc) = doc;
        Self {
            filepath,
            doc,
            bg_docs,
            gadget: None,
            cmn,
        }
    }
}