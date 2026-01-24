use std::{
    io::{self},
    ops::Sub,
};

use crate::{
    draw::{cursor::CursorRange, screen::Screen},
    editor::Editor,
    terminal_size::terminal_size,
};

mod cursor;
pub mod document;
pub mod screen;

#[derive(Copy, Clone)]
struct Range<T> {
    start: T,
    end: T,
}

impl<T> Range<T> {
    fn len(self) -> <T as Sub>::Output
    where
        T: Sub,
    {
        self.end - self.start
    }

    fn contains(self, pos: T) -> bool
    where
        T: PartialOrd,
    {
        pos >= self.start && pos < self.end
    }

    fn new(range: std::ops::Range<T>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl<T> From<std::ops::Range<T>> for Range<T> {
    fn from(value: std::ops::Range<T>) -> Self {
        Self::new(value)
    }
}

impl Range<usize> {
    fn one(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos + 1,
        }
    }
}

#[derive(Copy, Clone)]
struct Rect<T> {
    rows: Range<T>,
    cols: Range<T>,
}

impl<T> Rect<T> {
    fn width(self) -> <T as Sub>::Output
    where
        T: Sub,
    {
        self.cols.len()
    }

    fn height(self) -> <T as Sub>::Output
    where
        T: Sub,
    {
        self.rows.len()
    }

    fn new(cols: impl Into<Range<T>>, rows: impl Into<Range<T>>) -> Self {
        Self {
            rows: rows.into(),
            cols: cols.into(),
        }
    }
}

impl Editor {
    pub fn draw(&self) -> io::Result<()> {
        let (width, height) = terminal_size();
        let mut screen = Screen::new(width, height);

        self.doc()
            .draw(&mut screen, Rect::new(0..width, 0..height), |i| {
                self.cursors().line_ranges(i).collect()
            })?;

        if let Some(inspector) = &self.inspector {
            inspector
                .tree()
                .draw(&mut screen, Rect::new(width / 2..width, 0..height), |_| {
                    Default::default()
                })?;
        }

        {
            let last_screen = &mut *self.screen.lock();
            screen.draw_diff(last_screen)?;
            *last_screen = screen;
        }

        Ok(())
    }
}
