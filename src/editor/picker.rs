use std::{env, path::Path, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use lsp_types::DiagnosticSeverity;

use crate::{
    color, document::diagnostics::Severity, draw::screen::Canvas, editor::{Editor, gadget::Gadget}, grapheme::GraphemeExt, pos::{Pos, Utf16Pos, convert::ConvertableToPos}, style::Style
};

use super::gadget::ScreenRegion;

#[allow(unused)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub enum PickStyle {
    #[default]
    Default,
    Warning,
    Error,
}

pub struct Pick {
    string: String,
    style: PickStyle,
    file: Arc<Path>,
    pos: ConvertableToPos,
}

pub struct Picker {
    picks: Vec<Pick>,
    term: String,
    scroll: usize,
}

pub fn display_path(path: &Path) -> Option<String> {
    Some(
        path
            .strip_prefix(env::current_dir().ok()?)
            .ok().map(|path| path.to_string_lossy())
            .or_else(||
                path.strip_prefix(env::home_dir()?)
                    .ok()
                    .map(|path| {
                        let mut path = path.to_string_lossy();
                        path.to_mut().insert_str(0, "~/");
                        path
                    })
            )
            .unwrap_or(path.to_string_lossy())
            .to_string()
    )
}

impl Picker {
    fn r#type(&mut self, char: char) {
        self.term.push(char);
        self.scroll = 0;
    }

    fn backspace(&mut self) {
        self.term.pop();
        self.scroll = 0;
    }

    fn new(picks: Vec<Pick>) -> Self {
        Self {
            picks,
            term: String::new(),
            scroll: 0,
        }
    }

    pub fn file() -> Self {
        let mut picks = Vec::new();
        if let Ok(cwd) = &env::current_dir() {
            for entry in walkdir::WalkDir::new(cwd)
                .into_iter()
                .filter_map(|d| d.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let Some(string) = display_path(entry.path()) else {continue};
                picks.push(Pick {
                    string,
                    style: Default::default(),
                    file: entry.path().into(),
                    pos: Pos::ZERO.into(),
                })
            }
        }
        picks.sort_by_key(|p| p.string.len());
        Self::new(picks)
    }

    pub fn locations(locations: &[lsp_types::Location]) -> Self {
        let picks = locations
            .iter()
            .filter_map(|lsp_types::Location { uri, range }| {
                if uri.scheme() != "file" {
                    return None;
                }
                let path: Arc<Path> = uri.to_file_path().ok()?.into();
                let pos = Utf16Pos::from_lsp_pos(range.start).into();

                Some(Pick {
                    string: display_path(&path)?,
                    style: Default::default(),
                    file: path,
                    pos,
                })
            })
            .collect();
        Self::new(picks)
    }

    #[allow(unused)]
    pub fn diagnostics(diagnostics: &[(lsp_types::Url, Vec<lsp_types::Diagnostic>)]) -> Self {
        Self::new(
            diagnostics
                .iter()
                .filter_map(|(uri, diagnostics)| {
                    if uri.scheme() != "file" {
                        return None;
                    }
                    let path: Arc<Path> = uri.to_file_path().ok()?.into();
                    Some((path, diagnostics))
                })
                .flat_map(|(uri, diagnostics)| {
                    diagnostics.iter().map(move |diagnostic| (uri.clone(), diagnostic))
                })
                .filter_map(|(path, lsp_types::Diagnostic { range, severity, message, .. })| {
                    let pos = Utf16Pos::from_lsp_pos(range.start).into();

                    Some(Pick {
                        string: message.lines().next().unwrap_or("").to_owned(),
                        style: match *severity {
                            Some(DiagnosticSeverity::WARNING) => PickStyle::Warning,
                            Some(DiagnosticSeverity::ERROR) => PickStyle::Error,
                            _ => PickStyle::Default,
                        },
                        file: path,
                        pos,
                    })
                })
                .collect(),
        )
    }
}

impl Gadget for Picker {
    fn on_key(
        &mut self,
        event: crossterm::event::KeyEvent,
    ) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
        macro_rules! xx {
            ($($tokens: tt)*) => {
                Some(Box::new($($tokens)*))
            };
        }
        match event {
            KeyEvent {
                code: KeyCode::Char(char),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => {
                self.r#type(char);
                xx!(Editor::noop)
            }

            KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => {
                self.backspace();
                xx!(Editor::noop)
            }

            KeyEvent {
                code: KeyCode::Tab,
                kind: KeyEventKind::Press,
                ..
            } => {
                let terms = self
                    .term
                    .split_whitespace()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                self.picks
                    .retain(|p| terms.iter().all(|t| p.string.contains(t)));
                self.term.clear();
                xx!(Editor::noop)
            }

            KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            } => {
                let terms = self
                    .term
                    .split_whitespace()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                self.picks
                    .retain(|p| terms.iter().all(|t| p.string.contains(t)));
                if !self.picks.is_empty() {
                    let pick = self.picks.remove(0);
                    xx!(move |e| {
                        e.close_gadget();
                        e.open_file_doc_at(pick.file, pick.pos).unwrap();
                    })
                } else {
                    xx!(Editor::noop)
                }
            }

            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            } => {
                self.scroll += 4;
                xx!(Editor::noop)
            }
            KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            } => {
                self.scroll = self.scroll.saturating_sub(4);
                xx!(Editor::noop)
            }
            _ => None,
        }
    }

    fn screen_region(&self) -> ScreenRegion {
        ScreenRegion::RightPanel
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        for (i, g) in (0..canvas.width()).zip(self.term.graphemes()) {
            let cell = &mut canvas[(0, i)];
            cell.grapheme = g;
            cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into()
        }
        for (j, pick) in (2..canvas.height()).zip(
            self.picks
                .iter()
                .skip(self.scroll)
                .filter(|p| self.term.split_whitespace().all(|t| p.string.contains(t))),
        ) {
            for (i, g) in (0..canvas.width()).zip(pick.string.graphemes()) {
                let cell = &mut canvas[(j, i)];
                cell.grapheme = g;
                let fg = match pick.style {
                    PickStyle::Default => color::FG,
                    PickStyle::Warning => Severity::Warn.fg(),
                    PickStyle::Error => Severity::Err.fg(),
                };
                cell.style = (Style::fg(fg) + Style::bg(color::BG)).into();
            }
        }
    }
}
