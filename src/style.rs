use std::ops::Add;

use crossterm::style::{Attribute, Color, ContentStyle, Stylize};

use crate::style::darken::darken;

mod darken;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FlatStyle {
    pub fg: Color,
    pub bg: Color,
    pub italic: bool,
    pub bold: bool,
    pub under: Option<Under>,
    pub uc: Option<Color>,
    pub overline: bool,
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
            overline: Default::default(),
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
            overline,
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
        if overline {
            style = style.attribute(Attribute::OverLined)
        }

        style
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub italic: Option<bool>,
    pub bold: Option<bool>,
    pub under: Option<Option<Under>>,
    pub uc: Option<Option<Color>>,
    pub dark: Option<u8>,
}

impl Style {
    pub fn is_none(self) -> bool {
        self == Self::default()
    }
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
            dark: other.dark.or(self.dark),
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
            dark,
        } = value;
        let fg = fg.unwrap_or(Color::Reset);
        let fg = if let Some(dark) = dark {
            darken(fg, dark)
        } else {
            fg
        };
        FlatStyle {
            fg,
            bg: bg.unwrap_or(Color::Reset),
            italic: italic.unwrap_or(false),
            bold: bold.unwrap_or(false),
            under: under.flatten(),
            uc: uc.flatten(),
            overline: false,
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
        Self::italic_bool(true)
    }

    pub fn not_italic() -> Self {
        Self::italic_bool(false)
    }

    pub fn italic_bool(value: bool) -> Self {
        Self {
            italic: Some(value),
            ..Default::default()
        }
    }

    pub fn bold() -> Self {
        Self::bold_bool(true)
    }

    pub fn not_bold() -> Self {
        Self::bold_bool(false)
    }

    pub fn bold_bool(bool: bool) -> Self {
        Self {
            bold: Some(bool),
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

    pub fn dark(amount: u8) -> Self {
        Self {
            dark: Some(amount),
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

impl sulu::Style for Style {
    fn none() -> Self {
        Self::default()
    }

    fn color(name: &str, sulu::Color { r, g, b }: sulu::Color) -> Option<Self> {
        let color = Color::Rgb { r, g, b };
        match name {
            "f" => Some(Self::fg(color)),
            "b" => Some(Self::bg(color)),
            "u" => Some(Self::uc(Some(color))),
            _ => None,
        }
    }

    fn bool(name: &str, value: bool) -> Option<Self> {
        match name {
            "i" => Some(Self::italic_bool(value)),
            "b" => Some(Self::bold_bool(value)),
            "u" if !value => Some(Self::no_under()),
            "d" => Some(Self::dark(1)),
            _ => None,
        }
    }

    fn value(name: &str, value: &str) -> Result<Self, sulu::StyleValueError> {
        match name {
            "u" => match value {
                "line" => Ok(Under::Line.into()),
                "double" => Ok(Under::Double.into()),
                "curl" => Ok(Under::Curl.into()),
                "dotted" => Ok(Under::Dotted.into()),
                "dashed" => Ok(Under::Dashed.into()),
                _ => Err(sulu::StyleValueError::Value),
            },
            "d" => {
                if let Ok(value) = value.parse::<u8>() {
                    Ok(Self::dark(value))
                } else {
                    Err(sulu::StyleValueError::Value)
                }
            }
            _ => Err(sulu::StyleValueError::Name),
        }
    }

    fn combine(self, top: &Self) -> Self {
        self + *top
    }
}
