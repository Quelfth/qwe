use std::collections::HashSet;

use crate::{
    color,
    draw::screen::Canvas,
    editor::gadget::Gadget,
    grapheme::{Grapheme, GraphemeExt},
    key::{KeyOrChar, key},
    log::{LogCategory, log_iter},
    style::Style
};


pub struct LogViewer {
    scroll: usize,
    categories: HashSet<LogCategory>,
    details: bool,
    selected: usize,
}

impl LogViewer {
    pub fn new() -> Self {
        use LogCategory::*;
        Self {
            scroll: 0,
            categories: [EditorToLspMessage, LspError, Debug, LspMessage, LspRequest, LspNotification, LspEvent].into_iter().collect(),
            details: false,
            selected: 0,
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
            Key(key![j]) => {
                self.selected = self.selected.saturating_sub(1);
                Some(Box::new(|_| ()))
            }
            Key(key![k]) => {
                self.selected += 1;
                Some(Box::new(|_| ()))
            }
            Key(key![ ]) => {
                self.details.toggle();
                Some(Box::new(|_| ()))
            }
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
            let selected = self.selected;
            let mut i = 0;

            for log in log {
                if !self.categories.contains(&log.category) {continue}

                let bg = if i == selected {color::LIT_BG} else {color::BG};

                if i == selected && self.details {
                    for line in log.details.lines().rev() {
                        for (i, g) in (0..canvas.width()).into_iter().zip(line.graphemes()) {
                            let cell = &mut canvas[(y, i)];
                            cell.grapheme = g;
                            cell.style = (Style::fg(color::FG) + Style::bg(bg)).into();
                        }
                        y = y.checked_sub(1)?;
                    }
                }
                for line in log.message.lines().rev() {
                    for (i, g) in (0..canvas.width()).into_iter().zip(line.graphemes()) {
                        let cell = &mut canvas[(y, i)];
                        cell.grapheme = g;
                        cell.style = (Style::fg(color::FG) + Style::bg(bg)).into();
                    }
                    y = y.checked_sub(1)?;
                }
                let statusline = format!("    {}    {}", log.time, log.source);
                for (i, g) in (0..canvas.width()).into_iter().zip(statusline.graphemes()) {
                    let cell = &mut canvas[(y, i)];
                    cell.grapheme = g;
                    cell.style = (Style::fg(color::FG) + Style::bg(bg)).into();
                }
                y = y.checked_sub(2)?;
                i += 1;
            }
        };
    }

}