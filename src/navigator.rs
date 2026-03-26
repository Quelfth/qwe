use std::path::{Path, PathBuf};

use crate::{editor::{clipboard::Clipboard, keymap::Keymaps}, language_server::LspContext, presenter::Presenter};

use directory::Directory;

mod directory;

pub struct Navigator {
    home: Option<PathBuf>,
    cwd: Option<PathBuf>,
    root_path: PathBuf,
    root_dir: Directory,

    path: PathBuf,

    keymap: Keymaps,
    clipboard: Clipboard,
    lsp: Option<LspContext>,
    presenter: Presenter,
}

impl Navigator {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        keymap: Keymaps,
        clipboard: Clipboard,
        lsp: Option<LspContext>,
        presenter: Presenter
    ) -> Self {
        let home = std::env::home_dir();
        let cwd = std::env::current_dir().ok();
        let path = path.map(|p| p.as_ref().to_owned()).or_else(|| cwd.clone()).unwrap();

        let mut root_path = &*path.canonicalize().unwrap();
        while home.as_ref().is_none_or(|h| h != &root_path) && cwd.as_ref().is_none_or(|h| h != &root_path) && let Some(parent) = root_path.parent() {
            root_path = parent;
        }
        let root_path = root_path.to_owned();
        let root_dir = Directory::collect(&root_path);

        Self {
            home,
            cwd,
            root_path,
            root_dir,

            path,
            keymap,
            clipboard,
            lsp,
            presenter,
        }
    }
}