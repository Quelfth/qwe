use std::fmt::Display;

use crate::constants::TAB_WIDTH;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, PartialEq, Eq)]
pub struct Grapheme(Box<str>);

impl Default for Grapheme {
    fn default() -> Self {
        Self(" ".into())
    }
}

impl Display for Grapheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Grapheme {
    pub unsafe fn new_unchecked(data: impl AsRef<str>) -> Self {
        Self(data.as_ref().into())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_whitespace(&self) -> bool {
        self.0.chars().all(char::is_whitespace)
    }

    pub fn columns(&self) -> usize {
        if &*self.0 == "\t" { TAB_WIDTH } else { 1 }
    }
}

pub trait GraphemeExt {
    fn graphemes(&self) -> impl Iterator<Item = Grapheme>;
}

impl GraphemeExt for str {
    fn graphemes(&self) -> impl Iterator<Item = Grapheme> {
        UnicodeSegmentation::graphemes(self, true).map(|g| Grapheme(g.into()))
    }
}

impl GraphemeExt for String {
    fn graphemes(&self) -> impl Iterator<Item = Grapheme> {
        GraphemeExt::graphemes(&**self)
    }
}
