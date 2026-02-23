#![feature(never_type)]
#![feature(gen_blocks)]
#![feature(try_blocks)]
#![feature(super_let)]
#![feature(step_trait)]
#![feature(range_into_bounds)]
#![feature(range_bounds_is_empty)]
#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]
#![allow(clippy::large_enum_variant)]

use std::{
    cell::Cell,
    fs, io,
    panic::{self},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll},
    terminal::{self},
};

use crate::{
    aprintln::{aprint, aprintln},
    editor::Editor,
    lsp::{channel::EditorToLspMessage, run_lsp_thread},
    terminal_size::set_terminal_size,
};

mod aprintln;
mod constants;
mod custom_literal;
mod document;
mod draw;
mod editor;
mod grapheme;
mod incremental_select;
mod ix;
mod lang;
mod language_server;
mod lsp;
mod pos;
mod pred;
mod range_sequence;
mod range_tree;
mod rope;
mod setup;
mod style;
mod terminal_size;
mod theme;
mod ts;
mod util;

#[derive(Parser)]
struct Args {
    path: Option<PathBuf>,
}

thread_local! {
    static IS_MAIN_THREAD: Cell<bool> = const { Cell::new(false) };
}

fn is_main_thread() -> bool {
    IS_MAIN_THREAD.get()
}

fn main() -> io::Result<()> {
    IS_MAIN_THREAD.set(true);
    let Args { path } = Args::parse();

    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if !is_main_thread() {
            if let Some(location) = info.location() {
                aprint!(
                    "[{}:{}|{}] ",
                    location.file(),
                    location.line(),
                    location.column(),
                );
            }
            if let Some(payload) = info.payload_as_str() {
                aprintln!("Panic: {}", payload);
            } else {
                aprintln!("Panic!")
            }
            return;
        }
        _ = setup::teardown();
        default_hook(info);
    }));
    setup::setup()?;
    let result = try {
        let path = if let Some(path) = path {
            Some(PathedFile::open(path.into())?)
        } else {
            None
        };
        run(path)?
    };
    setup::teardown()?;
    result?;

    Ok(())
}

pub struct PathedFile {
    path: Arc<Path>,
    file: String,
}

impl PathedFile {
    pub fn open(path: Arc<Path>) -> io::Result<Self> {
        Ok(Self {
            file: fs::read_to_string(&path)?,
            path,
        })
    }
}

fn run(file: Option<PathedFile>) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    set_terminal_size(width, height);

    let mut editor = Editor::new();
    let (send_lsp_to_editor, recv_lsp_to_editor) = std::sync::mpsc::channel();
    let (send_editor_to_lsp, recv_editor_to_lsp) = std::sync::mpsc::channel();
    editor.set_lsp_channels(send_editor_to_lsp, recv_lsp_to_editor);
    if let Some(file) = file {
        editor.open_new_doc(file);
    }

    let _lsp_thread_handle = run_lsp_thread(lsp::channel::LspChannels {
        outgoing: send_lsp_to_editor,
        incoming: recv_editor_to_lsp,
    })?;

    editor.draw()?;

    loop {
        if poll(Duration::from_millis(20))? {
            match event::read()? {
                Event::FocusGained => (),
                Event::FocusLost => (),
                Event::Key(event) => match event {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        ..
                    } => break,
                    event => editor.on_key_event(event)?,
                },
                Event::Mouse(_) => (),
                Event::Paste(_) => (),
                Event::Resize(width, height) => {
                    if set_terminal_size(width, height) {
                        editor.draw()?
                    }
                }
            }
        }
        editor.poll()?;
    }
    if let Some(channel) = editor.lsp_send {
        channel.send(EditorToLspMessage::Exit).unwrap();
    }

    Ok(())
}
