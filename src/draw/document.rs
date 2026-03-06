use crate::{
    document::Document,
    draw::screen::Canvas,
    ix::{Ix, Line},
};

use super::CursorRange;

pub mod badges;
pub mod highlight;
pub mod main;
pub mod query;

impl Document {
    pub fn draw(&self, mut canvas: Canvas<'_>, cursors: impl Fn(Ix<Line>) -> Vec<CursorRange>) {
        self.main_draw(canvas.reborrow(), cursors);
        self.draw_edge_indicators(canvas);
    }
}
