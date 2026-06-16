use std::io;

use dispa::dispatch;

use crate::key::KeyOrChar;

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