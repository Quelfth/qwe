use std::{
    ffi::OsStr,
    fs,
    io,
    path::{self, Path, PathBuf},
    range::Range,
    sync::Arc,
};

use crate::{
    action::Action as _,
    app::{AppSignal, AppState, CommonState},
    color,
    document::Document,
    draw::{Rect, screen::Canvas},
    editor::{Editor, documents::Documents},
    global_config::GLOBAL_CONFIG,
    grapheme::{Grapheme, GraphemeExt},
    key::{KeyOrChar, key},
    lang::Language,
    language_server::LanguageServer,
    log::{DebugLog, log},
    lsp::channel::{EditorToLspMessage, LspToEditorMessage},
    navigator::directory::{Entry, FileDocument},
    pathed_file::PathedFile,
    presenter::{Present, Presenter},
    range_sequence::RangeSequence,
    style::Style,
    theme::theme,
    util::flip,
};

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

    docs: Documents,

    cmn: CommonState,

    name_box: Option<NameBox>,
}

pub struct NameBox {
    effect: NameBoxEffect,
    name: String,
}

impl NameBox {
    pub fn new_new() -> Self {
        Self { effect: NameBoxEffect::New, name: String::new() }
    }
    pub fn new_rename() -> Self {
        Self { effect: NameBoxEffect::Rename, name: String::new() }
    }
}

pub enum NameBoxEffect {
    New,
    Rename,
}

impl Navigator {
    pub fn new(
        path: Option<impl AsRef<Path>>,
        docs: Documents,
        cmn: CommonState,
    ) -> Self {
        let home = std::env::home_dir();
        let cwd = std::env::current_dir().ok();
        let path = path.and_then(|p| p.as_ref().canonicalize().ok()).or_else(|| cwd.clone()).unwrap();

        let mut root_path = &*path.canonicalize().unwrap();
        while home.as_ref().is_none_or(|h| h != root_path) && cwd.as_ref().is_none_or(|h| h != root_path) && let Some(parent) = root_path.parent() {
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

            cmn,

            name_box: None,
        }
    }

    pub fn into_editor(self) -> Editor {
        let Self { path, mut docs, cmn, .. } = self;
        let doc = docs.extract_by_path(&path)
            .map(|d| (Some(path.into()), d))
            .unwrap_or_default();

        Editor::from_parts(
            doc,
            docs,
            cmn,
        )
    }

    fn rel_path(&self) -> &Path {
        self.path.strip_prefix(&self.root_path).unwrap_or(&self.path)
    }

    fn parent_dir(&self) -> &Directory {
        let mut dir = &self.root_dir;
        let mut components = self.rel_path().components().peekable();
        while let Some(component) = components.next() {
            if components.peek().is_none() { break }
            if let Some(Entry::Directory(next)) = dir.get(component.as_os_str()) {
                dir = next;
            }
        }
        dir
    }

    fn reload(&mut self) {
        self.root_dir = Directory::collect(&self.root_path, &self.docs);
    }

    pub fn navigate_down(&mut self) -> Option<()> {
        use std::ops::Bound::*;
        let next = self.parent_dir()
            .entries()
            .range::<OsStr, _>((
                Excluded(self.path.file_name().unwrap()),
                Unbounded,
            )).next()?;

        self.path = self.path.parent()?.join(next.0);
        Some(())
    }

    pub fn navigate_up(&mut self) -> Option<()> {
        use std::ops::Bound::*;
        let next = self.parent_dir()
            .entries()
            .range::<OsStr, _>((
                Unbounded,
                Excluded(self.path.file_name().unwrap()),
            )).next_back()?;
        log!(DebugLog(self.path.file_name()));
    
        let parent = self.path.parent()?;
        self.path = parent.join(next.0);
        Some(())
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

    pub fn navigate_anywhere(&mut self) {
        if self.navigate_up().is_some() { return }
        if self.navigate_down().is_some() { return }
        self.navigate_out();
    }

    pub fn delete_empty(&mut self) {
        let Ok(metadata) = fs::symlink_metadata(&self.path) else { return };
        if metadata.is_dir() {
            if fs::remove_dir(&self.path).is_ok() {
                self.navigate_anywhere();
                self.reload();
            }
        } else if metadata.is_file()
            && metadata.len() == 0
            && fs::remove_file(&self.path).is_ok() {
                self.navigate_anywhere();
                self.reload();
            }
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
        if let Some(lsp) = &self.cmn.lsp
            && let Some(key) = doc_key
            && let Some(doc) = self.docs.by_key(key)
            && let Some(lang) = doc.language()
            && let Some(path) = self.docs.path_from_key(key)
        {
            _=lsp.tx.send(EditorToLspMessage::OpenDoc { lang, path, text: doc.text().to_string() });
        }
    }

    pub fn new_child(&mut self) {
        self.name_box = Some(NameBox::new_new())
    }

    pub fn new_sibling(&mut self) {
        self.navigate_out();
        self.new_child();
    }

    pub fn rename(&mut self) {
        self.name_box = Some(NameBox::new_rename())
    }

    pub fn update_and_draw(&mut self) -> io::Result<()> {
        self.open_selected();
        self.draw()?;
        Ok(())
    }
}

impl AppState for Navigator {
    fn poll(&mut self) -> io::Result<()> {
        if let Some(lsp) = &self.cmn.lsp {
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
                            self.cmn.presenter.defer_draw();
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

    fn on_key_or_char(&mut self, event: KeyOrChar) -> io::Result<Option<AppSignal>> {
        use KeyOrChar::Key;
        if let Some(NameBox { name, .. }) = &mut self.name_box {
            match event {
                    Key(key![esc]) => {self.name_box = None; self.update_and_draw()?},
                    Key(key![backspace]) => {name.pop(); self.update_and_draw()?}
                    Key(key![return]) => {
                        let NameBox { effect, mut name } = self.name_box.take().unwrap();
                        match effect {
                            NameBoxEffect::New => if let Some(last) = name.chars().next_back() && path::is_separator(last) {
                                name.pop();
                                if fs::create_dir(self.path.join(&name)).is_ok() {
                                    self.path = self.path.join(name);
                                }
                            } else {
                                if fs::File::create_new(self.path.join(&name)).map(drop).is_ok() {
                                    self.path = self.path.join(name);
                                }
                            },
                            NameBoxEffect::Rename => {
                                let mut path = self.path.clone();
                                path.set_file_name(name);
                                _ = fs::rename(&self.path, path);
                            }
                        }
                        self.reload();
                        self.update_and_draw()?;
                    }
                    key if let Some(c) = key.char() => {name.push(c); self.update_and_draw()?}
                    _ => (),
            }
            return Ok(None);
        }

        let mut signal = None;

        if let Key(key) = event && let Some(action) = GLOBAL_CONFIG.keymaps.navigator.load()[key] {
            signal = action.act(self);
            self.update_and_draw()?;
        }

        Ok(signal)
    }
}

impl Navigator {
    fn main_draw(&self, mut canvas: Canvas<'_>) -> io::Result<()> {
        let root_pane = self.root_pane();
        let root_text = root_pane.text();

        let warning_style = theme().highlight(&[&["diagnostic", "warning"]]);
        let error_style = theme().highlight(&[&["diagnostic", "error"]]);

        let mut margin = 0;
        for (i, g) in (0..canvas.width()).into_iter().zip(root_text.graphemes()) {
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
            let selected_ix = entries.iter().position(|&(n, _)| matches!(next_component, Some(component) if component.as_os_str() == n)).unwrap_or_default();
            let scroll = selected_ix
                .saturating_sub(canvas.height() as usize / 2)
                .min(entries.len() - canvas.height() as usize);
            let mut rows = (0..canvas.height()).into_iter();
            for (j, (n, e)) in entries.into_iter().skip(scroll).zip(rows.by_ref()).map(flip) {
                let selected = matches!(next_component, Some(component) if component.as_os_str() == n);
                let entry = dir.get(n).unwrap();
                let diagnostic_status = entry.diagnostic_status(&self.docs);
                let diagnostic_style = if diagnostic_status.errors > 0 {
                    error_style
                } else if diagnostic_status.warnings > 0 {
                    warning_style
                } else {
                    Style::default()
                };
                let bg = decide_bg(alt != selected);
                let mut cols = (prev_margin..next_margin).into_iter();
                for (i, g) in e.graphemes().zip(cols.by_ref()).map(flip) {
                    let cell = &mut canvas[(j, i)];
                    cell.grapheme = g;
                    cell.style = (Style::fg(color::NAV_FG) + Style::bg(bg) + diagnostic_style).into();
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
            alt.toggle();
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

impl Present for Navigator {
    fn presenter(&self) -> &Presenter { &self.cmn.presenter }
    fn bg_color(&self) -> Color { color::NAV_DEEP_BG }

    fn present(&self, mut canvas: Canvas<'_>) -> io::Result<()> {
        self.main_draw(canvas.reborrow())?;
        if let Some(NameBox { name, .. }) = &self.name_box {
            _ = canvas.at((canvas.height() / 2, canvas.width() / 2)).write(name, Style::fg(color::FG) + Style::bg(color::BG));
        }
        Ok(())
    }
}