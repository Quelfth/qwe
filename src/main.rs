#![feature(never_type)]
#![feature(gen_blocks)]
#![feature(try_blocks)]

use std::{fs, io, panic, path::PathBuf};

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{self},
};

use crate::{editor::Editor, terminal_size::set_terminal_size};

mod aprintln;
mod constants;
mod custom_literal;
mod document;
mod draw;
mod editor;
mod grapheme;
mod ix;
mod lang;
mod pos;
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

fn main() -> io::Result<()> {
    let Args { path } = Args::parse();

    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        _ = setup::teardown();
        default_hook(info);
    }));
    setup::setup()?;
    let result = try {
        run(if let Some(path) = path {
            Some(PathedFile::open(path)?)
        } else {
            None
        })?
    };
    setup::teardown()?;
    result?;

    Ok(())
}

pub struct PathedFile {
    path: PathBuf,
    file: String,
}

impl PathedFile {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        Ok(Self {
            file: fs::read_to_string(&path)?,
            path,
        })
    }
}

fn run(file: Option<PathedFile>) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    set_terminal_size(width, height);

    let mut editor = Editor::new(file);

    editor.draw()?;

    loop {
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

    Ok(())
}
