use std::{
    io::{self},
    ops::Sub,
    time::{Duration, Instant},
};

use crate::{
    draw::{cursor::CursorRange, screen::{Canvas, Screen}}, editor::{Editor, gadget::ScreenRegion}, ix::Ix, presenter::{Present, Presenter}, terminal_size::terminal_size
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

impl Present for Editor {

    fn presenter(&self) -> &Presenter {
        &self.presenter
    }

    fn present(&self, mut canvas: Canvas<'_>) -> io::Result<()> {
        let width = canvas.width();
        let height = canvas.height();
        let doc_rect = Rect::new(0..width, 0..height);
        self.doc().draw(canvas.reborrow(), |i| {
            self.doc()
                .cursors
                .as_ref()
                .map(|c| c.ranges_for_line(i).collect())
                .unwrap_or_default()
        });
    
        if let Some(gadget) = &self.gadget {
            gadget.draw(canvas.region(match gadget.screen_region() {
                ScreenRegion::DocOverlay => self.doc().overlay_rect(doc_rect),
                ScreenRegion::RightPanel => Rect::new(canvas.width() / 2..canvas.width(), 0..canvas.height()),
            }))
        }

        Ok(())
    }

}

//impl Editor {
//    pub fn defer_draw(&self) {
//        const DEFER_DURATION: Duration = Duration::from_millis(50);
//        self.defer_draw_to(Instant::now() + DEFER_DURATION);
//    }
//
//    fn defer_draw_to(&self, instant: Instant) {
//        let time = &mut *self.draw_defer.lock();
//        if time.is_none() {
//            *time = Some(instant);
//        }
//    }
//
//    pub fn draw(&self) -> io::Result<()> {
//
//        const MIN_REDRAW: Duration = Duration::from_millis(8);
//        if let Some(i) = self.last_draw.get() && i.elapsed() < MIN_REDRAW {
//            self.defer_draw_to(i + MIN_REDRAW);
//            return Ok(());
//        }
//
//        let (width, height) = terminal_size();
//        let mut screen = Screen::new(width, height);
//
//
//        {
//            let last_screen = &mut *self.screen.lock();
//            screen.draw_diff(last_screen)?;
//            *last_screen = screen;
//        }
//
//        *self.draw_defer.lock() = None;
//        self.last_draw.set(Some(Instant::now()));
//        Ok(())
//    }
//}
