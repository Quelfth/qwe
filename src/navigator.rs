use std::{io, path::{Path, PathBuf}};

use crate::{AppState, aprintln::aprintln, color, draw::screen::Canvas, editor::{clipboard::Clipboard, keymap::Keymaps}, grapheme::{Grapheme, GraphemeExt}, language_server::LspContext, navigator::directory::Entry, presenter::{Present, Presenter}, style::Style};

use crossterm::style::Color;
use directory::Directory;

mod directory;
mod pane;

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
        presenter: Presenter,
    ) -> Self {
        let home = std::env::home_dir();
        let cwd = std::env::current_dir().ok();
        let path = path.and_then(|p| p.as_ref().canonicalize().ok()).or_else(|| cwd.clone()).unwrap();

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

impl AppState for Navigator {
    fn poll(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Present for Navigator {
    fn presenter(&self) -> &Presenter { &self.presenter }
    fn bg_color(&self) -> Color { color::NAV_DEEP_BG }

    fn present(&self, mut canvas: Canvas<'_>) -> io::Result<()> {
        let root_pane = self.root_pane();
        let root_text = root_pane.text();

        let mut margin = 0;
        for (i, g) in (0..canvas.width()).zip(root_text.graphemes()) {
            let cell = &mut canvas[(0, i)];
            cell.grapheme = g;
            cell.style = (Style::fg(color::NAV_FG) + Style::bg(color::NAV_BG)).into();
            margin = i;
        }

        let width = root_text.graphemes().count();

        for j in 1..canvas.height() {
            for i in 0..canvas.width().min(width as u16) {
                let cell = &mut canvas[(j, i)];
                cell.grapheme = Grapheme::SPACE;
                cell.style = (Style::fg(color::NAV_FG) + Style::bg(color::NAV_BG)).into();
            }
        }

        let rel_path = self.path.strip_prefix(&self.root_path).unwrap_or(&self.path);
        let mut components = rel_path.components();
        
        let mut prev_margin = margin + 2;
        let mut next_dir = Some(&self.root_dir);
        while let Some(dir) = next_dir {
            for (j, e) in (0..canvas.height()).zip(dir.display_entries()) {
                for (i, g) in (prev_margin..canvas.width()).zip(e.graphemes()) {
                    let cell = &mut canvas[(j, i)];
                    cell.grapheme = g;
                    cell.style = (Style::fg(color::NAV_FG) + Style::bg(color::NAV_BG)).into();
                    margin = i.max(margin);
                }
            }
            next_dir = if let Some(component) = components.next() 
                && let Some(entry) = dir.get(component.as_os_str()) 
                && let Entry::Directory(dir) = entry
            {
                Some(dir)
            } else {
                None
            };
            prev_margin = margin + 2;
        }

        Ok(())
    }
}