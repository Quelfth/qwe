use std::range::Range;

use resharp::{Match, Regex};

use crate::{
    color, draw::screen::Canvas, editor::{Editor, gadget::Gadget}, grapheme::GraphemeExt, ix::{Byte, Ix}, key::{KeyOrChar, key}, style::Style
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
    fn on_key(&mut self, event: KeyOrChar) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
        macro_rules! xx {
            ($($tokens: tt)*) => {
                Some(Box::new($($tokens)*))
            };
        }
        use KeyOrChar::Key;
        match event {
            Key(key![backspace]) => {
                self.backspace();
                xx!(Editor::noop)
            }

            Key(key![return]) => self.find().map(|f| {
                let x: Box<dyn FnOnce(&mut Editor)> = Box::new(|e: &mut Editor| {
                    e.close_gadget();
                    if e.select_ranges(f).is_ok() && !e.doc.main_cursor_is_visible() {
                        e.scroll_to_main_cursor();
                    }
                });
                x
            }),

            key if let Some(char) = key.char() => {
                self.r#type(char);
                xx!(Editor::noop)
            }

            _ => None,
        }
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        for (i, g) in (0..canvas.width()).into_iter().zip(self.regex.graphemes()) {
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
