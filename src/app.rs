use std::{io, path::Path, sync::Arc};

use dispa::dispatch;

use crate::{editor::clipboard::Clipboard, key::KeyOrChar, language_server::LspContext, presenter::Presenter, timeline::{Timeline, global::GlobalEvent}};

pub enum AppSignal {
    Quit,
    Editor,
    Navigator,
}

#[dispatch]
pub trait AppState {
    fn poll(&mut self) -> io::Result<()>;

    fn on_key_or_char(&mut self, event: KeyOrChar) -> io::Result<Option<AppSignal>> { _=event; Ok(None) }

    fn on_paste(&mut self, text: String) -> io::Result<()> { _=text; Ok(()) }
}

#[derive(Default)]
pub struct CommonState {
    pub file_history: Vec<Arc<Path>>,
    pub file_future: Vec<Arc<Path>>,
    pub global_timeline: Timeline<GlobalEvent>,
    pub clipboard: Clipboard,
    pub lsp: Option<LspContext>,
    pub presenter: Presenter,
}
