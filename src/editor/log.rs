
use crate::{draw::screen::Canvas, editor::gadget::Gadget};


pub struct LogViewer {}

impl Gadget for LogViewer {
    fn draw(&self, canvas: Canvas<'_>) {}
}