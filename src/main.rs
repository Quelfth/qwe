#![feature(never_type)]
#![feature(gen_blocks)]
#![feature(try_blocks)]
#![feature(step_trait)]
#![feature(decl_macro)]
#![feature(new_range)]
//#![feature(share_trait)]

#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::blocks_in_conditions)]
#![allow(clippy::single_match)]

use std::{
    cell::Cell,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc},
};

use clap::{
    ArgAction::SetTrue,
    Parser,
};

use thiserror::Error;

use crate::{
    error::QweError,
    init::init,
    pos::Pos,
    run::run,
    setup::*,
};

mod action;
mod app;
mod aprintln;
mod color;
mod constants;
mod custom_literal;
mod document;
mod draw;
mod editor;
mod error;
mod global_config;
mod grapheme;
mod incremental_select;
mod init;
mod ix;
mod key;
mod keymap;
mod lang;
mod language_server;
mod log;
mod lsp;
mod markdown;
mod navigator;
mod pos;
mod pred;
mod presenter;
mod range_sequence;
mod range_tree;
mod rope;
mod run;
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
        num_args(std::ops::RangeFrom::from(0..)),
        conflicts_with("path"),
    )]
    find: Option<Vec<String>>,

    #[arg(
        short,
        long,
    )]
    line: Option<Pos>,

    #[arg(
        short = 's',
        long,
    )]
    autosave: bool,
}

thread_local! {
    static IS_MAIN_THREAD: Cell<bool> = const { Cell::new(false) };
}

fn is_main_thread() -> bool {
    IS_MAIN_THREAD.get()
}

fn main() {
    IS_MAIN_THREAD.set(true);

    let result = qwe(Args::parse());

    if let Err(error) = result {
        eprintln!("{error}");
    }
}

fn qwe(args: Args) -> Result<(), QweError> {
    let init = init(args)?;

    setup_panic_hook();
    setup().map_err(QweError::Setup)?;
    let result = run(init);
    teardown().map_err(QweError::Teardown)?;

    result?;

    Ok(())
}

struct PathedFile {
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

    fn open(path: Arc<Path>) -> io::Result<Self> {
        Ok(Self {
            file: fs::read_to_string(&path)?,
            path,
        })
    }

    fn create(path: Arc<Path>) -> io::Result<Self> {
        fs::File::create_new(&path)?;
        Ok(Self::empty(path))
    }

    fn create_with_dirs(path: Arc<Path>) -> Result<Self, CreateWithDirsPathedFileError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(CreateWithDirsPathedFileError::Dirs)?;
        }
        Self::create(path).map_err(CreateWithDirsPathedFileError::File)
    }
}

#[derive(Error, Debug)]
enum CreateWithDirsPathedFileError {
    #[error("{0}")]
    Dirs(io::Error),
    #[error("{0}")]
    File(io::Error),
}
