
use crate::{color, draw::screen::Canvas, editor::gadget::Gadget, grapheme::GraphemeExt, log::log_iter, style::Style};


pub struct LogViewer {}

impl LogViewer {
    pub const fn new() -> Self { Self { } }
}

impl Gadget for LogViewer {
    fn draw(&self, mut canvas: Canvas<'_>) {
        let mut y = canvas.height() - 1;
        let mut log = log_iter();
        try {loop {
            let Some(log) = log.next() else {break};
            for line in log.message.lines().rev() {
                for (i, g) in (0..canvas.width()).zip(line.graphemes()) {
                    let cell = &mut canvas[(y, i)];
                    cell.grapheme = g;
                    cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into();
                }
                y = y.checked_sub(1)?;
            }
            let statusline = format!("    {}    {}", log.time, log.source);
            for (i, g) in (0..canvas.width()).zip(statusline.graphemes()) {
                let cell = &mut canvas[(y, i)];
                cell.grapheme = g;
                cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into();
            }
            y = y.checked_sub(2)?;
        }};
    }
}