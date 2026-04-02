#![feature(never_type)]
#![feature(gen_blocks)]
#![feature(try_blocks)]
#![feature(step_trait)]
#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::blocks_in_conditions)]

use std::{
    cell::Cell,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc},
    time::Duration,
};

use clap::{ArgAction::SetTrue, Parser};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, poll},
    terminal::{self},
    style::Color,
};

use dispa::dispatch;
use tokio::sync::mpsc;

use crate::{
    editor::{Editor, keymap::InputEvent},
    ix::Ix, 
    draw::screen::Canvas,
    lsp::{channel::EditorToLspMessage, run_lsp_thread},
    navigator::Navigator,
    pos::Pos,
    presenter::{Present, Presenter},
    terminal_size::set_terminal_size,
};

mod aprintln;
mod color;
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
mod navigator;
mod pos;
mod pred;
mod presenter;
mod range_sequence;
mod range_tree;
mod rope;
mod setup;
mod style;
mod terminal_size;
mod theme;
mod timeline;
mod ts;
mod util;
mod cli;

#[derive(Parser)]
struct Args {
    path: Option<PathBuf>,
    #[arg(
        short, 
        long, 
        action = SetTrue,
        requires("path"),
    )]
    new: bool,
    #[arg(
        short,
        long,
        action = SetTrue,
        requires("new"),
    )]
    dirs: bool,
    #[arg(
        short,
        long,
        num_args(0..),
        conflicts_with("path"),
    )]
    find: Option<Vec<String>>,
    #[arg(
        short,
        long,
    )]
    line: Option<Pos>,
}

thread_local! {
    static IS_MAIN_THREAD: Cell<bool> = const { Cell::new(false) };
}

fn is_main_thread() -> bool {
    IS_MAIN_THREAD.get()
}

fn main() -> io::Result<()> {
    IS_MAIN_THREAD.set(true);
    let Args {
        path,
        new,
        dirs,
        find,
        line,
    } = Args::parse();
    let path = if let Some(path) = path {
        Some(path)
    } else {
        if let Some(find) = find {
            let dir = std::env::current_dir()?;
            let mut options = walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_owned())
                .collect::<Vec<_>>();
            options.sort_by_key(|o| o.as_os_str().len());
            Some(options
                .into_iter()
                .find(|e| find.iter().all(|f| e.to_string_lossy().contains(f))).unwrap())
        } else {
            None
        }
    };
    let path = if let Some(path) = path {
        Some(if !new {
            PathedFile::open(path.into())?
        } else if !dirs {
            PathedFile::create(path.into())?
        } else {
            PathedFile::create_with_dirs(path.into())?
        })
    } else {
        None
    };

    setup::setup_panic_hook();
    setup::setup()?;
    let result = run(path, line);
    setup::teardown()?;

    result?;

    Ok(())
}

pub struct PathedFile {
    path: Arc<Path>,
    file: String,
}

impl PathedFile {
    fn empty(path: Arc<Path>) -> Self {
        Self {
            file: "".to_owned(),
            path,
        }
    }

    pub fn open(path: Arc<Path>) -> io::Result<Self> {
        Ok(Self {
            file: fs::read_to_string(&path)?,
            path,
        })
    }

    pub fn create(path: Arc<Path>) -> io::Result<Self> {
        fs::File::create_new(&path)?;
        Ok(Self::empty(path))
    }

    pub fn create_with_dirs(path: Arc<Path>) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Self::create(path)
    }
}

fn run(file: Option<PathedFile>, pos: Option<Pos>) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    set_terminal_size(width, height);

    let mut editor = Editor::new();
    let (send_lsp_to_editor, recv_lsp_to_editor) = std::sync::mpsc::channel();
    let (send_editor_to_lsp, recv_editor_to_lsp) = mpsc::unbounded_channel();
    editor.set_lsp_channels(send_editor_to_lsp, recv_lsp_to_editor);
    if let Some(file) = file {
        _= editor.open_file_doc(file.path);
    } else {
        editor.open_scratch_doc();
    }

    let _lsp_thread_handle = run_lsp_thread(lsp::channel::LspChannels {
        outgoing: send_lsp_to_editor,
        incoming: recv_editor_to_lsp,
    })?;

    if let Some(pos) = pos {
        editor.jump_to(pos);
        *editor.doc().view_height.lock() = Ix::new(height as _);
        editor.scroll_main_cursor_on_screen();
    }
    editor.draw()?;

    #[dispatch(AppState)]
    #[dispatch(Present)]
    enum State {
        Editor(Editor),
        Navigator(Navigator),
    }

    let mut state = State::Editor(editor);

    loop {
        if poll(Duration::from_millis(2))? {
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
                    KeyEvent {
                        code: KeyCode::Char('n'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        ..
                    } if let State::Editor(editor) = state => {
                        state = State::Navigator(editor.into_navigator());
                        state.draw()?;
                    },
                    KeyEvent {
                        code: KeyCode::Char('e'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        ..
                    } if let State::Navigator(navigator) = state => {
                        state = State::Editor(navigator.into_editor());
                        state.draw()?;
                    },
                    event => state.on_key_event(InputEvent::Event(event))?,
                },
                Event::Mouse(event) => state.on_mouse_event(event)?,
                Event::Paste(_) => (),
                Event::Resize(width, height) => {
                    if set_terminal_size(width, height) {
                        state.draw()?
                    }
                }
            }
        }
        state.poll()?;
    }
    if let State::Editor(editor) = state {
        if let Some(cx) = editor.lsp {
            cx.tx.send(EditorToLspMessage::Exit).unwrap();
        }
    }

    Ok(())
}

#[dispatch]
pub trait AppState {
    fn poll(&mut self) -> io::Result<()>;
    fn on_key_event(&mut self, #[expect(unused)] event: InputEvent) -> io::Result<()> { Ok(()) }
    fn on_mouse_event(&mut self, #[expect(unused)] event: MouseEvent) -> io::Result<()> { Ok(()) }
}

