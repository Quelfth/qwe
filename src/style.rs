use std::ops::Add;

use crossterm::style::{Attribute, Color, ContentStyle, Stylize};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FlatStyle {
    pub fg: Color,
    pub bg: Color,
    pub italic: bool,
    pub bold: bool,
    pub under: Option<Under>,
    pub uc: Option<Color>,
}

impl Default for FlatStyle {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            italic: Default::default(),
            bold: Default::default(),
            under: Default::default(),
            uc: Default::default(),
        }
    }
}

impl From<FlatStyle> for ContentStyle {
    fn from(
        FlatStyle {
            fg,
            bg,
            italic,
            bold,
            under,
            uc,
        }: FlatStyle,
    ) -> Self {
        let mut style = Self::new().with(fg).on(bg);
        if italic {
            style = style.italic()
        }
        if bold {
            style = style.bold()
        }
        if let Some(under) = under {
            style = style.attribute(under.into())
        }
        if let Some(uc) = uc {
            style.underline_color = Some(uc)
        }

        style
    }
}

#[derive(Default, Copy, Clone)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub italic: Option<bool>,
    pub bold: Option<bool>,
    pub under: Option<Option<Under>>,
    pub uc: Option<Option<Color>>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Under {
    Line,
    Double,
    Curl,
    Dotted,
    Dashed,
}

impl From<Under> for Attribute {
    fn from(value: Under) -> Self {
        match value {
            Under::Line => Attribute::Underlined,
            Under::Double => Attribute::DoubleUnderlined,
            Under::Curl => Attribute::Undercurled,
            Under::Dotted => Attribute::Underdotted,
            Under::Dashed => Attribute::Underdashed,
        }
    }
}

impl Add for Style {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            italic: other.italic.or(self.italic),
            bold: other.bold.or(self.bold),
            under: other.under.or(self.under),
            uc: other.uc.or(self.uc),
        }
    }
}

impl From<Style> for FlatStyle {
    fn from(value: Style) -> Self {
        let Style {
            fg,
            bg,
            italic,
            bold,
            under,
            uc,
        } = value;
        FlatStyle {
            fg: fg.unwrap_or(Color::Reset),
            bg: bg.unwrap_or(Color::Reset),
            italic: italic.unwrap_or(false),
            bold: bold.unwrap_or(false),
            under: under.flatten(),
            uc: uc.flatten(),
        }
    }
}

impl Style {
    pub fn fg(color: Color) -> Self {
        Self {
            fg: Some(color),
            ..Default::default()
        }
    }

    pub fn bg(color: Color) -> Self {
        Self {
            bg: Some(color),
            ..Default::default()
        }
    }

    pub fn italic() -> Self {
        Self {
            italic: Some(true),
            ..Default::default()
        }
    }

    pub fn bold() -> Self {
        Self {
            bold: Some(true),
            ..Default::default()
        }
    }

    pub fn uc(color: Option<Color>) -> Self {
        Self {
            uc: Some(color),
            ..Default::default()
        }
    }

    pub fn no_under() -> Self {
        Self {
            under: Some(None),
            ..Default::default()
        }
    }
}

impl From<Under> for Style {
    fn from(value: Under) -> Self {
        Self {
            under: Some(Some(value)),
            ..Default::default()
        }
    }
}
