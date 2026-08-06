use crate::{
    draw::screen::Canvas, editor::gadget::Gadget, ix::ix, key::{KeyOrChar, key}
};

use super::{
    Editor,
    gadget::ScreenRegion,
};


pub struct LineJumper {
    line: u32,
}

impl LineJumper {
    pub fn new() -> Self {
        Self {
            line: 0,
        }
    }
}

impl Gadget for LineJumper {
    fn on_key(&mut self, event: KeyOrChar) -> Option<Box<dyn FnOnce(&mut Editor)>> {
        match event {
            key if let Some(char) = key.char() && let Some(digit) = char.to_digit(10) => {
                self.line *= 10;
                self.line += digit;
                Some(Box::new(|_|()))
            }
            KeyOrChar::Key(key![backspace]) => {
                self.line /= 10;
                Some(Box::new(|_|()))
            }
            KeyOrChar::Key(key![return] | key![ ]) => {
                let line = self.line;
                if line == 0 {
                    Some(Box::new(move |editor| {
                        editor.doc.scroll = editor.doc.text().line_len().saturating_sub(*editor.doc.view_height.lock());
                        editor.close_gadget();
                    }))
                } else {
                    Some(Box::new(move |editor| {
                        editor.doc.scroll = ix(line as _).saturating_sub(*editor.doc.view_height.lock() / 2);
                        editor.close_gadget();
                    }))
                }
            }
            _ => None
        }
    }

    fn screen_region(&self) -> ScreenRegion {
        ScreenRegion::RightPanel
    }

    fn draw(&self, _: Canvas<'_>) {}
}
