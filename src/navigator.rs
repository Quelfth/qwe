use std::{io, path::{Path, PathBuf}};

use crate::{AppState, color, draw::screen::Canvas, editor::{clipboard::Clipboard, keymap::Keymaps}, grapheme::{Grapheme, GraphemeExt}, language_server::LspContext, navigator::directory::Entry, presenter::{Present, Presenter}, style::Style, util::flip};

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

        let mut alt = true;
        
        let mut prev_margin = margin + 2;
        let mut next_dir = Some(&self.root_dir);
        while let Some(dir) = next_dir {
            let bg = if alt { color::NAV_BG_ALT } else { color::NAV_BG };
            let entries = dir.display_entries().collect::<Vec<_>>();
            let width = entries.iter().map(|e| e.graphemes().count()).max().unwrap_or_default() as u16;
            let next_margin = prev_margin + width;
            let mut rows = 0..canvas.height();
            for (j, e) in entries.into_iter().zip(rows.by_ref()).map(flip) {
                let mut cols = prev_margin..next_margin;
                for (i, g) in e.graphemes().zip(cols.by_ref()).map(flip) {
                    let cell = &mut canvas[(j, i)];
                    cell.grapheme = g;
                    cell.style = (Style::fg(color::NAV_FG) + Style::bg(bg)).into();
                }
                for i in cols {
                    let cell = &mut canvas[(j, i)];
                    cell.style.bg = bg;
                }
            }
            for j in rows {
                for i in prev_margin..next_margin {
                    let cell = &mut canvas[(j, i)];
                    cell.style.bg = bg;
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
            prev_margin = next_margin + 1;
            alt ^= true;
        }

        Ok(())
    }
}