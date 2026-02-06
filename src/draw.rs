use std::{
    io::{self},
    ops::Sub,
};

use crate::{
    draw::{cursor::CursorRange, screen::Screen},
    editor::{Editor, gadget::ScreenRegion},
    ix::Ix,
    terminal_size::terminal_size,
};

mod cursor;
pub mod document;
pub mod jump_labels;
pub mod screen;

#[derive(Copy, Clone)]
pub struct Range<T> {
    pub start: T,
    pub end: T,
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

// impl Range<usize> {
//     fn one(pos: usize) -> Self {
//         Self {
//             start: pos,
//             end: pos + 1,
//         }
//     }
// }

impl<U> Range<Ix<U>> {
    fn one(pos: Ix<U>) -> Self {
        Self {
            start: pos,
            end: pos + Ix::new(1),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Rect<T> {
    pub rows: Range<T>,
    pub cols: Range<T>,
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

        let doc_rect = Rect::new(0..width, 0..height);
        self.doc().draw(screen.canvas(doc_rect), |i| {
            self.doc()
                .cursors
                .as_ref()
                .map(|c| c.line_ranges(i).collect())
                .unwrap_or_default()
        });

        if let Some(gadget) = &self.gadget {
            gadget.draw(screen.canvas(match gadget.screen_region() {
                ScreenRegion::DocOverlay => self.doc().overlay_rect(doc_rect),
                ScreenRegion::RightPanel => Rect::new(width / 2..width, 0..height),
            }))
        }

        {
            let last_screen = &mut *self.screen.lock();
            screen.draw_diff(last_screen)?;
            *last_screen = screen;
        }

        Ok(())
    }
}
