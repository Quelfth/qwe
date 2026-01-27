use std::ops::Range;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use regex::Regex;

use crate::{
    editor::{Editor, gadget::Gadget},
    pos::Pos,
};

pub struct Finder {
    haystack: String,
    offset: usize,
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
                    _ = e.select_ranges(f);
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
}

impl Finder {
    pub fn new(haystack: String, offset: usize) -> Self {
        Self {
            haystack,
            offset,
            regex: String::new(),
        }
    }

    pub fn r#type(&mut self, char: char) {
        self.regex.push(char);
    }

    pub fn backspace(&mut self) {
        self.regex.pop();
    }

    pub fn find(&self) -> Option<Vec<Range<usize>>> {
        let re = Regex::new(&self.regex).ok()?;

        Some(
            re.find_iter(&self.haystack)
                .map(|m| {
                    let Range { start, end } = m.range();
                    start + self.offset..end + self.offset
                })
                .collect(),
        )
    }
}
