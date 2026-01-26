use std::collections::HashMap;

use auto_enums::auto_enum;

use crate::{document::Document, pos::Pos};

pub struct JumpLabels {
    longest: usize,
    try_rev: bool,
    typed: String,
    labels: HashMap<String, Pos>,
}

impl JumpLabels {
    fn new(labels: impl IntoIterator<Item = (Pos, String)>, try_rev: bool) -> Self {
        let mut longest = 0;
        Self {
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

    pub fn generate(doc: &Document, lines: usize) -> Self {
        let first_line = doc.scroll;
        let poss: Vec<Pos> = gen {
            for (i, line) in (first_line..).zip(doc.lines_to(lines)) {
                let mut graphemes = line.graphemes().peekable();
                let mut j = 0;
                while let Some(grapheme) = graphemes.next() {
                    if grapheme.is_ident() {
                        yield Pos { line: i, column: j };
                        while let Some(grapheme) = graphemes.peek()
                            && grapheme.is_ident()
                        {
                            graphemes.next();
                            j += 1;
                        }
                    }

                    j += 1;
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

        JumpLabels::new(poss.into_iter().zip(label_gen), try_rev)
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
