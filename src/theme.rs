use std::sync::LazyLock;

use sulu::Theme;

use crate::style::Style;

pub const THEME_SOURCE: &str = include_str!("../errata.sulu");

pub fn theme() -> &'static Theme<Style> {
    static THEME: LazyLock<Theme<Style>> = LazyLock::new(|| sulu::parse(THEME_SOURCE).unwrap());
    &THEME
}
