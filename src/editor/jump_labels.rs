use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    custom_literal::integer::rgb,
    document::Document,
    draw::screen::Canvas,
    editor::gadget::Gadget,
    grapheme::GraphemeExt,
    ix::{Column, Ix, Line},
    pos::Pos,
    style::FlatStyle,
};

use super::{Editor, gadget::ScreenRegion};

pub struct JumpLabels {
    scroll: Ix<Line>,
    horizontal_scroll: Ix<Column>,
    longest: usize,
    try_rev: bool,
    typed: String,
    labels: HashMap<String, Pos>,
}

impl Gadget for JumpLabels {
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
                if char == ' ' {
                    return xx!(Editor::close_gadget);
                }
                self.r#type(char);

                match self.check() {
                    Ok(jump) => xx!(move |e| {
                        e.jump_to(jump);
                        e.close_gadget();
                    }),
                    Err(CheckFail::NotYet) => xx!(Editor::noop),
                    Err(CheckFail::TooLong) => xx!(Editor::close_gadget),
                }
            }
            _ => None,
        }
    }

    fn screen_region(&self) -> ScreenRegion {
        ScreenRegion::DocOverlay
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        for (pos, label) in self.labels() {
            if pos.line < self.scroll || pos.line > self.scroll + Ix::new(canvas.height() as usize) { continue }
            for (i, g) in (0..).zip(label.graphemes()) {
                if pos.column < self.horizontal_scroll || pos.column + Ix::new(i as _) > self.horizontal_scroll + Ix::new(canvas.width() as usize) {continue}
                let cell = &mut canvas[(
                    (pos.line - self.scroll).inner() as u16, 
                    (pos.column - self.horizontal_scroll).inner() as u16 + i,
                )];
                cell.grapheme = g;
                cell.style = FlatStyle {
                    fg: rgb! {0xffffff},
                    bg: cell.style.bg,
                    bold: true,
                    ..Default::default()
                }
            }
        }
    }
}

impl JumpLabels {
    fn new(
        labels: impl IntoIterator<Item = (Pos, String)>,
        try_rev: bool,
        scroll: Ix<Line>,
        horizontal_scroll: Ix<Column>,
    ) -> Self {
        let mut longest = 0;
        Self {
            scroll,
            horizontal_scroll,
            typed: String::new(),
            labels: labels
                .into_iter()
                .map(|(a, b)| {
                    longest = longest.max(b.len());
                    (b, a)
                })
                .collect(),
            try_rev,
            longest,
        }
    }

    pub fn generate(doc: &Document, lines: Ix<Line>) -> Self {
        let first_line = doc.scroll;
        let poss: Vec<Pos> = gen {
            for (i, line) in (first_line..).zip(doc.lines_to(lines)) {
                let mut graphemes = line.graphemes().peekable();
                let mut j = Ix::new(0);
                while let Some(grapheme) = graphemes.next() {
                    if grapheme.is_ident() {
                        yield Pos { line: i, column: j };
                        while let Some(grapheme) = graphemes.peek()
                            && grapheme.is_ident()
                        {
                            graphemes.next();
                            j += Ix::new(1);
                        }
                    }

                    j += Ix::new(1);
                }
            }
        }
        .collect();
        let len = poss.len();
        let (label_gen, try_rev): (&mut dyn Iterator<Item = String>, _) = match len {
            ..=150 => (&mut small_gen(), true),
            151..=676 => (&mut med_gen(), false),
            _ => panic!(),
        };

        JumpLabels::new(poss.into_iter().zip(label_gen), try_rev, doc.scroll, doc.horizontal_scroll)
    }

    pub fn r#type(&mut self, char: char) {
        self.typed.push(char);
    }

    pub fn check(&self) -> Result<Pos, CheckFail> {
        if self.typed.len() > self.longest {
            return Err(CheckFail::TooLong);
        }
        self.labels
            .get(&self.typed)
            .or_else(|| {
                self.try_rev.then_some(()).and_then(|()| {
                    self.labels
                        .get(&self.typed.chars().rev().collect::<String>())
                })
            })
            .copied()
            .ok_or(CheckFail::NotYet)
    }

    pub fn labels(&self) -> impl Iterator<Item = (Pos, &str)> {
        self.labels.iter().map(|(s, p)| (*p, &**s))
    }
}

pub enum CheckFail {
    NotYet,
    TooLong,
}

fn small_gen() -> impl Iterator<Item = String> {
    gen {
        for first in [
            'a', 's', 'd', 'f', 'q', 'z', 'w', 'x', 'e', 'c', 'r', 'v', 'g', 't', 'b',
        ] {
            for second in ['j', 'k', 'l', 'h', 'u', 'n', 'i', 'm', 'o', 'p'] {
                yield format!("{first}{second}");
            }
        }
    }
}

fn med_gen() -> impl Iterator<Item = String> {
    gen {
        for first in 'a'..='z' {
            for second in 'a'..='z' {
                yield format!("{first}{second}");
            }
        }
    }
}
