use std::{ffi::OsStr, io, path::{Path, PathBuf}, sync::Arc};

use crate::{AppState, PathedFile, color, document::Document, draw::{Range, Rect, screen::Canvas}, editor::{Editor, clipboard::Clipboard, documents::Documents, keymap::{InputEvent, Keymaps}}, grapheme::{Grapheme, GraphemeExt}, lang::Language, language_server::{LanguageServer, LspContext}, lsp::channel::{EditorToLspMessage, LspToEditorMessage}, navigator::directory::{Entry, FileDocument}, presenter::{Present, Presenter}, range_sequence::RangeSequence, style::Style, util::flip};

use crossterm::{event::{KeyCode, KeyEvent}, style::Color};
use directory::Directory;

mod directory;
mod pane;

pub struct Navigator {
    home: Option<PathBuf>,
    cwd: Option<PathBuf>,
    root_path: PathBuf,
    root_dir: Directory,

    path: PathBuf,

    docs: Documents,

    keymap: Keymaps,
    clipboard: Clipboard,
    lsp: Option<LspContext>,
    presenter: Presenter,
}

impl Navigator {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        docs: Documents,
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
        let root_dir = Directory::collect(&root_path, &docs);

        Self {
            home,
            cwd,
            root_path,
            root_dir,

            path,
            docs,

            keymap,
            clipboard,
            lsp,
            presenter,
        }
    }

    pub fn into_editor(self) -> Editor {
        let Self { path, mut docs, keymap, clipboard, lsp, presenter, .. } = self;
        let doc = docs.extract_by_path(&path)
            .map(|d| (Some(path.into()), d))
            .unwrap_or_default();
        
        Editor::from_parts(
            doc,
            docs,
            keymap,
            clipboard,
            lsp,
            presenter,
        )
    }

    fn rel_path(&self) -> &Path {
        self.path.strip_prefix(&self.root_path).unwrap_or(&self.path)
    }

    pub fn navigate_down(&mut self) {
        let mut dir = &self.root_dir;
        let mut components = self.rel_path().components().collect::<Vec<_>>();
        let Some(final_component) = components.pop() else { return };
        for component in components {
            if let Some(Entry::Directory(next)) = dir.get(component.as_os_str()) {
                dir = next;
            }
        }
        use std::ops::Bound::*;
        let Some(next) = dir.entries().range::<OsStr, _>((Excluded(final_component.as_os_str()), Unbounded)).next() else { return };

        let Some(parent) = self.path.parent() else { return };
        self.path = parent.join(next.0);
    }

    pub fn navigate_up(&mut self) {
        let mut dir = &self.root_dir;
        let mut components = self.rel_path().components().collect::<Vec<_>>();
        let Some(final_component) = components.pop() else { return };
        for component in components {
            if let Some(Entry::Directory(next)) = dir.get(component.as_os_str()) {
                dir = next;
            }
        }
        use std::ops::Bound::*;
        let Some(next) = dir.entries().range::<OsStr, _>((Unbounded, Excluded(final_component.as_os_str()))).next_back() else { return };
    
        let Some(parent) = self.path.parent() else { return };
        self.path = parent.join(next.0);
    }

    pub fn navigate_out(&mut self) {
        self.path.pop();
    }

    pub fn navigate_in(&mut self) {
        let mut dir = &self.root_dir;
        let components = self.rel_path().components().collect::<Vec<_>>();
        for component in components {
            let Some(Entry::Directory(next)) = dir.get(component.as_os_str()) else {
                return;
            };
            dir = next;
        }
        let Some(next) = dir.entries().iter().next() else { return };
        self.path = self.path.join(next.0);
    }

    pub fn open_selected(&mut self) {
        let mut components = self.rel_path().components().map(|c| c.as_os_str().to_owned()).collect::<Vec<_>>();
        let mut dir = &mut self.root_dir;
        let Some(final_component) = components.pop() else { return };
        for component in components {    
            dir = if let Some(Entry::Directory(next)) = dir.get_mut(component.as_os_str()) {
                next
            } else { return };
        }

        let Some(entry) = dir.get_mut(&final_component) else { return };

        let Entry::File { doc, .. } = entry else { return };
        if !matches!(doc, FileDocument::OnDisk) { return };

        let doc_key = self.docs.key_from_path(&self.path).or_else(|| {
            let path: Arc<Path> = self.path.clone().into();
            let PathedFile { path, file } = PathedFile::open(path.clone()).ok()?;
            let new_doc = Document::new(
                path.extension()
                    .and_then(|e| Language::from_file_ext(&e.to_string_lossy())),
                file,
                Some(Default::default()),
            );

            Some(self.docs.insert_pathed(path, new_doc))
        });

        *doc = if let Some(key) = doc_key { FileDocument::Text(key) } else { FileDocument::Binary };
        if let Some(lsp) = &self.lsp
            && let Some(key) = doc_key
            && let Some(doc) = self.docs.by_key(key)
            && let Some(lang) = doc.language()
            && let Some(path) = self.docs.path_from_key(key)
        {
            _=lsp.tx.send(EditorToLspMessage::OpenDoc { lang, path, text: doc.text().to_string() });
        }
    }

    pub fn update_and_draw(&mut self) -> io::Result<()> {
        self.open_selected();
        self.draw()?;
        Ok(())
    }
}

impl AppState for Navigator {
    fn poll(&mut self) -> io::Result<()> {
        if let Some(lsp) = &self.lsp {
            while let Ok(msg) = lsp.rx.try_recv() {
                use LspToEditorMessage::*;
                match msg {
                    NewLsp { lang, init_result } =>
                        lsp.servers.lock()
                            .entry(lang)
                            .or_default()
                            .push(LanguageServer::new(init_result)),
                    SemanticTokens { uri, tokens } => {
                        if uri.scheme() == "file"
                            && let Ok(path) = uri.to_file_path()
                            && let Ok(path) = path.canonicalize()
                            && let Some(doc) = self.docs.by_path_mut(&path)
                            && let Some(lang) = doc.language()
                            && let Some(servers) = lsp.servers.lock().get(&lang)
                        {
                            doc.semtoks = RangeSequence::from_abs_ordered(servers[0].translate_semtoks(tokens, doc.text()));
                            self.presenter.defer_draw();
                        }
                    },
                    Diagnostics { .. } => (),
                    _ => (),
                }
            }
        }

        self.poll_draw()?;
        Ok(())
    }

    fn on_key_event(&mut self, event: InputEvent) -> io::Result<()> {
        match event {
            InputEvent::Event(key_event) => match key_event {
                KeyEvent { code: KeyCode::Char('j'), .. } => {self.navigate_down(); self.update_and_draw()?},
                KeyEvent { code: KeyCode::Char('k'), .. } => {self.navigate_up(); self.update_and_draw()?},
                KeyEvent { code: KeyCode::Char('h'), .. } => {self.navigate_out(); self.update_and_draw()?},
                KeyEvent { code: KeyCode::Char('l'), .. } => {self.navigate_in(); self.update_and_draw()?},
                _ => (),
            },
            InputEvent::Key(_) => todo!(),
        }
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
            cell.style = (Style::fg(color::NAV_FG) + Style::bg(color::NAV_BG_ALT)).into();
            margin = i;
        }
        let width = root_text.graphemes().count();

        canvas[(0, width as u16)].style.bg = color::NAV_BG_ALT;

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
        let mut next_dir = Ok(&self.root_dir);
        while let Ok(dir) = next_dir {
            let next_component = components.next();
            next_dir = if let Some(component) = next_component
                && let Some(entry) = dir.get(component.as_os_str())
            {
                match entry {
                    Entry::Directory(directory) => Ok(directory),
                    Entry::File { name, doc } => Err(Some((name, doc))),
                    Entry::Link(_) => Err(None),
                }
            } else {
                Err(None)
            };
            const fn decide_bg(alt: bool) -> Color {
                if alt {
                    color::NAV_BG_ALT
                } else {
                    color::NAV_BG
                }
            }
            let bg = decide_bg(alt);
            let entries = dir.display_entries().collect::<Vec<_>>();
            let width = entries.iter().map(|(_, e)| e.graphemes().count()).max().unwrap_or_default() as u16;
            let next_margin = prev_margin + width;
            let mut rows = 0..canvas.height();
            for (j, (n, e)) in entries.into_iter().zip(rows.by_ref()).map(flip) {
                let selected = matches!(next_component, Some(component) if component.as_os_str() == n);
                let bg = decide_bg(alt != selected);
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
                if selected {
                    canvas[(j, next_margin)].style.bg = bg;
                }
            }
            for j in rows {
                for i in prev_margin..next_margin {
                    let cell = &mut canvas[(j, i)];
                    cell.style.bg = bg;
                }
            }
            prev_margin = next_margin + 1;
            alt ^= true;
        }

        if let Err(Some((_, doc))) = next_dir {
            match doc {
                FileDocument::Text(doc_key) => {
                    let doc = self.docs.by_key(*doc_key);
                    if let Some(doc) = doc {
                        doc.draw(canvas.region(Rect { rows: Range { start: 0, end: canvas.height() }, cols: Range { start: prev_margin, end: canvas.width() } }));
                    }
                },
                FileDocument::Binary => (),
                FileDocument::OnDisk => (),
            }
        }

        Ok(())
    }
}