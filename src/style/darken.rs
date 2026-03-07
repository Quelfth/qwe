use crossterm::style::Color;

pub(crate) fn darken(color: Color) -> Color {
    use Color::*;
    match color {
        DarkGrey => Black,
        Red => DarkRed,
        Green => DarkGreen,
        Yellow => DarkYellow,
        Blue => DarkBlue,
        Magenta => DarkMagenta,
        Cyan => DarkCyan,
        White => Grey,
        Rgb { r, g, b } => {
            fn darken(x: u8) -> u8 {
                x / 2 + x / 8 + x / 16
            }
            Rgb {
                r: darken(r),
                g: darken(g),
                b: darken(b),
            }
        }
        color => color,
    }
}
