use std::{borrow::Cow, fmt::Display};

use crate::{
    constants::TAB_WIDTH,
    ix::{Byte, Column, Ix, ix},
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

/// Don't make this public it's sus.
const fn raw(str: &'static str) -> Grapheme {
    Grapheme(Cow::Borrowed(str))
}

impl Grapheme {
    pub fn new_unchecked(data: impl AsRef<str>) -> Self {
        Self(data.as_ref().to_owned().into())
    }

    pub const SPACE: Self = raw(" ");
    pub const UPPER_LEFT_TRIANGLE: Self = raw("◤");
    pub const UPPER_RIGHT_TRIANGLE: Self = raw("◥");
    pub const LOWER_LEFT_TRIANGLE: Self = raw("◣");
    pub const LEFT_TRIANGLE: Self = raw("");
    pub const RIGHT_TRIANGLE: Self = raw("");
    pub const LEFT_SEMICIRCLE: Self = raw("");
    pub const RIGHT_SEMICIRCLE: Self = raw("");
    pub const DOT: Self = raw(".");
    pub const VERTICAL_SQUIGGLE: Self = raw("𜰊");
    pub const BRACE_2_TOP: Self = raw("⎰");
    pub const BRACE_2_BOTTOM: Self = raw("⎱");
    pub const BRACE_TOP: Self = raw("⎧");
    pub const BRACE_BAR: Self = raw("⎪");
    pub const BRACE_CUSP: Self = raw("⎨");
    pub const BRACE_BOTTOM: Self = raw("⎩");
    pub const BRACE_CUSP_TOP: Self = raw("⎭");
    pub const BRACE_CUSP_BOTTOM: Self = raw("⎫");
    pub const RULER: Self = raw("▎");
    pub const BULLET: Self = raw("◦");
    pub const RIGHT_PAREN_TOP: Self = raw("⎞");
    pub const RIGHT_PAREN_MIDDLE: Self = raw("⎟");
    pub const RIGHT_PAREN_BOTTOM: Self = raw("⎠");



    pub fn len(&self) -> Ix<Byte> {
        ix(self.0.len())
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
        use unicode_width::UnicodeWidthStr as _;
        ix(if &*self.0 == "\t" { TAB_WIDTH } else { self.0.width() })
    }

    pub fn apply_ag(&self, ag: u8) -> Option<Self> {
        if ag == 0 { return None };

        Some(Self(Cow::Borrowed(match (&*self.0, ag) {
            ("<", 1) => "⟨",
            (">", 1) => "⟩",
            _ => return None,
        })))
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
