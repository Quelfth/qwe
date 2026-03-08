use std::{env, path::Path, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    PathedFile, color,
    draw::screen::Canvas,
    editor::{Editor, gadget::Gadget},
    grapheme::GraphemeExt,
    pos::Pos,
    style::Style,
};

use super::gadget::ScreenRegion;

pub struct Pick {
    string: String,
    file: Arc<Path>,
    pos: Pos,
}

pub struct Picker {
    picks: Vec<Pick>,
    term: String,
    scroll: usize,
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
                let string = entry
                    .path()
                    .strip_prefix(cwd)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();
                picks.push(Pick {
                    string,
                    file: entry.path().into(),
                    pos: Pos::ZERO,
                })
            }
        }
        Self {
            picks,
            term: String::new(),
            scroll: 0,
        }
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
                        e.open_new_doc(PathedFile::open(pick.file).unwrap());
                        e.jump_to(pick.pos);
                        e.doc.scroll_main_cursor_on_screen();
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
                cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into();
            }
        }
    }
}
