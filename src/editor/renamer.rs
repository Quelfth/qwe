use std::mem;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{color, draw::screen::Canvas, editor::gadget::Gadget, grapheme::GraphemeExt, style::Style};

use super::Editor;


pub struct Renamer {
    name: String,
}

impl Renamer {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn r#type(&mut self, char: char) {
        self.name.push(char);
    }
    
    pub fn backspace(&mut self) {
        self.name.pop();
    }
}

impl Gadget for Renamer {
    fn on_key(&mut self, event: KeyEvent) -> Option<Box<dyn FnOnce(&mut Editor)>> {
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
            } => {
                let name = mem::take(&mut self.name);
                xx!(move |e| {
                    e.complete_rename(name);
                    e.close_gadget()
                })
            },
            
            _ => None,
        }
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        for (i, g) in (0..canvas.width()).zip(self.name.graphemes()) {
            let cell = &mut canvas[(0, i)];
            cell.grapheme = g;
            cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into()
        }
    }
}