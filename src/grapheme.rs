use std::{borrow::Cow, fmt::Display};

use crate::{
    constants::TAB_WIDTH,
    ix::{Byte, Column, Ix},
};

use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Grapheme(Cow<'static, str>);

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
        Self(data.as_ref().to_owned().into())
    }

    pub const SPACE: Self = Self(Cow::Borrowed(" "));
    pub const UPPER_LEFT_TRIANGLE: Self = Self(Cow::Borrowed("◤"));
    pub const UPPER_RIGHT_TRIANGLE: Self = Self(Cow::Borrowed("◥"));
    pub const LOWER_LEFT_TRIANGLE: Self = Self(Cow::Borrowed("◣"));
    pub const LEFT_TRIANGLE: Self = Self(Cow::Borrowed(""));
    pub const RIGHT_TRIANGLE: Self = Self(Cow::Borrowed(""));
    pub const LEFT_SEMICIRCLE: Self = Self(Cow::Borrowed(""));
    pub const RIGHT_SEMICIRCLE: Self = Self(Cow::Borrowed(""));
    pub const DOT: Self = Self(Cow::Borrowed("."));

    pub fn len(&self) -> Ix<Byte> {
        Ix::new(self.0.len())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_whitespace(&self) -> bool {
        self.0.chars().all(char::is_whitespace)
    }

    pub fn is_newline(&self) -> bool {
        self.0.chars().any(|c| c == '\n')
    }

    pub fn is_ident(&self) -> bool {
        self.0.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    pub fn columns(&self) -> Ix<Column> {
        Ix::new(if &*self.0 == "\t" { TAB_WIDTH } else { 1 })
    }
}

pub trait GraphemeExt {
    fn graphemes(&self) -> impl Iterator<Item = Grapheme>;
}

impl GraphemeExt for str {
    fn graphemes(&self) -> impl Iterator<Item = Grapheme> {
        UnicodeSegmentation::graphemes(self, true).map(|g| Grapheme(g.to_owned().into()))
    }
}

impl GraphemeExt for String {
    fn graphemes(&self) -> impl Iterator<Item = Grapheme> {
        GraphemeExt::graphemes(&**self)
    }
}
