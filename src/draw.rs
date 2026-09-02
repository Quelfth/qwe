use std::{
    io::{self},
    range::Range,
    ops::Sub,
};

use crate::{
    draw::{cursor::CursorRange, screen::{Canvas}},
    editor::{Editor, gadget::ScreenRegion},
    presenter::{Present, Presenter},
    util::RangeLen,
};

mod cursor;
pub mod document;
pub mod screen;

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
        &self.cmn.presenter
    }

    fn present(&self, mut canvas: Canvas<'_>) -> io::Result<()> {
        let width = canvas.width();
        let height = canvas.height();
        let doc_rect = Rect::new(0..width, 0..height);
        self.doc().draw(canvas.reborrow());
    
        if let Some(gadget) = &self.gadget {
            gadget.draw(canvas.region(match gadget.screen_region() {
                ScreenRegion::DocOverlay => self.doc().overlay_rect(doc_rect),
                ScreenRegion::RightPanel => Rect::new(canvas.width() / 2..canvas.width(), 0..canvas.height()),
            }))
        }

        Ok(())
    }

}
