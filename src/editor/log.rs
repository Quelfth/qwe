use std::collections::HashSet;

use crate::{color, draw::screen::Canvas, editor::gadget::Gadget, grapheme::{Grapheme, GraphemeExt}, key::{KeyOrChar, key}, log::{LogCategory, log_iter}, style::Style};


pub struct LogViewer {
    scroll: usize,
    categories: HashSet<LogCategory>,
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            categories: [LogCategory::EditorToLspMessage].into_iter().collect(),
        }
    }
}

impl Gadget for LogViewer {
    fn on_key(&mut self, event: KeyOrChar) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
        use KeyOrChar::Key;
        match event {
            Key(key![ctrl d] | key![scroll down]) => {
                self.scroll = self.scroll.saturating_sub(1);
                Some(Box::new(|_| ()))
            },
            Key(key![ctrl u] | key![scroll up]) => {
                self.scroll += 1;
                Some(Box::new(|_| ()))
            },
            _ => None,
        }
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        let mut y = canvas.height() - 1;
        let mut log = log_iter();
        for _ in 0..self.scroll {
            log.next();
        }
        try {
            if self.scroll != 0 {
                for i in 0..canvas.width() {
                    let cell = &mut canvas[(y, i)];
                    cell.grapheme = Grapheme::DOT;
                    cell.style = (Style::fg(color::FG) + Style::bg(color::LIT_BG)).into();
                }
                y = y.checked_sub(2)?;
            }
            for log in log {
                if !self.categories.contains(&log.category) {continue}

                for line in log.message.lines().rev() {
                    for (i, g) in (0..canvas.width()).into_iter().zip(line.graphemes()) {
                        let cell = &mut canvas[(y, i)];
                        cell.grapheme = g;
                        cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into();
                    }
                    y = y.checked_sub(1)?;
                }
                let statusline = format!("    {}    {}", log.time, log.source);
                for (i, g) in (0..canvas.width()).into_iter().zip(statusline.graphemes()) {
                    let cell = &mut canvas[(y, i)];
                    cell.grapheme = g;
                    cell.style = (Style::fg(color::FG) + Style::bg(color::BG)).into();
                }
                y = y.checked_sub(2)?;
            }
        };
    }

}