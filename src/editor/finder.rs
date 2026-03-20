use std::ops::Range;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use resharp::{Match, Regex};

use crate::{
    color,
    draw::screen::Canvas,
    editor::{Editor, gadget::Gadget},
    grapheme::GraphemeExt,
    ix::{Byte, Ix},
    style::Style,
};

pub struct Haystack {
    pub text: String,
    pub offset: usize,
}

pub struct Finder {
    haystacks: Vec<Haystack>,
    regex: String,
}

impl Gadget for Finder {
    fn on_key(&mut self, event: KeyEvent) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
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
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            } => self.find().map(|f| {
                let x: Box<dyn FnOnce(&mut Editor)> = Box::new(|e: &mut Editor| {
                    e.close_gadget();
                    if e.select_ranges(f).is_ok() && !e.doc.main_cursor_is_visible() {
                        e.scroll_to_main_cursor();
                    }
                });
                x
            }),

            KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            } => {
                xx!(Editor::close_gadget)
            }
            _ => None,
        }
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        for (i, g) in (0..canvas.width()).zip(self.regex.graphemes()) {
            let cell = &mut canvas[(0, i)];
            cell.grapheme = g;
            cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into()
        }
    }
}

impl Finder {
    pub fn new(haystacks: Vec<Haystack>) -> Self {
        Self {
            haystacks,
            regex: String::new(),
        }
    }

    pub fn r#type(&mut self, char: char) {
        self.regex.push(char);
    }

    pub fn backspace(&mut self) {
        self.regex.pop();
    }

    pub fn find(&self) -> Option<Vec<Range<Ix<Byte>>>> {
        let re = Regex::new(&format!{"({}){}", self.regex, r"&\p{utf8}"}).ok()?;

        Some(
            self.haystacks
                .iter()
                .flat_map(|Haystack { text, offset }|
                    re.find_all(text.as_bytes()).ok().into_iter().flat_map(move |m|
                        m.into_iter().map(move |Match { start, end }|
                            Ix::new(start + offset)..Ix::new(end + offset)
                        )
                    )
                )
                .collect(),
        )
    }
}
