use std::mem;

use crate::{color, draw::screen::Canvas, editor::gadget::Gadget, grapheme::GraphemeExt, key::{KeyOrChar, key}, style::Style};

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
    fn on_key(&mut self, event: KeyOrChar) -> Option<Box<dyn FnOnce(&mut Editor)>> {
        macro_rules! xx {
            ($($tokens: tt)*) => {
                Some(Box::new($($tokens)*))
            };
        }
        use KeyOrChar::*;
        match event {
            key if let Some(char) = key.char() => {
                self.r#type(char);
                xx!(Editor::noop)
            }
    
            Key(key![backspace]) => {
                self.backspace();
                xx!(Editor::noop)
            }
    
            Key(key![return]) => {
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
        for (i, g) in (0..canvas.width()).into_iter().zip(self.name.graphemes()) {
            let cell = &mut canvas[(0, i)];
            cell.grapheme = g;
            cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into()
        }
    }
}