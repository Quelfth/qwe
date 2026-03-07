use crossterm::style::Color;

pub(crate) fn darken(color: Color, amount: u8) -> Color {
    if amount == 0 {
        return color;
    }
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
            let darken = |x: u8| match amount {
                2 => x / 2,
                _ => x / 2 + x / 4,
            };
            Rgb {
                r: darken(r),
                g: darken(g),
                b: darken(b),
            }
        }
        color => color,
    }
}
