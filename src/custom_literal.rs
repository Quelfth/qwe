pub mod integer {
    macro_rules! rgb {
        ($lit:literal) => {{
            const RGB: u32 = $lit;
            ::crossterm::style::Color::Rgb {
                r: const { (RGB >> 16) as u8 },
                g: const { (RGB >> 8) as u8 },
                b: const { RGB as u8 },
            }
        }};
    }
    pub(crate) use rgb;
}
