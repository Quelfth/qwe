
use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{color, draw::screen::Canvas, editor::gadget::Gadget, grapheme::{Grapheme, GraphemeExt}, log::{LogCategory, log_iter}, style::Style};


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
    fn on_key(&mut self, event: KeyEvent) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
        match event {
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.scroll = self.scroll.saturating_sub(1);
                Some(Box::new(|_| ()))
            },
            KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
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
            loop {
                let Some(log) = log.next() else {break};
                if !self.categories.contains(&log.category) {continue}

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
            }
        };
    }

}